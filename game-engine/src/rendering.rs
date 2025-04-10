// rendering.rs
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::camera3d::Camera3d;
use crate::object3d::Object3d;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const SHADER: &str = r#"
struct VsOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) in_position: vec2<f32>, @location(1) in_color: vec3<f32>) -> VsOutput {
    return VsOutput(vec4<f32>(in_position, 0.0, 1.0), in_color);
}

@fragment
fn fs_main(@location(1) in_color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(in_color, 1.0);
}
"#;

pub struct Renderer {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera3d,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, _width: u32, _height: u32, camera: Camera3d) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Didn't find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device");

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(&device, &config);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader WGSL"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,
            camera,
        }
    }

    fn pixel_to_ndc(&self, x: f32, y: f32) -> [f32; 2] {
        let ndc_x = (x / self.size.width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / self.size.height as f32) * 2.0;
        [ndc_x, ndc_y]
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: new_size.width,
            height: new_size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &config);
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    /// This method loop over the objects, transforms their points, projects them with the camera,
    /// and generates the line segments.
    /// After that, it dynamically generates a vertex buffer and issues the draw command with wgpu.
    pub fn draw(&mut self, objects: &mut [Object3d]) -> Result<(), wgpu::SurfaceError> {
        let proj_matrix = self.camera.projection_matrix();
        let screen_center_x = self.size.width as f32 / 2.0;
        let screen_center_y = self.size.height as f32 / 2.0;

        let mut vertices: Vec<Vertex> = Vec::new();
        for object in objects.iter_mut() {
            if !object.render {
                continue;
            }
            let transformed = object.transform_points();
            let pts2d: Vec<(f32, f32)> = transformed
                .iter()
                .map(|p| {
                    if let Some((x, y)) = self.camera.project_point_with(p, &proj_matrix) {
                        (screen_center_x + x * 100.0, screen_center_y - y * 100.0)
                    } else {
                        (0.0, 0.0)
                    }
                })
                .collect();
            for edge in &object.edges {
                for window_edge in edge.windows(2) {
                    let a_idx = window_edge[0];
                    let b_idx = window_edge[1];
                    let (ax, ay) = pts2d[a_idx];
                    let (bx, by) = pts2d[b_idx];
                    let pos_a = self.pixel_to_ndc(ax, ay);
                    let pos_b = self.pixel_to_ndc(bx, by);
                    vertices.push(Vertex {
                        position: pos_a,
                        color: [1.0, 1.0, 1.0],
                    });
                    vertices.push(Vertex {
                        position: pos_b,
                        color: [1.0, 1.0, 1.0],
                    });
                }
            }
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Dynamic Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..(vertices.len() as u32), 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}
