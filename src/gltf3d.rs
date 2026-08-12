use std::path::Path;

use serde::Serialize;

use crate::Result;

pub const REPORT_SCHEMA: &str = "aetherion.gltf-import/v1";
pub const GLTF_UNIT_SCALE: i32 = 1000;
pub const MAX_INPUT_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_BUFFER_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_IMAGES: usize = 4096;

#[derive(Clone, Debug, Serialize)]
pub struct ImportSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub meshes: usize,
    pub materials: usize,
    pub objects: usize,
    pub triangles: usize,
    pub unit_scale: i32,
    pub textures_ignored: bool,
}

#[cfg(feature = "gltf-import")]
mod runtime {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use glam::{Mat4, Vec4};
    use gltf::mesh::Mode;
    use serde_json::to_vec_pretty;

    use super::{
        GLTF_UNIT_SCALE, ImportSummary, MAX_BUFFER_BYTES, MAX_IMAGES, MAX_INPUT_BYTES,
        REPORT_SCHEMA,
    };
    use crate::Result;
    use crate::render3d::{
        self, Material3d, Mesh3d, NORMAL_SCALE, Object3d, SCENE_SCHEMA, Scene3d, Transform3d,
        UV_SCALE,
    };

    struct ImportContext {
        buffers: Vec<gltf::buffer::Data>,
        materials: BTreeMap<String, Material3d>,
        meshes: Vec<Mesh3d>,
        objects: Vec<Object3d>,
        triangles: usize,
    }

    pub fn import(input: &Path, output: &Path) -> Result<ImportSummary> {
        let metadata = fs::metadata(input)
            .map_err(|error| format!("gltf_import_read: {}: {error}", input.display()))?;
        if metadata.len() > MAX_INPUT_BYTES {
            return Err("gltf_import_too_large: plafond 16777216 octets".into());
        }
        let gltf =
            gltf::Gltf::open(input).map_err(|error| format!("gltf_import_invalid: {error}"))?;
        let gltf::Gltf { document, blob } = gltf;
        if document.images().count() > MAX_IMAGES {
            return Err("gltf_import_image_quota: plafond 4096 images".into());
        }
        let base = input
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let buffers = gltf::import_buffers(&document, Some(base), blob)
            .map_err(|error| format!("gltf_import_buffers_invalid: {error}"))?;
        let buffer_bytes = buffers.iter().try_fold(0_u64, |total, buffer| {
            total
                .checked_add(buffer.0.len() as u64)
                .ok_or("gltf_import_buffer_quota")
        })?;
        if buffer_bytes > MAX_BUFFER_BYTES {
            return Err("gltf_import_buffer_quota: plafond 67108864 octets".into());
        }

        let mut context = ImportContext {
            buffers,
            materials: BTreeMap::new(),
            meshes: Vec::new(),
            objects: Vec::new(),
            triangles: 0,
        };
        let scene = document
            .default_scene()
            .or_else(|| document.scenes().next())
            .ok_or("gltf_import_scene_missing")?;
        for node in scene.nodes() {
            visit_node(&mut context, node, Mat4::IDENTITY)?;
        }

        let triangle_count = context.triangles;
        let scene = Scene3d {
            schema: SCENE_SCHEMA.into(),
            camera: render3d::Camera3d::default(),
            background: [16, 20, 28],
            triangles: Vec::new(),
            meshes: context.meshes,
            materials: context.materials.into_values().collect(),
            objects: context.objects,
            animations: Vec::new(),
        };
        render3d::validate(&scene)?;
        publish_scene(output, &scene)?;
        Ok(ImportSummary {
            schema: REPORT_SCHEMA,
            status: "imported",
            meshes: scene.meshes.len(),
            materials: scene.materials.len(),
            objects: scene.objects.len(),
            triangles: triangle_count,
            unit_scale: GLTF_UNIT_SCALE,
            textures_ignored: true,
        })
    }

    fn visit_node(
        context: &mut ImportContext,
        node: gltf::Node<'_>,
        parent_transform: Mat4,
    ) -> Result<()> {
        let transform = parent_transform * Mat4::from_cols_array_2d(&node.transform().matrix());
        if let Some(mesh) = node.mesh() {
            for (primitive_index, primitive) in mesh.primitives().enumerate() {
                if primitive.mode() != Mode::Triangles {
                    return Err(format!(
                        "gltf_import_primitive_mode: node {} primitive {}",
                        node.index(),
                        primitive_index
                    )
                    .into());
                }
                let reader = primitive.reader(|buffer| {
                    context
                        .buffers
                        .get(buffer.index())
                        .map(|data| data.0.as_slice())
                });
                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or("gltf_import_positions_missing")?
                    .collect();
                let normals = reader
                    .read_normals()
                    .map(|values| {
                        values
                            .map(|normal| transform_normal(transform, normal))
                            .collect::<Result<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default();
                if !normals.is_empty() && normals.len() != positions.len() {
                    return Err("gltf_import_normals_length_invalid".into());
                }
                let uvs = reader
                    .read_tex_coords(0)
                    .map(|values| {
                        values
                            .into_f32()
                            .map(|uv| Ok([quantize_uv(uv[0])?, quantize_uv(uv[1])?]))
                            .collect::<Result<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default();
                if !uvs.is_empty() && uvs.len() != positions.len() {
                    return Err("gltf_import_uvs_length_invalid".into());
                }
                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|indices| indices.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect());
                if !indices.len().is_multiple_of(3) {
                    return Err("gltf_import_indices_invalid".into());
                }
                for index in &indices {
                    if *index as usize >= positions.len() {
                        return Err("gltf_import_index_out_of_bounds".into());
                    }
                }
                let mesh_id = format!("gltf-node-{}-mesh-{}", node.index(), primitive_index);
                let object_id = format!("gltf-node-{}-object-{}", node.index(), primitive_index);
                let vertices = positions
                    .into_iter()
                    .map(|position| transform_position(transform, position))
                    .collect::<Result<Vec<_>>>()?;
                let material_id = material_id(context, primitive.material())?;
                let triangles = indices
                    .chunks_exact(3)
                    .map(|face| [face[0], face[1], face[2]])
                    .collect::<Vec<_>>();
                context.triangles = context
                    .triangles
                    .checked_add(triangles.len())
                    .ok_or("gltf_import_triangle_quota")?;
                context.meshes.push(Mesh3d {
                    id: mesh_id.clone(),
                    vertices,
                    triangles,
                    normals,
                    uvs,
                });
                context.objects.push(Object3d {
                    id: object_id,
                    mesh: mesh_id,
                    material: material_id,
                    transform: Transform3d::default(),
                    lods: Vec::new(),
                });
            }
        }
        for child in node.children() {
            visit_node(context, child, transform)?;
        }
        Ok(())
    }

    fn transform_position(transform: Mat4, position: [f32; 3]) -> Result<render3d::Vertex3> {
        let value = transform * Vec4::new(position[0], position[1], position[2], 1.0);
        if !value.x.is_finite() || !value.y.is_finite() || !value.z.is_finite() || value.w == 0.0 {
            return Err("gltf_import_transform_invalid".into());
        }
        let divided = value.truncate() / value.w;
        Ok(render3d::Vertex3 {
            x: quantize(divided.x)?,
            y: quantize(divided.y)?,
            z: quantize(divided.z)?,
        })
    }

    fn transform_normal(transform: Mat4, normal: [f32; 3]) -> Result<[i32; 3]> {
        if normal.iter().any(|value| !value.is_finite()) {
            return Err("gltf_import_normal_invalid".into());
        }
        let inverse_transpose = transform.inverse().transpose();
        let value = inverse_transpose * Vec4::new(normal[0], normal[1], normal[2], 0.0);
        let length = value.truncate().length();
        if !length.is_finite() || length <= f32::EPSILON {
            return Err("gltf_import_normal_transform_invalid".into());
        }
        let normalized = value.truncate() / length;
        Ok([
            quantize_normal(normalized.x)?,
            quantize_normal(normalized.y)?,
            quantize_normal(normalized.z)?,
        ])
    }

    fn quantize_normal(value: f32) -> Result<i32> {
        quantize_scaled(value, NORMAL_SCALE, "gltf_import_normal_overflow")
    }

    fn quantize_uv(value: f32) -> Result<i32> {
        quantize_scaled(value, UV_SCALE, "gltf_import_uv_overflow")
    }

    fn quantize_scaled(value: f32, scale: i32, overflow: &'static str) -> Result<i32> {
        if !value.is_finite() {
            return Err("gltf_import_number_invalid".into());
        }
        let scaled = f64::from(value) * f64::from(scale);
        let rounded = if scaled >= 0.0 {
            (scaled + 0.5).floor()
        } else {
            (scaled - 0.5).ceil()
        };
        if rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
            return Err(overflow.into());
        }
        Ok(rounded as i32)
    }

    fn quantize(value: f32) -> Result<i32> {
        quantize_scaled(value, GLTF_UNIT_SCALE, "gltf_import_coordinate_overflow")
    }

    fn material_id(context: &mut ImportContext, material: gltf::Material<'_>) -> Result<String> {
        let id = material
            .index()
            .map(|index| format!("gltf-material-{index}"))
            .unwrap_or_else(|| "gltf-material-default".into());
        if context.materials.contains_key(&id) {
            return Ok(id);
        }
        let factor = material.pbr_metallic_roughness().base_color_factor();
        let color = [
            quantize_color(factor[0]),
            quantize_color(factor[1]),
            quantize_color(factor[2]),
        ];
        let opacity = (u32::from(quantize_color(factor[3])) * 1000 / u32::from(u8::MAX)) as u16;
        context.materials.insert(
            id.clone(),
            Material3d {
                id: id.clone(),
                color,
                opacity,
                base_color_texture: None,
            },
        );
        Ok(id)
    }

    fn quantize_color(value: f32) -> u8 {
        if !value.is_finite() {
            return 0;
        }
        (f64::from(value.clamp(0.0, 1.0)) * f64::from(u8::MAX) + 0.5).floor() as u8
    }

    fn publish_scene(output: &Path, scene: &Scene3d) -> Result<()> {
        let mut bytes =
            to_vec_pretty(scene).map_err(|error| format!("gltf_import_serialize: {error}"))?;
        bytes.push(b'\n');
        let parent = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .map_err(|error| format!("gltf_import_output: {}: {error}", parent.display()))?;
        if output.exists() {
            return Err("gltf_import_output_exists".into());
        }
        let temporary = temporary_path(parent, output);
        fs::write(&temporary, &bytes)
            .map_err(|error| format!("gltf_import_write: {}: {error}", temporary.display()))?;
        if let Err(error) = fs::rename(&temporary, output) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("gltf_import_publish: {error}").into());
        }
        Ok(())
    }

    fn temporary_path(parent: &Path, output: &Path) -> PathBuf {
        let name = output
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("scene.json");
        parent.join(format!(".aetherion-gltf-{name}-{}", std::process::id()))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn quantization_is_symmetric_and_bounded() {
            assert_eq!(quantize(1.2345).unwrap(), 1235);
            assert_eq!(quantize(-1.2345).unwrap(), -1235);
            assert!(quantize(f32::INFINITY).is_err());
        }

        #[test]
        fn color_conversion_clamps_inputs() {
            assert_eq!(quantize_color(-1.0), 0);
            assert_eq!(quantize_color(0.5), 128);
            assert_eq!(quantize_color(2.0), 255);
        }
    }
}

#[cfg(feature = "gltf-import")]
pub fn import(input: &Path, output: &Path) -> Result<ImportSummary> {
    runtime::import(input, output)
}

#[cfg(not(feature = "gltf-import"))]
pub fn import(_input: &Path, _output: &Path) -> Result<ImportSummary> {
    Err("gltf_import_feature_disabled: compilez avec --features gltf-import".into())
}
