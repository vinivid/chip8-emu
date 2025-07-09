use std::{sync::Arc};
use wgpu::{util::DeviceExt};
use winit::{
    dpi::PhysicalSize,
    window::Window
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x2];

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
struct PixelColor {
    col: f32
}

impl PixelColor {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32
                }
            ]
        }
    }
}

pub struct Gpu {
    surface : wgpu::Surface<'static>,
    device  : wgpu::Device,
    queue   : wgpu::Queue,
    config  : wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer : wgpu::Buffer,
    index_buffer : wgpu::Buffer,
    num_indices : u32,
    pub window : Arc<Window>,
    pixel_array : [PixelColor; 2048],
    pixel_buffer : wgpu::Buffer
}

impl Gpu {
    pub async fn new(window : Arc<Window>) -> anyhow::Result<Self> {
        let size= window.inner_size();

        /* The primary backend uses quite a bit of memory but i don't care */
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends : wgpu::Backends::PRIMARY, ..Default::default()
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), 
                buffers: &[Vertex::desc(), PixelColor::desc()], 
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

        /* This represents a pixel on the screen, on the the top left
        position. It is necessary to put it in a such a weird position 
        because we are going to draw multiple instances of it based on the offset
        of the instance.
            The 0.03125 is the width of the pixel, that is 2/64, where 2 is the
        width of the scree coordinates and 64 is the width of the screen.
            0.0625 is the height of the pixel, following the same logic.
        */
        const PIXEL: &[Vertex] = &[
            Vertex { pos : [-1.0, 1.0]},  //Top left
            Vertex { pos : [-1.0, 0.9375]}, //Bottom left
            Vertex { pos : [-0.96875, 1.0]},   //Top right
            Vertex { pos : [-0.96875, 0.9375]}   //Bottom right
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(PIXEL),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        const INDICES: &[u16] = &[
            2, 0, 1,
            2, 1, 3
        ];

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = INDICES.len() as u32;

        let pixel_array : [PixelColor; 2048] = [PixelColor {col : 0.0}; 2048];

        let pixel_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Pixel instances buffer"),
                contents: bytemuck::cast_slice(&[pixel_array]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Ok(Self {
            surface,
            device,
            queue,
            render_pipeline,
            config,
            vertex_buffer,
            index_buffer,
            num_indices,
            window,
            pixel_array,
            pixel_buffer
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

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.pixel_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // 2048 is the total number of pixels (64 x 32)
            render_pass.draw_indexed(0..self.num_indices, 0, 0..2048);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn clear_screen(&mut self) {
        self.pixel_array = [PixelColor {col : 0.0}; 2048];
        self.queue.write_buffer(&self.pixel_buffer, 0, bytemuck::cast_slice(&[self.pixel_array]));
        self.window.request_redraw();
    }

    pub fn xor_sprite(&mut self, pos_x : usize, pos_y : usize, sprite_data : Vec<u8>) {
        for (row, value) in sprite_data.iter().enumerate() {
            for column in 0..8 {
                if (value & (0x80 >> column)) != 0 {
                    let pixel_at_position = self.pixel_array[(pos_y+row)*64 + (column + pos_x)].col;
                    if pixel_at_position > 0.3 {
                        self.pixel_array[(pos_y+row)*64 + (column + pos_x)].col = 0.0;
                    } else {
                        self.pixel_array[(pos_y+row)*64 + (column + pos_x)].col = 1.0;
                    }
                }
            }
        }

        self.queue.write_buffer(&self.pixel_buffer, 0, bytemuck::cast_slice(&[self.pixel_array]));
        self.window.request_redraw();
    }
}