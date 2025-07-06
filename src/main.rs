use wgpu::hal::dx12::Instance;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::*, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window
};

#[derive(Default)]
struct App {
    window : Option<Window>
}

struct State<'a> {
    surface : wgpu::Surface<'a>,
    device  : wgpu::Device,
    queue   : wgpu::Queue,
    config  : wgpu::SurfaceConfiguration,
    size    : PhysicalSize<u32>,
    window  : &'a Window
}

impl<'a> State<'a> {
    async fn new(window : &'a Window) -> Self {
        let size= window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends : wgpu::Backends::all(), ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface = instance.create_surface(window)
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
            label : Some("device"),
            ..Default::default()
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

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size
        }
    }

    fn resize(&mut self, new_size : PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view  = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label : Some("Render Encoder")
        };
        let mut command_encoder = self.device.create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view : &image_view,
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

        drawable.present();

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
        
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing"); 
                event_loop.exit();
            }
            _ => (),
        }
    }
}

async fn run() {
    env_logger::init();

    let event_loop = EventLoop::builder().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let mut state = State::new(&app.window.unwrap()).await;
    let _ = event_loop.run_app(&mut app);
}

fn main() {
    pollster::block_on(run())
}
