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
}

#[cfg(feature = "render-gpu")]
mod runtime {
    use std::mem::size_of;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use bytemuck::{Pod, Zeroable};
    use glam::Mat4;
    use wgpu::util::DeviceExt;
    use winit::dpi::PhysicalSize;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::EventLoop;
    use winit::window::{Window, WindowBuilder};

    use crate::Result;
    use crate::gpu3d::RunSummary;
    use crate::render3d::{self, Camera3d, Scene3d};

    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
    const MIN_DEPTH: f32 = 0.1;
    const MAX_DEPTH: f32 = 100_000.0;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    pub(super) struct Vertex {
        position: [f32; 3],
        color: [f32; 3],
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

    struct Renderer {
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        pipeline: wgpu::RenderPipeline,
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        index_count: u32,
        camera_buffer: wgpu::Buffer,
        camera_bind_group: wgpu::BindGroup,
        depth: DepthTexture,
        camera: Camera3d,
        scene_size: PhysicalSize<u32>,
        background: [f32; 3],
    }

    impl Renderer {
        async fn new(window: &Arc<Window>, scene: &Scene3d) -> Result<Self> {
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

            let (vertices, indices) = scene_vertices(scene)?;
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("aetherion-gpu-vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("aetherion-gpu-indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

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

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("aetherion-gpu-shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("aetherion-gpu-pipeline-layout"),
                bind_group_layouts: &[&camera_layout],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("aetherion-gpu-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::descriptor()],
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
                vertex_buffer,
                index_buffer,
                index_count: u32::try_from(indices.len()).map_err(|_| "render_gpu_index_quota")?,
                camera_buffer,
                camera_bind_group,
                depth,
                camera: scene.camera,
                scene_size,
                background: scene
                    .background
                    .map(|channel| f32::from(channel) / f32::from(u8::MAX)),
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
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.index_count, 0, 0..1);
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

    pub(super) fn scene_vertices(scene: &Scene3d) -> Result<(Vec<Vertex>, Vec<u32>)> {
        let triangles = render3d::expanded_triangles(scene)?;
        let mut vertices = Vec::with_capacity(triangles.len() * 3);
        let mut indices = Vec::with_capacity(triangles.len() * 3);
        for triangle in triangles {
            let base = u32::try_from(vertices.len()).map_err(|_| "render_gpu_vertex_quota")?;
            let color = triangle
                .color
                .map(|channel| f32::from(channel) / f32::from(u8::MAX));
            for vertex in triangle.vertices {
                vertices.push(Vertex {
                    position: [
                        vertex.x as f32,
                        vertex.y as f32,
                        -(vertex.z - scene.camera.z) as f32,
                    ],
                    color,
                });
            }
            indices.extend([base, base + 1, base + 2]);
        }
        Ok((vertices, indices))
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

    pub fn run(
        scene_path: &Path,
        assets_path: Option<&Path>,
        width: u32,
        height: u32,
        max_frames: Option<u64>,
    ) -> Result<RunSummary> {
        if width == 0 || height == 0 {
            return Err("render_gpu_dimensions_invalid".into());
        }
        let scene = if let Some(assets_path) = assets_path {
            let mut scene = render3d::load_unresolved(scene_path)?;
            render3d::resolve_assets(&mut scene, assets_path)?;
            scene
        } else {
            render3d::load(scene_path)?
        };
        render3d::validate(&scene)?;
        let triangle_count = render3d::expanded_triangles(&scene)?.len();
        if max_frames == Some(0) {
            return Ok(RunSummary {
                schema: super::REPORT_SCHEMA,
                status: "completed",
                width,
                height,
                frames_rendered: 0,
                triangles: triangle_count,
            });
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
        let mut renderer = pollster::block_on(Renderer::new(&window, &scene))?;
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
        Ok(RunSummary {
            schema: super::REPORT_SCHEMA,
            status: "completed",
            width,
            height,
            frames_rendered: frames_rendered.load(Ordering::Relaxed),
            triangles: triangle_count,
        })
    }

    const SHADER: &str = r#"
struct Camera {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_projection * vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
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
) -> Result<RunSummary> {
    runtime::run(scene, assets, width, height, max_frames)
}

#[cfg(not(feature = "render-gpu"))]
pub fn run(
    _scene: &Path,
    _assets: Option<&Path>,
    _width: u32,
    _height: u32,
    _max_frames: Option<u64>,
) -> Result<RunSummary> {
    Err("render_gpu_feature_disabled: compilez avec --features render-gpu".into())
}

#[cfg(all(test, feature = "render-gpu"))]
mod tests {
    use super::runtime::{camera_matrix, scene_vertices};
    use crate::render3d::{Camera3d, SCENE_SCHEMA, Scene3d};
    use winit::dpi::PhysicalSize;

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
        let (vertices, indices) = scene_vertices(&scene).unwrap();
        assert!(vertices.is_empty());
        assert!(indices.is_empty());
    }
}
