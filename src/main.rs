mod app;
mod gpu;
mod cpu;

use app::{App};
use winit::{
    event_loop::{ControlFlow, EventLoop}, 
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus}
};

use std::process::ExitCode;
use std::time::Duration;

fn run() -> std::process::ExitCode {
    env_logger::init();

    let mut event_loop = EventLoop::builder().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    loop {
        let timeout = Some(Duration::ZERO);
        let status = event_loop.pump_app_events(timeout, &mut app);

        if let PumpStatus::Exit(exit_code) = status {
            break ExitCode::from(exit_code as u8);
        }

        let cpu = match &mut app.cpu {
            Some(canvas) => canvas,
            None => continue,
        };

        cpu.process();
    }
}

fn main() {
    let _ = run();
}
