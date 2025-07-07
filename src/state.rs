use std::{sync::Arc};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize, 
    event_loop::ActiveEventLoop, 
    keyboard::{KeyCode}, 
    window::Window
};


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

/* Implments des, des is a funtion that generates the  */
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Test {
    ok : [f32; 4]
}

/*   Implements the state of the entire program, handling rendering
 * to the screen, keyboard inputs and other stuff. (Is this really
 * a good way to do this?)
 */
pub struct State {
    surface : wgpu::Surface<'static>,
    device  : wgpu::Device,
    queue   : wgpu::Queue,
    config  : wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    window  : Arc<Window>,
    vertex_buffer : wgpu::Buffer,
    num_vertices : u32,
    index_buffer : wgpu::Buffer,
    num_indices : u32,
    oklahoma : Test,
    test_buffer : wgpu::Buffer,
    test_bind_group : wgpu::BindGroup
}

impl Test {
    fn new() -> Self {
        Self { ok: [0.0, 0.0, 0.0, 0.0] }
    }
}

impl State {
    /// This function initializes the program an its window
    pub async fn new(window : Arc<Window>) -> anyhow::Result<Self> {
        let size= window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends : wgpu::Backends::all(), ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface = instance.create_surface(Arc::clone(&window))
            .unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference : wgpu::PowerPreference::default(),
            compatible_surface : Some(&surface),
            force_fallback_adapter : false
        };

        let adapter = instance.request_adapter(&adapter_descriptor)
            .await
            .unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features : wgpu::Features::empty(),
            required_limits : wgpu::Limits::default(),
            memory_hints : wgpu::MemoryHints::default(),
            trace : wgpu::Trace::Off,
            label : None,
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor)
            .await.unwrap();

        let surface_capabilites = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilites
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilites.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format : surface_format,
            width : size.width,
            height : size.height,
            present_mode : surface_capabilites.present_modes[0],
            alpha_mode : surface_capabilites.alpha_modes[0],
            view_formats : vec![],
            desired_maximum_frame_latency : 2
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut oklahoma = Test::new();

        let test_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:Some("Dogshit"),
                contents: bytemuck::cast_slice(&[oklahoma]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let test_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("shit layout")
            }
        );

        let test_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout : &test_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: test_buffer.as_entire_binding().into()
                    }
                ],
                label: Some("bi"),
            }
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&test_bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), 
                buffers: &[Vertex::desc()], 
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false, 
            },
            multiview: None, 
            cache: None, 
            }
        );

        const SQUARE: &[Vertex] = &[
            Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
            Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
            Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
            Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
            Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(SQUARE),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let num_vertices = SQUARE.len() as u32;

        const INDICES: &[u16] = &[
            0, 1, 4,
            1, 2, 4,
            2, 3, 4,
        ];

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = INDICES.len() as u32;


        Ok(Self {
            window,
            surface,
            device,
            queue,
            render_pipeline,
            config,
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
            oklahoma,
            test_buffer,
            test_bind_group
        })
    }

    pub fn resize(&mut self, new_size : PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        let output = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let view  = output.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label : Some("Render Encoder")
        };
        let mut encoder = self.device.create_command_encoder(&command_encoder_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            println!("{:?}", self.oklahoma.ok);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.test_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::KeyW, true) => {
                self.oklahoma.ok = [0.0, self.oklahoma.ok[1] + 0.1, 0.0, 0.0];
                self.queue.write_buffer(&self.test_buffer, 0, bytemuck::cast_slice(&[self.oklahoma]));
            },
            _ => {}
        }
    }
}