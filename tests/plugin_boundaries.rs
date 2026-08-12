#![cfg(feature = "plugin-runtime")]

use aetherion::plugin::{
    self, Capability, HOST_ABI_MAJOR, HOST_ABI_MINOR, MANIFEST_SCHEMA, MAX_FILES, MAX_IO_BYTES,
    PluginAbi, PluginManifest, PluginQuotas,
};
use aetherion::plugin_runtime::{
    self, DEFAULT_ENTRYPOINT, FILES_QUOTA_ERROR, HOST_MODULE, HostContext, IO_READ_QUOTA_ERROR,
    NETWORK_DENIED_ERROR,
};

const RETURN_SEVEN: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65, 0x74, 0x68, 0x65, 0x72, 0x69, 0x6f, 0x6e,
    0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x07, 0x0b,
];

fn manifest(capabilities: Vec<Capability>) -> PluginManifest {
    PluginManifest {
        schema: MANIFEST_SCHEMA.into(),
        id: "org.aetherion.boundaries".into(),
        version: "1.0.0".into(),
        abi: PluginAbi {
            major: HOST_ABI_MAJOR,
            minimum_host_minor: HOST_ABI_MINOR,
        },
        capabilities,
        quotas: PluginQuotas {
            memory_bytes: 65536,
            fuel: 1000,
            io_read_bytes: 1,
            io_write_bytes: 0,
            files: 1,
        },
    }
}

fn leb(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            return bytes;
        }
    }
}

fn wasm_with_import(
    module_name: &str,
    import_name: &str,
    function_type: &[u8],
    body: &[u8],
) -> Vec<u8> {
    fn string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend(leb(value.len() as u32));
        bytes.extend(value.as_bytes());
    }
    fn section(bytes: &mut Vec<u8>, id: u8, payload: &[u8]) {
        bytes.push(id);
        bytes.extend(leb(payload.len() as u32));
        bytes.extend(payload);
    }

    let mut module = b"\0asm\x01\0\0\0".to_vec();
    let mut types = vec![2];
    types.extend(function_type);
    types.extend([0x60, 0, 1, 0x7f]);
    section(&mut module, 1, &types);
    let mut imports = vec![1];
    string(&mut imports, module_name);
    string(&mut imports, import_name);
    imports.extend([0, 0]);
    section(&mut module, 2, &imports);
    section(&mut module, 3, &[1, 1]);
    let mut exports = vec![1];
    string(&mut exports, DEFAULT_ENTRYPOINT);
    exports.extend([0, 1]);
    section(&mut module, 7, &exports);
    let mut code = vec![1];
    let mut function = vec![0];
    function.extend(body);
    function.push(0x0b);
    code.extend(leb(function.len() as u32));
    code.extend(function);
    section(&mut module, 10, &code);
    module
}

fn asset_read_module() -> Vec<u8> {
    wasm_with_import(
        HOST_MODULE,
        "asset_read_byte",
        &[0x60, 2, 0x7f, 0x7f, 1, 0x7f],
        &[0x41, 0, 0x41, 0, 0x10, 0],
    )
}

#[test]
fn boundary_corpus_keeps_stable_error_prefixes() {
    let base = manifest(vec![]);
    let cases = [
        (
            "empty",
            plugin_runtime::execute_bytes_with_manifest(
                &[],
                DEFAULT_ENTRYPOINT,
                &base,
                HostContext::default(),
            ),
            "plugin_runtime_module_empty",
        ),
        (
            "invalid",
            plugin_runtime::execute_bytes_with_manifest(
                &[0, 1, 2],
                DEFAULT_ENTRYPOINT,
                &base,
                HostContext::default(),
            ),
            "plugin_runtime_compile",
        ),
        (
            "missing-export",
            plugin_runtime::execute_bytes_with_manifest(
                RETURN_SEVEN,
                "missing",
                &base,
                HostContext::default(),
            ),
            "plugin_runtime_export",
        ),
        (
            "unknown-host-import",
            plugin_runtime::execute_bytes_with_manifest(
                &wasm_with_import(HOST_MODULE, "unknown", &[0x60, 0, 1, 0x7f], &[0x10, 0]),
                DEFAULT_ENTRYPOINT,
                &base,
                HostContext::default(),
            ),
            "plugin_runtime_import_unknown",
        ),
        (
            "network-import",
            plugin_runtime::execute_bytes_with_manifest(
                &wasm_with_import(
                    "wasi_snapshot_preview1",
                    "fd_write",
                    &[0x60, 0, 1, 0x7f],
                    &[0x10, 0],
                ),
                DEFAULT_ENTRYPOINT,
                &base,
                HostContext::default(),
            ),
            NETWORK_DENIED_ERROR,
        ),
    ];
    for (label, result, prefix) in cases {
        let error = result.unwrap_err();
        assert!(
            error.message.starts_with(prefix),
            "{label}: {}",
            error.message
        );
    }
}

#[test]
fn boundary_corpus_covers_signature_and_quota_edges() {
    let mut wrong_signature = RETURN_SEVEN.to_vec();
    wrong_signature[14] = 0x7e;
    let return_opcode = wrong_signature.len() - 3;
    wrong_signature[return_opcode] = 0x42;
    let error = plugin_runtime::execute_bytes_with_manifest(
        &wrong_signature,
        DEFAULT_ENTRYPOINT,
        &manifest(vec![]),
        HostContext::default(),
    )
    .unwrap_err();
    assert!(
        error.message.starts_with("plugin_runtime_export"),
        "{}",
        error.message
    );

    let host = HostContext::default()
        .with_asset_bytes("payload", vec![7])
        .unwrap();
    let mut no_read = manifest(vec![Capability::AssetRead]);
    no_read.quotas.io_read_bytes = 0;
    let error = plugin_runtime::execute_bytes_with_manifest(
        &asset_read_module(),
        DEFAULT_ENTRYPOINT,
        &no_read,
        host.clone(),
    )
    .unwrap_err();
    assert!(
        error.message.starts_with(IO_READ_QUOTA_ERROR),
        "{}",
        error.message
    );

    let mut no_files = manifest(vec![Capability::AssetRead]);
    no_files.quotas.files = 0;
    let error = plugin_runtime::execute_bytes_with_manifest(
        &asset_read_module(),
        DEFAULT_ENTRYPOINT,
        &no_files,
        host,
    )
    .unwrap_err();
    assert!(
        error.message.starts_with(FILES_QUOTA_ERROR),
        "{}",
        error.message
    );

    let mut maximums = manifest(vec![]);
    maximums.quotas.io_read_bytes = MAX_IO_BYTES;
    maximums.quotas.io_write_bytes = MAX_IO_BYTES;
    maximums.quotas.files = MAX_FILES;
    assert!(plugin::validate(&maximums).is_ok());
    maximums.quotas.io_read_bytes = MAX_IO_BYTES + 1;
    assert!(
        plugin::validate(&maximums)
            .unwrap_err()
            .message
            .starts_with("plugin_quota_io")
    );
}

#[test]
fn boundary_corpus_rejects_unknown_and_duplicate_manifest_capabilities() {
    let unknown = r#"{"schema":"aetherion.plugin/v1","id":"org.aetherion.boundaries","version":"1.0.0","abi":{"major":1,"minimum_host_minor":1},"capabilities":["filesystem"],"quotas":{"memory_bytes":65536,"fuel":1000,"io_read_bytes":0,"io_write_bytes":0,"files":0}}"#;
    let error = serde_json::from_str::<PluginManifest>(unknown).unwrap_err();
    assert!(error.to_string().contains("unknown variant"));

    let mut duplicate = manifest(vec![Capability::AssetRead, Capability::AssetRead]);
    let error = plugin::validate(&duplicate).unwrap_err();
    assert!(error.message.starts_with("plugin_capability_duplicate"));
    duplicate.capabilities.clear();
    assert!(plugin::validate(&duplicate).is_ok());
}
