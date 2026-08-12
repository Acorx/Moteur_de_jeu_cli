#![cfg(feature = "gltf-import")]

use std::fs;
use std::path::PathBuf;

use aetherion::gltf3d;
use aetherion::render3d;

fn temporary_directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "aetherion-gltf-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn imports_triangle_nodes_materials_and_publishes_canonical_scene() {
    let directory = temporary_directory();
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("triangle.gltf");
    let output = directory.join("scene.json");
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/triangle.gltf"),
        &input,
    )
    .unwrap();

    let summary = gltf3d::import(&input, &output).unwrap();
    assert_eq!(summary.schema, "aetherion.gltf-import/v1");
    assert_eq!(summary.meshes, 1);
    assert_eq!(summary.materials, 1);
    assert_eq!(summary.objects, 1);
    assert_eq!(summary.triangles, 1);
    assert!(summary.textures_ignored);

    let scene = render3d::load(&output).unwrap();
    assert_eq!(scene.meshes[0].vertices[0].x, 0);
    assert_eq!(scene.meshes[0].vertices[0].y, 2000);
    assert_eq!(scene.meshes[0].vertices[0].z, 1000);
    assert_eq!(scene.materials[0].color, [255, 0, 0]);
    assert_eq!(scene.materials[0].opacity, 1000);
    assert!(
        String::from_utf8(fs::read(&output).unwrap())
            .unwrap()
            .ends_with('\n')
    );

    fs::remove_dir_all(directory).unwrap();
}
