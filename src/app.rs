use winit::{
    application::ApplicationHandler, 
    dpi::PhysicalSize, 
    event::*, 
    event_loop::{ActiveEventLoop}, 
    keyboard::{PhysicalKey, KeyCode}, 
    window::Window
};
use std::sync::Arc;
use crate::arch::Arch;
pub struct App {
    pub arch: Option<Arch>,
}

impl App {
    pub fn new() -> Self {
        Self {
            arch: None,
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if let Some(arch) = &mut self.arch {
            match (code, is_pressed) {
                (KeyCode::Escape, true) => event_loop.exit(),
                (KeyCode::Digit1, true) => arch.keypad[1] = true,
                (KeyCode::Digit2, true) => arch.keypad[2] = true,
                (KeyCode::Digit3, true) => arch.keypad[3] = true,
                (KeyCode::Digit4, true) => arch.keypad[12] = true,
                (KeyCode::KeyQ, true) => arch.keypad[4] = true,
                (KeyCode::KeyW, true) => arch.keypad[5] = true,
                (KeyCode::KeyE, true) => arch.keypad[6] = true,
                (KeyCode::KeyR, true) => arch.keypad[13] = true,
                (KeyCode::KeyA, true) => arch.keypad[7] = true,
                (KeyCode::KeyS, true) => arch.keypad[8] = true,
                (KeyCode::KeyD, true) => arch.keypad[9] = true,
                (KeyCode::KeyF, true) => arch.keypad[14] = true,
                (KeyCode::KeyZ, true) => arch.keypad[10] = true,
                (KeyCode::KeyX, true) => arch.keypad[0] = true,
                (KeyCode::KeyC, true) => arch.keypad[11] = true,
                (KeyCode::KeyV, true) => arch.keypad[15] = true,
                (KeyCode::Digit1, false) => arch.keypad[1] = false,
                (KeyCode::Digit2, false) => arch.keypad[2] = false,
                (KeyCode::Digit3, false) => arch.keypad[3] = false,
                (KeyCode::Digit4, false) => arch.keypad[12] = false,
                (KeyCode::KeyQ, false) => arch.keypad[4] = false,
                (KeyCode::KeyW, false) => arch.keypad[5] = false,
                (KeyCode::KeyE, false) => arch.keypad[6] = false,
                (KeyCode::KeyR, false) => arch.keypad[13] = false,
                (KeyCode::KeyA, false) => arch.keypad[7] = false,
                (KeyCode::KeyS, false) => arch.keypad[8] = false,
                (KeyCode::KeyD, false) => arch.keypad[9] = false,
                (KeyCode::KeyF, false) => arch.keypad[14] = false,
                (KeyCode::KeyZ, false) => arch.keypad[10] = false,
                (KeyCode::KeyX, false) => arch.keypad[0] = false,
                (KeyCode::KeyC, false) => arch.keypad[11] = false,
                (KeyCode::KeyV, false) => arch.keypad[15] = false,
                _ => {}
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let mut arch = Arch::new(window);
        arch.cpu.put_rom("Pong (1 player).ch8");
        self.arch = Some(arch);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {

        let arch = match &mut self.arch {
            Some(canvas) => canvas,
            None => return,
        };
        
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => arch.gpu.resize(PhysicalSize { 
                width: size.width, 
                height: size.height }),
            WindowEvent::RedrawRequested => {let _ = arch.gpu.render();}
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