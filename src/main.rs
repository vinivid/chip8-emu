mod state;
mod app;
mod renderer;

use std::{error::Error};
use state::State;
use app::App;
use winit::event_loop::EventLoop;

fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let event_loop : EventLoop<State> = EventLoop::with_user_event().build()?;

    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);

    Ok(())
}

fn main() {
    let _ = run();
}
