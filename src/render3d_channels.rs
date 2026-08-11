use std::path::{Path, PathBuf};

use crate::Result;
use crate::capture::{Channel, Channels};
use crate::render::{Image, RenderBuffers};
use crate::render3d::{Capture3dChannel, Capture3dSegmentation};

pub(crate) struct ChannelBatch {
    pub files: Vec<ChannelFile>,
    pub manifest: Option<Vec<Capture3dChannel>>,
    pub segmentation_mapping: Option<Vec<Capture3dSegmentation>>,
}

pub(crate) struct ChannelFile {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

pub(crate) fn build(
    output: &Path,
    buffers: &RenderBuffers,
    channels: &Channels,
    mapping: &[Capture3dSegmentation],
) -> Result<ChannelBatch> {
    let mut files = Vec::new();
    let mut manifest = Vec::new();
    if channels.contains(Channel::Depth) {
        push(
            &mut files,
            &mut manifest,
            output,
            "depth",
            "depth.pgm",
            "pgm-p5-u16be",
            encode_depth(buffers),
        );
    }
    if channels.contains(Channel::Normals) {
        push(
            &mut files,
            &mut manifest,
            output,
            "normals",
            "normals.ppm",
            "ppm-p6-rgb8",
            crate::render::encode_ppm(&buffers.normals),
        );
    }
    if channels.contains(Channel::Segmentation) {
        push(
            &mut files,
            &mut manifest,
            output,
            "segmentation",
            "segmentation.ppm",
            "ppm-p6-rgb24-id",
            encode_segmentation(buffers),
        );
    }
    Ok(ChannelBatch {
        files,
        manifest: (!channels.is_default()).then_some(manifest),
        segmentation_mapping: channels
            .contains(Channel::Segmentation)
            .then(|| mapping.to_vec()),
    })
}

fn push(
    files: &mut Vec<ChannelFile>,
    manifest: &mut Vec<Capture3dChannel>,
    output: &Path,
    name: &'static str,
    suffix: &str,
    encoding: &'static str,
    bytes: Vec<u8>,
) {
    let path = auxiliary_path(output, suffix);
    manifest.push(Capture3dChannel {
        name,
        file: normalized(&path),
        encoding,
    });
    files.push(ChannelFile { path, bytes });
}

fn auxiliary_path(output: &Path, suffix: &str) -> PathBuf {
    let stem = output
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("capture3d");
    output.with_file_name(format!("{stem}.{suffix}"))
}

fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn encode_depth(buffers: &RenderBuffers) -> Vec<u8> {
    let mut bytes = format!(
        "P5\n{} {}\n65535\n",
        buffers.color.width, buffers.color.height
    )
    .into_bytes();
    for value in &buffers.depth {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}

fn encode_segmentation(buffers: &RenderBuffers) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(buffers.segmentation.len() * 3);
    for value in &buffers.segmentation {
        pixels.extend_from_slice(&value.to_be_bytes()[1..]);
    }
    crate::render::encode_ppm(&Image {
        width: buffers.color.width,
        height: buffers.color.height,
        pixels,
    })
}

pub(crate) fn publish_atomic(
    output: &Path,
    image: &[u8],
    manifest: &Path,
    manifest_bytes: &[u8],
    auxiliary: &[ChannelFile],
) -> Result<()> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("capture3d_output: {}: {error}", parent.display()))?;
    let mut targets = vec![output.to_path_buf(), manifest.to_path_buf()];
    targets.extend(auxiliary.iter().map(|file| file.path.clone()));
    if targets.iter().any(|path| path.exists()) {
        return Err("capture3d_output_exists: un fichier cible existe déjà".into());
    }
    let stage = parent.join(format!(".aetherion-capture3d-stage-{}", std::process::id()));
    if stage.exists() {
        std::fs::remove_dir_all(&stage)
            .map_err(|error| format!("capture3d_stage_cleanup: {error}"))?;
    }
    std::fs::create_dir(&stage).map_err(|error| format!("capture3d_stage_create: {error}"))?;
    let result = (|| -> Result<()> {
        let mut entries: Vec<(&Path, &[u8])> = vec![(output, image), (manifest, manifest_bytes)];
        entries.extend(
            auxiliary
                .iter()
                .map(|file| (file.path.as_path(), file.bytes.as_slice())),
        );
        for (index, (_, bytes)) in entries.iter().enumerate() {
            std::fs::write(stage.join(index.to_string()), bytes)
                .map_err(|error| format!("capture3d_stage_write: {error}"))?;
        }
        let mut published = Vec::new();
        for (index, (target, _)) in entries.iter().enumerate() {
            if let Err(error) = std::fs::rename(stage.join(index.to_string()), target) {
                for path in published {
                    let _ = std::fs::remove_file(path);
                }
                return Err(format!("capture3d_publish: {error}").into());
            }
            published.push(*target);
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&stage);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffers() -> RenderBuffers {
        RenderBuffers {
            color: Image {
                width: 2,
                height: 1,
                pixels: vec![1, 2, 3, 4, 5, 6],
            },
            depth: vec![0, 0x1234],
            normals: Image {
                width: 2,
                height: 1,
                pixels: vec![128, 128, 255, 1, 2, 3],
            },
            segmentation: vec![0, 0x010203],
            segmentation_mapping: Vec::new(),
        }
    }

    #[test]
    fn encoders_are_big_endian_and_deterministic() {
        let value = buffers();
        assert!(encode_depth(&value).ends_with(&[0, 0, 0x12, 0x34]));
        assert!(encode_segmentation(&value).ends_with(&[0, 0, 0, 1, 2, 3]));
    }

    #[test]
    fn all_three_channels_have_stable_names_and_metadata() {
        let channels = Channels::parse("color,depth,normals,segmentation").unwrap();
        let batch = build(Path::new("out/frame.ppm"), &buffers(), &channels, &[]).unwrap();
        let metadata = batch.manifest.unwrap();
        assert_eq!(
            metadata.iter().map(|item| item.name).collect::<Vec<_>>(),
            vec!["depth", "normals", "segmentation"]
        );
        assert_eq!(metadata[0].file, "out/frame.depth.pgm");
        assert_eq!(metadata[1].encoding, "ppm-p6-rgb8");
        assert_eq!(metadata[2].encoding, "ppm-p6-rgb24-id");
    }
}
