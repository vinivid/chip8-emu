mod app;
mod gpu;
mod cpu;

use std::{error::Error};
use app::{App, SimEvents};
use winit::event_loop::EventLoop;

fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let event_loop : EventLoop<SimEvents> = EventLoop::with_user_event().build()?;

    let mut app = App::new(event_loop.create_proxy());
    let _ = event_loop.run_app(&mut app);

    Ok(())
}

fn main() {
    let _ = run();
}
