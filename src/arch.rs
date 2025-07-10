use std::sync::Arc;
use winit::window::Window;

use crate::cpu::{Cpu, GpuInstruction};
use crate::gpu::Gpu;

pub struct Arch {
    pub cpu: Cpu,
    pub gpu: Gpu,
    pub keypad: [bool; 16]
}

impl Arch {
    pub fn new(window : Arc<Window>) -> Self {
        let keypad = [false; 16];
        let cpu = Cpu::new(Arc::new(keypad));
        let gpu = pollster::block_on(Gpu::new(Arc::clone(&window))).unwrap();
        let keypad = [false; 16];

        Self {
            cpu,
            gpu,
            keypad
        }
    }

    pub fn emulate(&mut self) {
        match self.cpu.process() {
            GpuInstruction::Clear => {self.gpu.clear_screen();}
            GpuInstruction::XorSprite(pos_x, pos_y, sprite_data) => {
                if self.gpu.xor_sprite(pos_x, pos_y, sprite_data) {
                    self.cpu.reg[15] = 1;
                } else {
                    self.cpu.reg[15] = 0;
                }
            }
            _ => {}
        }
    }
}