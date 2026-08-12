use serde::Serialize;
use std::path::Path;

use crate::Result;

pub const REPORT_SCHEMA: &str = "aetherion.gpu-demo/v1";

#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub width: u32,
    pub height: u32,
    pub frames_rendered: u64,
    pub triangles: usize,
    pub objects: usize,
    pub culled_objects: usize,
    pub draw_calls: usize,
}

pub const BENCHMARK_REPORT_SCHEMA: &str = "aetherion.gpu-benchmark/v1";

#[derive(Clone, Debug, Serialize)]
pub struct BenchmarkSummary {
    pub schema: &'static str,
    pub status: &'static str,
    pub width: u32,
    pub height: u32,
    pub frames_rendered: u64,
    pub triangles: usize,
    pub objects: usize,
    pub culled_objects: usize,
    pub draw_calls: usize,
    pub elapsed_ms: u64,
    pub fps_milli: u64,
    pub adapter: String,
}

#[cfg(feature = "render-gpu")]
mod runtime {
    use std::collections::BTreeMap;
    use std::mem::size_of;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    use bytemuck::{Pod, Zeroable};
    use glam::{Mat4, Vec3};
    use image::GenericImageView;
    use wgpu::util::DeviceExt;
    use winit::dpi::PhysicalSize;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::EventLoop;
    use winit::window::{Window, WindowBuilder};

    use crate::Result;
    use crate::gpu3d::{BenchmarkSummary, RunSummary};
    use crate::render3d::{self, Camera3d, NORMAL_SCALE, Scene3d, Transform3d, UV_SCALE};

    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
    const MIN_DEPTH: f32 = 0.1;
    const MAX_DEPTH: f32 = 100_000.0;
    const MAX_TEXTURE_DIMENSION: u32 = 4096;
    const MAX_TEXTURE_PIXELS: u64 = 16_777_216;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    pub(super) struct Vertex {
        position: [f32; 3],
        color: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
    }

    impl Vertex {
        fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 6]>() as wgpu::BufferAddress,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 9]>() as wgpu::BufferAddress,
                        shader_location: 3,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    struct Instance {
        model: [[f32; 4]; 4],
        normal_model: [[f32; 4]; 4],
    }

    impl Instance {
        fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 4,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        shader_location: 5,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                        shader_location: 6,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                        shader_location: 7,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                        shader_location: 8,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 20]>() as wgpu::BufferAddress,
                        shader_location: 9,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 24]>() as wgpu::BufferAddress,
                        shader_location: 10,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: size_of::<[f32; 28]>() as wgpu::BufferAddress,
                        shader_location: 11,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    struct CameraUniform {
        view_projection: [[f32; 4]; 4],
    }

    struct DepthTexture {
        view: wgpu::TextureView,
    }

    impl DepthTexture {
        fn create(device: &wgpu::Device, size: PhysicalSize<u32>) -> Self {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("aetherion-gpu-depth"),
                size: wgpu::Extent3d {
                    width: size.width.max(1),
                    height: size.height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            Self {
                view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            }
        }
    }

    struct GpuBatch {
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        instance_buffer: wgpu::Buffer,
        index_count: u32,
        instance_count: u32,
        texture_index: usize,
    }

    struct GpuTexture {
        _texture: wgpu::Texture,
        _view: wgpu::TextureView,
        _sampler: wgpu::Sampler,
        bind_group: wgpu::BindGroup,
    }

    struct Renderer {
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        pipeline: wgpu::RenderPipeline,
        batches: Vec<GpuBatch>,
        textures: Vec<GpuTexture>,
        camera_buffer: wgpu::Buffer,
        camera_bind_group: wgpu::BindGroup,
        depth: DepthTexture,
        camera: Camera3d,
        scene_size: PhysicalSize<u32>,
        background: [f32; 3],
        adapter: String,
        objects: usize,
        culled_objects: usize,
        triangles: usize,
    }

    impl Renderer {
        async fn new(
            window: &Arc<Window>,
            scene: &Scene3d,
            texture_bytes: &BTreeMap<String, Vec<u8>>,
        ) -> Result<Self> {
            let scene_size = window.inner_size();
            let instance = wgpu::Instance::default();
            let surface = instance
                .create_surface(window.clone())
                .map_err(|error| format!("render_gpu_surface: {error}"))?;
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok_or("render_gpu_adapter: aucun adaptateur compatible")?;
            let adapter_name = adapter.get_info().name;
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("aetherion-gpu-device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .map_err(|error| format!("render_gpu_device: {error}"))?;

            let capabilities = surface.get_capabilities(&adapter);
            let format = capabilities
                .formats
                .iter()
                .copied()
                .find(wgpu::TextureFormat::is_srgb)
                .or_else(|| capabilities.formats.first().copied())
                .ok_or("render_gpu_surface_format_missing")?;
            let present_mode = capabilities
                .present_modes
                .iter()
                .copied()
                .find(|mode| *mode == wgpu::PresentMode::Fifo)
                .or_else(|| capabilities.present_modes.first().copied())
                .ok_or("render_gpu_present_mode_missing")?;
            let alpha_mode = capabilities
                .alpha_modes
                .first()
                .copied()
                .ok_or("render_gpu_alpha_mode_missing")?;
            let config = surface_configuration(format, present_mode, alpha_mode, scene_size);
            surface.configure(&device, &config);

            let prepared = scene_batches(scene, texture_bytes, scene_size)?;
            let batch_data = prepared.batches;

            let camera_uniform = CameraUniform {
                view_projection: camera_matrix(scene.camera, scene_size).to_cols_array_2d(),
            };
            let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("aetherion-gpu-camera"),
                contents: bytemuck::bytes_of(&camera_uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("aetherion-gpu-camera-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(size_of::<CameraUniform>() as u64),
                    },
                    count: None,
                }],
            });
            let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("aetherion-gpu-camera-bind-group"),
                layout: &camera_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

            let texture_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("aetherion-gpu-texture-layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
            let (textures, texture_indices) =
                create_gpu_textures(&device, &queue, &texture_layout, &batch_data, texture_bytes)?;
            let batches = batch_data
                .into_iter()
                .map(|batch| {
                    let index_count =
                        u32::try_from(batch.indices.len()).map_err(|_| "render_gpu_index_quota")?;
                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("aetherion-gpu-vertices"),
                            contents: bytemuck::cast_slice(&batch.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    let index_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("aetherion-gpu-indices"),
                            contents: bytemuck::cast_slice(&batch.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                    let instance_count = u32::try_from(batch.instances.len())
                        .map_err(|_| "render_gpu_instance_quota")?;
                    let instance_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("aetherion-gpu-instances"),
                            contents: bytemuck::cast_slice(&batch.instances),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    Ok(GpuBatch {
                        vertex_buffer,
                        index_buffer,
                        instance_buffer,
                        index_count,
                        instance_count,
                        texture_index: texture_indices[&batch.texture],
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("aetherion-gpu-shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("aetherion-gpu-pipeline-layout"),
                bind_group_layouts: &[&camera_layout, &texture_layout],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("aetherion-gpu-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::descriptor(), Instance::descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

            let depth = DepthTexture::create(&device, scene_size);
            Ok(Self {
                surface,
                device,
                queue,
                config,
                pipeline,
                batches,
                textures,
                camera_buffer,
                camera_bind_group,
                depth,
                camera: scene.camera,
                scene_size,
                background: scene
                    .background
                    .map(|channel| f32::from(channel) / f32::from(u8::MAX)),
                adapter: adapter_name,
                objects: prepared.objects,
                culled_objects: prepared.culled_objects,
                triangles: prepared.triangles,
            })
        }

        fn resize(&mut self, size: PhysicalSize<u32>) {
            if size.width == 0 || size.height == 0 {
                return;
            }
            self.scene_size = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth = DepthTexture::create(&self.device, size);
            let camera = CameraUniform {
                view_projection: camera_matrix(self.camera, size).to_cols_array_2d(),
            };
            self.queue
                .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera));
        }

        fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
            let frame = self.surface.get_current_texture()?;
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("aetherion-gpu-frame"),
                });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("aetherion-gpu-render-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: f64::from(self.background[0]),
                                g: f64::from(self.background[1]),
                                b: f64::from(self.background[2]),
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.camera_bind_group, &[]);
                for batch in &self.batches {
                    pass.set_bind_group(1, &self.textures[batch.texture_index].bind_group, &[]);
                    pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
                    pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..batch.index_count, 0, 0..batch.instance_count);
                }
            }
            self.queue.submit([encoder.finish()]);
            frame.present();
            Ok(())
        }
    }

    fn surface_configuration(
        format: wgpu::TextureFormat,
        present_mode: wgpu::PresentMode,
        alpha_mode: wgpu::CompositeAlphaMode,
        size: PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    pub(super) struct BatchData {
        pub(super) texture: Option<String>,
        pub(super) vertices: Vec<Vertex>,
        pub(super) indices: Vec<u32>,
        instances: Vec<Instance>,
    }

    impl BatchData {
        #[cfg(test)]
        pub(super) fn instance_count(&self) -> usize {
            self.instances.len()
        }
    }

    type TextureIndices = BTreeMap<Option<String>, usize>;
    type GpuTextureSet = (Vec<GpuTexture>, TextureIndices);

    pub(super) struct PreparedScene {
        pub(super) batches: Vec<BatchData>,
        pub(super) objects: usize,
        pub(super) culled_objects: usize,
        pub(super) triangles: usize,
    }

    pub(super) fn scene_batches(
        scene: &Scene3d,
        texture_bytes: &BTreeMap<String, Vec<u8>>,
        viewport: PhysicalSize<u32>,
    ) -> Result<PreparedScene> {
        let mut meshes = BTreeMap::new();
        let mut mesh_bounds = BTreeMap::new();
        for mesh in &scene.meshes {
            meshes.insert(mesh.id.as_str(), mesh);
            mesh_bounds.insert(mesh.id.as_str(), render3d::mesh_bounds(mesh)?);
        }
        let mut materials = BTreeMap::new();
        for material in &scene.materials {
            materials.insert(material.id.as_str(), material);
        }
        for object in &scene.objects {
            if !meshes.contains_key(object.mesh.as_str()) {
                return Err("render_gpu_mesh_reference_missing".into());
            }
            if !materials.contains_key(object.material.as_str()) {
                return Err("render_gpu_material_reference_missing".into());
            }
            if object
                .transform
                .rotation
                .iter()
                .any(|angle| angle.rem_euclid(90_000) != 0)
            {
                return Err(
                    "scene3d_rotation_invalid: quarts de tour en millidegres uniquement".into(),
                );
            }
        }
        let visible = visible_objects(scene, &mesh_bounds, viewport)?;
        let mut batches = BTreeMap::<(String, String, Option<String>), BatchData>::new();
        let mut rendered_triangles: usize = 0;

        for object in &scene.objects {
            if !visible.contains(&object.id) {
                continue;
            }
            let mesh = meshes
                .get(object.mesh.as_str())
                .ok_or("render_gpu_mesh_reference_missing")?;
            let material = materials
                .get(object.material.as_str())
                .ok_or("render_gpu_material_reference_missing")?;
            let texture = material.base_color_texture.clone();
            if let Some(id) = texture.as_ref()
                && !texture_bytes.contains_key(id)
            {
                return Err(format!("render_gpu_texture_missing: {id}").into());
            }
            let key = (mesh.id.clone(), material.id.clone(), texture.clone());
            if !batches.contains_key(&key) {
                let (vertices, indices) = mesh_geometry(mesh, material, scene.background)?;
                batches.insert(
                    key.clone(),
                    BatchData {
                        texture,
                        vertices,
                        indices,
                        instances: Vec::new(),
                    },
                );
            }
            let batch = batches.get_mut(&key).ok_or("render_gpu_batch_missing")?;
            batch
                .instances
                .push(instance_for(object.transform, scene.camera.z));
            rendered_triangles = rendered_triangles
                .checked_add(mesh.triangles.len())
                .ok_or("render_gpu_triangle_quota")?;
        }

        if !scene.triangles.is_empty() {
            let key = ("__legacy".into(), "__legacy".into(), None);
            let mut batch = BatchData {
                texture: None,
                vertices: Vec::with_capacity(scene.triangles.len() * 3),
                indices: Vec::with_capacity(scene.triangles.len() * 3),
                instances: vec![instance_for(Transform3d::default(), scene.camera.z)],
            };
            for triangle in &scene.triangles {
                let base =
                    u32::try_from(batch.vertices.len()).map_err(|_| "render_gpu_vertex_quota")?;
                let positions = triangle.vertices.map(base_position);
                let normal = flat_normal(positions);
                let color = triangle
                    .color
                    .map(|channel| f32::from(channel) / f32::from(u8::MAX));
                batch
                    .vertices
                    .extend(positions.into_iter().map(|position| Vertex {
                        position,
                        color,
                        normal,
                        uv: [0.0, 0.0],
                    }));
                batch.indices.extend([base, base + 1, base + 2]);
            }
            rendered_triangles = rendered_triangles
                .checked_add(scene.triangles.len())
                .ok_or("render_gpu_triangle_quota")?;
            batches.insert(key, batch);
        }

        let objects = scene.objects.len();
        Ok(PreparedScene {
            batches: batches.into_values().collect(),
            objects,
            culled_objects: objects.saturating_sub(visible.len()),
            triangles: rendered_triangles,
        })
    }

    fn mesh_geometry(
        mesh: &render3d::Mesh3d,
        material: &render3d::Material3d,
        background: [u8; 3],
    ) -> Result<(Vec<Vertex>, Vec<u32>)> {
        let color = material_color(material, background);
        if mesh.normals.is_empty() {
            let mut vertices = Vec::with_capacity(mesh.triangles.len() * 3);
            let mut indices = Vec::with_capacity(mesh.triangles.len() * 3);
            for face in &mesh.triangles {
                let base = u32::try_from(vertices.len()).map_err(|_| "render_gpu_vertex_quota")?;
                let mut positions = [[0.0; 3]; 3];
                for (slot, index) in face.iter().enumerate() {
                    positions[slot] = mesh
                        .vertices
                        .get(*index as usize)
                        .copied()
                        .map(base_position)
                        .ok_or("scene3d_mesh_index_invalid")?;
                }
                let normal = flat_normal(positions);
                vertices.extend(positions.into_iter().map(|position| Vertex {
                    position,
                    color,
                    normal,
                    uv: [0.0, 0.0],
                }));
                indices.extend([base, base + 1, base + 2]);
            }
            return Ok((vertices, indices));
        }

        let vertices = mesh
            .vertices
            .iter()
            .enumerate()
            .map(|(index, vertex)| Vertex {
                position: base_position(*vertex),
                color,
                normal: source_normal(mesh.normals[index]),
                uv: mesh
                    .uvs
                    .get(index)
                    .map(|uv| {
                        [
                            uv[0] as f32 / UV_SCALE as f32,
                            uv[1] as f32 / UV_SCALE as f32,
                        ]
                    })
                    .unwrap_or([0.0, 0.0]),
            })
            .collect();
        let indices = mesh
            .triangles
            .iter()
            .flat_map(|face| face.iter().copied())
            .collect();
        Ok((vertices, indices))
    }

    fn material_color(material: &render3d::Material3d, background: [u8; 3]) -> [f32; 3] {
        [0, 1, 2].map(|channel| {
            let value = (u32::from(material.color[channel]) * u32::from(material.opacity)
                + u32::from(background[channel]) * (1000 - u32::from(material.opacity)))
                / 1000;
            f32::from(value as u8) / f32::from(u8::MAX)
        })
    }

    fn base_position(vertex: render3d::Vertex3) -> [f32; 3] {
        [vertex.x as f32, vertex.y as f32, -vertex.z as f32]
    }

    fn source_normal(normal: [i32; 3]) -> [f32; 3] {
        let value = Vec3::new(
            -(normal[0] as f32 / NORMAL_SCALE as f32),
            -(normal[1] as f32 / NORMAL_SCALE as f32),
            normal[2] as f32 / NORMAL_SCALE as f32,
        );
        if value.length_squared() <= f32::EPSILON {
            Vec3::Z.to_array()
        } else {
            value.normalize().to_array()
        }
    }

    fn instance_for(transform: Transform3d, camera_z: i32) -> Instance {
        let scale = Mat4::from_scale(Vec3::new(
            transform.scale[0] as f32 / 1000.0,
            transform.scale[1] as f32 / 1000.0,
            transform.scale[2] as f32 / 1000.0,
        ));
        let rotate_x = Mat4::from_rotation_x((transform.rotation[0] as f32).to_radians());
        let rotate_y = Mat4::from_rotation_y((transform.rotation[1] as f32).to_radians());
        let rotate_z = Mat4::from_rotation_z((transform.rotation[2] as f32).to_radians());
        let object = Mat4::from_translation(Vec3::new(
            transform.translation[0] as f32,
            transform.translation[1] as f32,
            transform.translation[2] as f32,
        )) * rotate_z
            * rotate_y
            * rotate_x
            * scale;
        let reflection = Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0));
        let model = Mat4::from_translation(Vec3::new(0.0, 0.0, camera_z as f32))
            * reflection
            * object
            * reflection;
        Instance {
            model: model.to_cols_array_2d(),
            normal_model: model.inverse().transpose().to_cols_array_2d(),
        }
    }

    fn visible_objects(
        scene: &Scene3d,
        mesh_bounds: &BTreeMap<&str, Option<render3d::MeshBounds3d>>,
        viewport: PhysicalSize<u32>,
    ) -> Result<std::collections::BTreeSet<String>> {
        let mut bounds = BTreeMap::<String, ([f32; 3], [f32; 3])>::new();
        for object in &scene.objects {
            let local = mesh_bounds
                .get(object.mesh.as_str())
                .ok_or("render_gpu_mesh_reference_missing")?;
            let Some(local) = *local else {
                continue;
            };
            let transformed = render3d::transform_mesh_bounds(local, object.transform)?;
            let entry = bounds
                .entry(object.id.clone())
                .or_insert(([f32::MAX; 3], [f32::MIN; 3]));
            let min = [
                transformed.min.x as f32,
                transformed.min.y as f32,
                transformed.min.z as f32,
            ];
            let max = [
                transformed.max.x as f32,
                transformed.max.y as f32,
                transformed.max.z as f32,
            ];
            for axis in 0..3 {
                entry.0[axis] = entry.0[axis].min(min[axis]);
                entry.1[axis] = entry.1[axis].max(max[axis]);
            }
        }
        let camera = scene.camera;
        let scale = camera.pixels_per_unit.max(1) as f32;
        let half_width = viewport.width.max(1) as f32 / scale / 2.0;
        let half_height = viewport.height.max(1) as f32 / scale / 2.0;
        let min_x = camera.x as f32 - half_width;
        let max_x = camera.x as f32 + half_width;
        let min_y = camera.y as f32 - half_height;
        let max_y = camera.y as f32 + half_height;
        let min_z = camera.z as f32 + MIN_DEPTH;
        let max_z = camera.z as f32 + MAX_DEPTH;
        Ok(bounds
            .into_iter()
            .filter_map(|(id, (min, max))| {
                let intersects = max[0] >= min_x
                    && min[0] <= max_x
                    && max[1] >= min_y
                    && min[1] <= max_y
                    && max[2] >= min_z
                    && min[2] <= max_z;
                intersects.then_some(id)
            })
            .collect())
    }

    fn create_gpu_textures(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        batches: &[BatchData],
        texture_bytes: &BTreeMap<String, Vec<u8>>,
    ) -> Result<GpuTextureSet> {
        let mut keys = BTreeMap::<Option<String>, ()>::new();
        for batch in batches {
            keys.insert(batch.texture.clone(), ());
        }
        let mut textures = Vec::with_capacity(keys.len());
        let mut indices = BTreeMap::new();
        for (key, ()) in keys {
            let (width, height, pixels) = key
                .as_ref()
                .map(|id| {
                    texture_bytes
                        .get(id)
                        .ok_or_else(|| format!("render_gpu_texture_missing: {id}").into())
                        .and_then(|bytes| decode_texture(id, bytes))
                })
                .unwrap_or_else(|| Ok((1, 1, vec![255, 255, 255, 255])))?;
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("aetherion-gpu-base-color"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("aetherion-gpu-base-color-sampler"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..wgpu::SamplerDescriptor::default()
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("aetherion-gpu-base-color-bind-group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let index = textures.len();
            indices.insert(key, index);
            textures.push(GpuTexture {
                _texture: texture,
                _view: view,
                _sampler: sampler,
                bind_group,
            });
        }
        Ok((textures, indices))
    }

    fn decode_texture(id: &str, bytes: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
        let decoded = image::load_from_memory(bytes)
            .map_err(|error| format!("render_gpu_texture_decode: {id}: {error}"))?;
        let (width, height) = decoded.dimensions();
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or("render_gpu_texture_quota")?;
        if width == 0
            || height == 0
            || width > MAX_TEXTURE_DIMENSION
            || height > MAX_TEXTURE_DIMENSION
            || pixels > MAX_TEXTURE_PIXELS
        {
            return Err(format!("render_gpu_texture_quota: {id}").into());
        }
        Ok((width, height, decoded.to_rgba8().into_raw()))
    }

    fn flat_normal(positions: [[f32; 3]; 3]) -> [f32; 3] {
        let first = Vec3::from_array(positions[0]);
        let second = Vec3::from_array(positions[1]);
        let third = Vec3::from_array(positions[2]);
        let cross = (second - first).cross(third - first);
        if cross.length_squared() <= f32::EPSILON {
            Vec3::Z.to_array()
        } else {
            cross.normalize().to_array()
        }
    }

    pub(super) fn camera_matrix(camera: Camera3d, size: PhysicalSize<u32>) -> Mat4 {
        let pixels_per_unit = camera.pixels_per_unit.max(1) as f32;
        let half_width = size.width.max(1) as f32 / pixels_per_unit / 2.0;
        let half_height = size.height.max(1) as f32 / pixels_per_unit / 2.0;
        Mat4::orthographic_rh_gl(
            camera.x as f32 - half_width,
            camera.x as f32 + half_width,
            camera.y as f32 - half_height,
            camera.y as f32 + half_height,
            MIN_DEPTH,
            MAX_DEPTH,
        )
    }

    fn load_scene(
        scene_path: &Path,
        assets_path: Option<&Path>,
        cache_root: Option<&Path>,
    ) -> Result<(Scene3d, BTreeMap<String, Vec<u8>>)> {
        if let Some(assets_path) = assets_path {
            let mut scene = render3d::load_unresolved(scene_path)?;
            let textures =
                render3d::resolve_assets_with_textures_cached(&mut scene, assets_path, cache_root)?;
            Ok((scene, textures))
        } else {
            Ok((render3d::load(scene_path)?, BTreeMap::new()))
        }
    }

    pub fn run(
        scene_path: &Path,
        assets_path: Option<&Path>,
        width: u32,
        height: u32,
        max_frames: Option<u64>,
        cache_root: Option<&Path>,
    ) -> Result<RunSummary> {
        run_internal(
            scene_path,
            assets_path,
            width,
            height,
            max_frames,
            cache_root,
        )
        .map(|(summary, _, _)| summary)
    }

    pub fn benchmark(
        scene_path: &Path,
        assets_path: Option<&Path>,
        width: u32,
        height: u32,
        frames: u64,
        cache_root: Option<&Path>,
    ) -> Result<BenchmarkSummary> {
        if frames == 0 {
            return Err("render_gpu_benchmark_frames_invalid: minimum 1".into());
        }
        let (summary, adapter, elapsed) = run_internal(
            scene_path,
            assets_path,
            width,
            height,
            Some(frames),
            cache_root,
        )?;
        let elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
        let fps_milli = if elapsed_ms == 0 {
            0
        } else {
            summary
                .frames_rendered
                .saturating_mul(1000)
                .saturating_mul(1000)
                .checked_div(elapsed_ms)
                .unwrap_or(0)
        };
        Ok(BenchmarkSummary {
            schema: super::BENCHMARK_REPORT_SCHEMA,
            status: "completed",
            width: summary.width,
            height: summary.height,
            frames_rendered: summary.frames_rendered,
            triangles: summary.triangles,
            objects: summary.objects,
            culled_objects: summary.culled_objects,
            draw_calls: summary.draw_calls,
            elapsed_ms,
            fps_milli,
            adapter,
        })
    }

    fn run_internal(
        scene_path: &Path,
        assets_path: Option<&Path>,
        width: u32,
        height: u32,
        max_frames: Option<u64>,
        cache_root: Option<&Path>,
    ) -> Result<(RunSummary, String, std::time::Duration)> {
        if width == 0 || height == 0 {
            return Err("render_gpu_dimensions_invalid".into());
        }
        let (scene, texture_bytes) = load_scene(scene_path, assets_path, cache_root)?;
        let prepared = scene_batches(&scene, &texture_bytes, PhysicalSize::new(width, height))?;
        if max_frames == Some(0) {
            return Ok((
                RunSummary {
                    schema: super::REPORT_SCHEMA,
                    status: "completed",
                    width,
                    height,
                    frames_rendered: 0,
                    triangles: prepared.triangles,
                    objects: prepared.objects,
                    culled_objects: prepared.culled_objects,
                    draw_calls: prepared.batches.len(),
                },
                "not-started".into(),
                std::time::Duration::ZERO,
            ));
        }
        if max_frames.is_some_and(|frames| frames > 1_000_000) {
            return Err("render_gpu_frames_quota: plafond 1000000".into());
        }
        let event_loop =
            EventLoop::new().map_err(|error| format!("render_gpu_event_loop: {error}"))?;
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Aetherion GPU demo")
                .with_inner_size(PhysicalSize::new(width, height))
                .build(&event_loop)
                .map_err(|error| format!("render_gpu_window: {error}"))?,
        );
        let mut renderer = pollster::block_on(Renderer::new(&window, &scene, &texture_bytes))?;
        let adapter = renderer.adapter.clone();
        let objects = renderer.objects;
        let culled_objects = renderer.culled_objects;
        let triangles = renderer.triangles;
        let draw_calls = renderer.batches.len();
        let started = Instant::now();
        let frames_rendered = Arc::new(AtomicU64::new(0));
        let frames_for_loop = Arc::clone(&frames_rendered);
        event_loop
            .run(move |event, target| match event {
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(size) => renderer.resize(size),
                        WindowEvent::RedrawRequested => match renderer.render() {
                            Ok(()) => {
                                let rendered = frames_for_loop.fetch_add(1, Ordering::Relaxed) + 1;
                                if max_frames.is_some_and(|limit| rendered >= limit) {
                                    target.exit();
                                }
                            }
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                renderer.resize(renderer.scene_size)
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(wgpu::SurfaceError::Timeout) => {}
                        },
                        _ => {}
                    }
                }
                Event::AboutToWait => window.request_redraw(),
                _ => {}
            })
            .map_err(|error| format!("render_gpu_event_loop: {error}"))?;
        Ok((
            RunSummary {
                schema: super::REPORT_SCHEMA,
                status: "completed",
                width,
                height,
                frames_rendered: frames_rendered.load(Ordering::Relaxed),
                triangles,
                objects,
                culled_objects,
                draw_calls,
            },
            adapter,
            started.elapsed(),
        ))
    }

    const SHADER: &str = r#"
struct Camera {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var base_color_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) normal_model_0: vec4<f32>,
    @location(9) normal_model_1: vec4<f32>,
    @location(10) normal_model_2: vec4<f32>,
    @location(11) normal_model_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let model = mat4x4<f32>(
        input.model_0,
        input.model_1,
        input.model_2,
        input.model_3,
    );
    let normal_model = mat4x4<f32>(
        input.normal_model_0,
        input.normal_model_1,
        input.normal_model_2,
        input.normal_model_3,
    );
    let world_position = model * vec4<f32>(input.position, 1.0);
    output.position = camera.view_projection * world_position;
    output.color = input.color;
    output.normal = normalize((normal_model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_direction = normalize(vec3<f32>(0.45, 0.65, 1.0));
    let diffuse = max(dot(normalize(input.normal), light_direction), 0.0);
    let intensity = 0.25 + 0.75 * diffuse;
    let albedo = textureSample(base_color_texture, base_color_sampler, input.uv).rgb;
    return vec4<f32>(input.color * albedo * intensity, 1.0);
}
"#;
}

#[cfg(feature = "render-gpu")]
pub fn run(
    scene: &Path,
    assets: Option<&Path>,
    width: u32,
    height: u32,
    max_frames: Option<u64>,
    cache_root: Option<&Path>,
) -> Result<RunSummary> {
    runtime::run(scene, assets, width, height, max_frames, cache_root)
}

#[cfg(feature = "render-gpu")]
pub fn benchmark(
    scene: &Path,
    assets: Option<&Path>,
    width: u32,
    height: u32,
    frames: u64,
    cache_root: Option<&Path>,
) -> Result<BenchmarkSummary> {
    runtime::benchmark(scene, assets, width, height, frames, cache_root)
}

#[cfg(not(feature = "render-gpu"))]
pub fn run(
    _scene: &Path,
    _assets: Option<&Path>,
    _width: u32,
    _height: u32,
    _max_frames: Option<u64>,
    _cache_root: Option<&Path>,
) -> Result<RunSummary> {
    Err("render_gpu_feature_disabled: compilez avec --features render-gpu".into())
}

#[cfg(not(feature = "render-gpu"))]
pub fn benchmark(
    _scene: &Path,
    _assets: Option<&Path>,
    _width: u32,
    _height: u32,
    _frames: u64,
    _cache_root: Option<&Path>,
) -> Result<BenchmarkSummary> {
    Err("render_gpu_feature_disabled: compilez avec --features render-gpu".into())
}

#[cfg(all(test, feature = "render-gpu"))]
mod tests {
    use super::runtime::{camera_matrix, scene_batches};
    use crate::render3d::{
        Camera3d, Material3d, Mesh3d, Object3d, SCENE_SCHEMA, Scene3d, Transform3d, Vertex3,
    };
    use winit::dpi::PhysicalSize;

    fn textured_scene() -> Scene3d {
        Scene3d {
            schema: SCENE_SCHEMA.into(),
            camera: Camera3d::default(),
            background: [0, 0, 0],
            triangles: Vec::new(),
            meshes: vec![Mesh3d {
                id: "mesh".into(),
                vertices: vec![
                    Vertex3 { x: 0, y: 0, z: 1 },
                    Vertex3 { x: 1, y: 0, z: 1 },
                    Vertex3 { x: 0, y: 1, z: 1 },
                ],
                triangles: vec![[0, 1, 2]],
                normals: Vec::new(),
                uvs: vec![
                    [0, 0],
                    [crate::render3d::UV_SCALE, 0],
                    [0, crate::render3d::UV_SCALE],
                ],
            }],
            materials: vec![Material3d {
                id: "material".into(),
                color: [255, 255, 255],
                opacity: 1000,
                base_color_texture: Some("albedo".into()),
            }],
            objects: vec![Object3d {
                id: "object".into(),
                mesh: "mesh".into(),
                material: "material".into(),
                transform: Transform3d::default(),
            }],
            animations: Vec::new(),
        }
    }

    #[test]
    fn camera_matrix_is_finite_and_scene_conversion_is_bounded() {
        let camera = Camera3d {
            pixels_per_unit: 16,
            ..Camera3d::default()
        };
        let matrix = camera_matrix(camera, PhysicalSize::new(1280, 720));
        assert!(matrix.to_cols_array().iter().all(|value| value.is_finite()));
        let scene = Scene3d {
            schema: SCENE_SCHEMA.into(),
            camera,
            background: [0, 0, 0],
            triangles: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            objects: Vec::new(),
            animations: Vec::new(),
        };
        let prepared = scene_batches(
            &scene,
            &std::collections::BTreeMap::new(),
            PhysicalSize::new(1280, 720),
        )
        .unwrap();
        assert!(prepared.batches.is_empty());
    }

    #[test]
    fn textured_batches_require_and_retain_external_texture_assets() {
        let scene = textured_scene();
        assert!(
            scene_batches(
                &scene,
                &std::collections::BTreeMap::new(),
                PhysicalSize::new(1280, 720),
            )
            .is_err()
        );
        let mut textures = std::collections::BTreeMap::new();
        textures.insert("albedo".into(), vec![0; 4]);
        let prepared = scene_batches(&scene, &textures, PhysicalSize::new(1280, 720)).unwrap();
        assert_eq!(prepared.batches.len(), 1);
        assert_eq!(prepared.batches[0].texture.as_deref(), Some("albedo"));
        assert_eq!(prepared.objects, 1);
        assert_eq!(prepared.culled_objects, 0);
        assert_eq!(prepared.batches[0].vertices.len(), 3);
        assert_eq!(prepared.batches[0].indices.len(), 3);
        assert_eq!(prepared.batches[0].instance_count(), 1);

        let mut instanced = textured_scene();
        let mut second = instanced.objects[0].clone();
        second.id = "object-2".into();
        second.transform.translation = [2, 0, 0];
        instanced.objects.push(second);
        let prepared = scene_batches(&instanced, &textures, PhysicalSize::new(1280, 720)).unwrap();
        assert_eq!(prepared.batches.len(), 1);
        assert_eq!(prepared.batches[0].vertices.len(), 3);
        assert_eq!(prepared.batches[0].indices.len(), 3);
        assert_eq!(prepared.batches[0].instance_count(), 2);
        assert_eq!(prepared.triangles, 2);

        let mut offscreen = scene;
        offscreen.objects[0].transform.translation = [10_000, 0, 0];
        let prepared = scene_batches(&offscreen, &textures, PhysicalSize::new(1280, 720)).unwrap();
        assert_eq!(prepared.culled_objects, 1);
        assert_eq!(prepared.triangles, 0);
    }
}
