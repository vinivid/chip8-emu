use std::{sync::Arc};
use winit::{
    dpi::PhysicalSize, 
    event_loop::ActiveEventLoop, 
    keyboard::{KeyCode}, 
    window::Window
};

pub struct State {
    surface : wgpu::Surface<'static>,
    device  : wgpu::Device,
    queue   : wgpu::Queue,
    config  : wgpu::SurfaceConfiguration,
    size    : PhysicalSize<u32>,
    window  : Arc<Window>
}

impl State {
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

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size
        })
    }

    pub fn resize(&mut self, new_size : PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
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
        let mut command_encoder = self.device.create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view : &view,
            resolve_target : None,
            ops : wgpu::Operations {
                load : wgpu::LoadOp::Clear(wgpu::Color 
                    { r: 0.75, g: 0.5, b: 0.25, a: 1.0}
                ),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label : Some("Render Pass"),
            color_attachments : &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set : None,
            timestamp_writes: None
        };

        command_encoder.begin_render_pass(&render_pass_descriptor);
        self.queue.submit(std::iter::once(command_encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}