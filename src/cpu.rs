use std::{iter::zip, sync::Arc};
use winit::{
    window::Window,
    event_loop::EventLoopProxy
};
use crate::gpu::Gpu;
use crate::app::SimEvents;

struct Stack {
    top_index : usize,
    values : [u16; 16]
}

impl Stack {
    fn new() -> Self {
        Self { 
            top_index: 0, 
            values: [0; 16]
        }
    }

    fn push(&mut self, vl : u16) {
        self.values[self.top_index] = vl;
        self.top_index += 1;
    }

    fn pop(&mut self) -> u16 {
        self.top_index -= 1;
        self.values[self.top_index + 1]        
    }
}

pub struct Cpu {
    //Even tho memory is separate in the archtecture, for 
    // now there is no reason to separete them
    memory : [u8; 4096],
    pc: u16,
    i_reg:  u16, // Register for pointing at memory
    reg: [u8; 16], //General use registers
    delay_timer: u8,
    sound_timer: u8, //What even is a sound timer?
    stack: Stack,
    event_loop_proxy: EventLoopProxy<SimEvents>,
    pub gpu: Gpu
}

impl Cpu {
    fn write_font(mem : &mut[u8; 4096]) {
        const FONT_START_ADDRES : usize = 0x50;
        const FONT_END_ADDRES : usize = 0x9F;
        const FONT_DATA : [u8; 80]
            = [
                0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                0x20, 0x60, 0x20, 0x20, 0x70, // 1
                0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                0xF0, 0x80, 0xF0, 0x80, 0x80, // F
            ];
        
        for (mem_value, font_data) in zip(&mut mem[FONT_START_ADDRES..(FONT_END_ADDRES + 1)], FONT_DATA) {
            *mem_value = font_data; 
        }
    }
    
    pub fn new(window : Arc<Window>, event_loop_proxy : EventLoopProxy<SimEvents>) -> Self {
        let mut memory : [u8; 4096] = [0; 4096];
        let gpu = pollster::block_on(Gpu::new(Arc::clone(&window))).unwrap();
        Self::write_font(&mut memory);

        Self {
            memory,
            pc: 0x200, //Start of a program
            i_reg : 0,
            reg: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            stack: Stack::new(),
            event_loop_proxy,
            gpu
        }        
    }

    pub fn put_rom(&mut self, rom_path : &str) {
        use std::fs::File;
        use std::io::Read;
        use std::io::BufReader;

        let rom_reader = BufReader::new(File::open(rom_path).unwrap());
        //0x200 is the start of the program to load to the chip 8
        let mut mem_addr: usize = 0x200;
        for byte in  rom_reader.bytes() {
            self.memory[mem_addr] = byte.unwrap();
            mem_addr += 1;
        }

        let _ = self.event_loop_proxy.send_event(SimEvents::Process);
    }

    pub fn process(&mut self) {
        let left_8_bits = self.memory[self.pc as usize];
        let right_8_bits = self.memory[(self.pc + 1) as usize];
        self.pc += 2;
        println!("Pc advancing {}", self.pc);

        match left_8_bits & 0xF0 {
            0 => {
                println!("test");
                let _ = self.event_loop_proxy.send_event(SimEvents::Process);
            }

            _ => {
                println!("shit");
                //let _ = self.event_loop_proxy.send_event(SimEvents::Process);
            }
        }
    }
}