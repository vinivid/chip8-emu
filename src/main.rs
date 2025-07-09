mod app;
mod gpu;
mod cpu;

use app::{App};
use winit::{
    event_loop::EventLoop, 
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus}
};

use std::process::ExitCode;
use std::time::Duration;
use std::thread;

fn run() -> std::process::ExitCode {
    env_logger::init();

    let mut event_loop = EventLoop::builder().build().unwrap();
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
        thread::sleep(Duration::from_millis(16));
    }
}

fn main() {
    let _ = run();
}
