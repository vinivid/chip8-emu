use winit::{
    application::ApplicationHandler, 
    dpi::PhysicalSize, 
    event::*, 
    event_loop::{ActiveEventLoop, EventLoopProxy}, 
    keyboard::{PhysicalKey, KeyCode}, 
    window::Window
};
use std::sync::Arc;
use crate::cpu::Cpu;

pub enum SimEvents {
    PutRom,
    Process,
}

pub struct App {
    event_loop_proxy: EventLoopProxy<SimEvents>,
    cpu: Option<Cpu>,
}

impl App {
    pub fn new(loop_proxy : EventLoopProxy<SimEvents>) -> Self {
        Self {
            event_loop_proxy : loop_proxy,
            cpu: None,
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}

impl ApplicationHandler<SimEvents> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.cpu = Some(Cpu::new(window, self.event_loop_proxy.clone()));
        let _  = self.event_loop_proxy.send_event(SimEvents::PutRom);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: SimEvents) {
        let cpu = match &mut self.cpu {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            SimEvents::PutRom => {
                cpu.put_rom("IBM Logo.ch8");
            }
            SimEvents::Process => {
                cpu.process();
            }
        }
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {

        let cpu = match &mut self.cpu {
            Some(canvas) => canvas,
            None => return,
        };
        
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => cpu.gpu.resize(PhysicalSize { 
                width: size.width, 
                height: size.height }),
            WindowEvent::RedrawRequested => {
                let _ = cpu.gpu.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => Self::handle_key(self, event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}