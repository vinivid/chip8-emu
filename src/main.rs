use chip8::{app::App};
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

        if let Some(arch) = &mut app.arch {
            arch.emulate();
            thread::sleep(Duration::from_millis(8));
        };
    }
}

fn main() {
    let _ = run();
}
