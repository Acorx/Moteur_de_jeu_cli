"""Generate deterministic external-asset GPU benchmark scenes."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

SCENE_SCHEMA = "aetherion.scene3d/v1"
ASSETS_SCHEMA = "aetherion.assets3d/v1"
MESH_SCHEMA = "aetherion.mesh3d/v1"
MATERIAL_SCHEMA = "aetherion.material3d/v1"
PRESETS = (1_000, 10_000, 100_000)


def checksum(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def write_json(path: Path, value: object) -> bytes:
    data = (json.dumps(value, ensure_ascii=True, indent=2) + "\n").encode("utf-8")
    path.write_bytes(data)
    return data


def build_mesh(triangles: int) -> dict[str, object]:
    # Reuse vertices so the 100k preset stays below the 16 MiB asset quota.
    columns = 512
    vertex_count = triangles + columns + 1
    vertices = [
        {
            "x": (index % columns) * 4 - 1_024,
            "y": (index // columns) * 4 - 400,
            "z": 10,
        }
        for index in range(vertex_count)
    ]
    faces = [[index, index + 1, index + columns] for index in range(triangles)]
    return {
        "schema": MESH_SCHEMA,
        "mesh": {
            "id": "benchmark-mesh",
            "vertices": vertices,
            "triangles": faces,
        },
    }


def build_material() -> dict[str, object]:
    return {
        "schema": MATERIAL_SCHEMA,
        "material": {"id": "benchmark-material", "color": [90, 170, 255], "opacity": 1000},
    }


def build_scene() -> dict[str, object]:
    return {
        "schema": SCENE_SCHEMA,
        "camera": {"x": 0, "y": 0, "z": 0, "pixels_per_unit": 16},
        "background": [12, 16, 24],
        "meshes": [],
        "materials": [],
        "objects": [
            {
                "id": "benchmark-object",
                "mesh": "benchmark-mesh",
                "material": "benchmark-material",
            }
        ],
    }


def build_manifest(mesh_size: int, material_size: int, mesh_checksum: int, material_checksum: int) -> dict[str, object]:
    return {
        "schema": ASSETS_SCHEMA,
        "assets": [
            {
                "id": "benchmark-material",
                "path": "material.json",
                "type": "material",
                "size": material_size,
                "checksum": material_checksum,
            },
            {
                "id": "benchmark-mesh",
                "path": "mesh.json",
                "type": "mesh",
                "size": mesh_size,
                "checksum": mesh_checksum,
            },
        ],
    }


def generate(output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    for triangles in PRESETS:
        directory = output_dir / f"triangles-{triangles // 1_000}k"
        directory.mkdir(parents=True, exist_ok=True)
        mesh = build_mesh(triangles)
        material = build_material()
        mesh_bytes = write_json(directory / "mesh.json", mesh)
        material_bytes = write_json(directory / "material.json", material)
        write_json(directory / "scene.json", build_scene())
        write_json(
            directory / "assets.json",
            build_manifest(
                len(mesh_bytes),
                len(material_bytes),
                checksum(mesh_bytes),
                checksum(material_bytes),
            ),
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    generate(args.output_dir)


if __name__ == "__main__":
    main()
