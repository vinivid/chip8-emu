use crate::renderer::Renderer;
use std::sync::Arc;
use winit::{
    event_loop::ActiveEventLoop, 
    keyboard::KeyCode, 
    window::Window
};

pub struct State {
    pub renderer : Renderer,
    window  : Arc<Window>,
}

impl State {
    pub fn new(window : Arc<Window>) -> Self {
        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window))).unwrap();
        Self { 
            renderer, 
            window
        }
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}