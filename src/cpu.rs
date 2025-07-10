use std::iter::zip;
use std::sync::Arc;

use rand::Rng;

#[derive(Debug)]
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
        self.values[self.top_index]
    }
}

#[derive(PartialEq, Debug)]
pub enum GpuInstruction {
    Clear,
    XorSprite(usize, usize, Vec<u8>),
    Nothing
}

#[derive(Debug)]
pub struct Cpu {
    memory : [u8; 4096],
    pc: u16,
    i_reg:  u16, // Register for pointing at memory
    pub reg: [u8; 16], //General use registers
    delay_timer: u8,
    sound_timer: u8, //What even is a sound timer?
    stack: Stack,
    keypad_view: Arc<[bool; 16]>,
    waiting_for_key: (bool, usize),
}

impl Cpu {
    const FONT_START_ADDRES : usize = 0x50;

    fn write_font(mem : &mut[u8; 4096]) {
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
        
        for (mem_value, font_data) in zip(&mut mem[Self::FONT_START_ADDRES..(FONT_END_ADDRES + 1)], FONT_DATA) {
            *mem_value = font_data; 
        }
    }
    
    pub fn new(keypad_view : Arc<[bool; 16]>) -> Self {
        let mut memory : [u8; 4096] = [0; 4096];
        Self::write_font(&mut memory);

        Self {
            memory,
            pc: 0x200, //Start of a program
            i_reg : 0,
            reg: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            stack: Stack::new(),
            keypad_view,
            waiting_for_key: (false, 17),
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

    }

    /// Returns the value of the first key that is being pressed.
    /// If there are no keys being pressed, returns none.
    fn check_if_key_is_pressed(&mut self) -> Option<usize> {
        for (i, x) in self.keypad_view.iter().enumerate() {
            if *x {return Some(i);}
        }
        None
    }

    fn arith_operations_execution(&mut self, instr : u16) {
        let vx = ((instr & 0x0F00) >> 8) as usize;
        let vy = ((instr & 0x00F0) >> 4) as usize;
        match instr & 0x000F {
            //LD Vx, Vy
            0x0000 => {self.reg[vx] = self.reg[vy];}
            //OR Vx, Vy
            0x0001 => {self.reg[vx] = self.reg[vx] | self.reg[vy]}
            //AND Vx, Vy
            0x0002 => {self.reg[vx] = self.reg[vx] & self.reg[vy]}
            //XOR Vx, Vy
            0x0003 => {self.reg[vx] = self.reg[vx] ^ self.reg[vy]}
            //8xy4 - ADD Vx, Vy
            0x0004 => {
                let a = self.reg[vx] as u16;
                let b = self.reg[vy] as u16;
                let res = a.wrapping_add(b);
                self.reg[vx] = (res & 0x00FF) as u8;
                if res > 255 {
                    self.reg[15] = 1;
                } else {
                    self.reg[15] = 0;
                }
            }
            //SUB Vx, Vy
            0x0005 => {
                let a = self.reg[vx] as u16;
                let b = self.reg[vy] as u16;
                let res = a.wrapping_sub(b);
                self.reg[vx] = (res & 0x00FF) as u8;
                if a > b {
                    self.reg[15] = 1;
                } else {
                    self.reg[15] = 0;
                }
            }
            //SHR Vx {, Vy}
            0x0006 => {
                if (self.reg[vx] & 0x1) == 1 {
                    self.reg[15] = 1;
                } else {
                    self.reg[15] = 1;
                }

                self.reg[vx] = self.reg[vx] >> 1;
            }
            //SUBN Vx, Vy
            0x0007 => {
                let a = self.reg[vx] as u16;
                let b = self.reg[vy] as u16;
                let res = b.wrapping_sub(a);
                self.reg[vx] = (res & 0x00FF) as u8;
                if b > a {
                    self.reg[15] = 1;
                } else {
                    self.reg[15] = 0;
                }
            }
            //SHL Vx {, Vy}
            0x000E => {
                if (self.reg[vx] & 128) == 1 {
                    self.reg[15] = 1;
                } else {
                    self.reg[15] = 1;
                }

                self.reg[vx] = self.reg[vx] << 1;
            }
            _ => println!("invalid instruction {:X}", instr)
        }
    }

    fn decode_and_execute(&mut self, instr : u16) -> GpuInstruction {
        match instr & 0xF000 {
            0x0000 => {
                // Considering only the base chip8 instructions, only the
                // last 4 bits matter for instructions with 0.
                match instr & 0x000F {
                    //Clear screen - CLS
                    0x0000 => { return GpuInstruction::Clear;}

                    // Return - RET
                    0x000E => { self.pc = self.stack.pop();}
                    _ => {println!("invalid instruction");}
                }
            }

            // jump - JP addr 
            0x1000 => { self.pc = instr & 0x0FFF; }

            //call addr
            0x2000 => { 
                self.stack.push(self.pc);
                self.pc = instr & 0x0FFF;
            }

            //Skip if equal - SE vx, kk
            0x3000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                if self.reg[vx] == ((instr & 0x00FF) as u8) {
                    self.pc += 2;             
                }
            }

            //Skip if not equal - SE vx, kk
            0x4000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                if self.reg[vx] != ((instr & 0x00FF) as u8) {
                    self.pc += 2;             
                }
            }

            //Skip if register is equal - SE vx, vy
            0x5000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let vy = ((instr & 0x00F0) >> 4) as usize;
                if self.reg[vx] == self.reg[vy] {
                    self.pc += 2;             
                }
            }

            // load immediate - LD vx byte 
            0x6000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let immediate: u8 = (instr & 0x00FF) as u8;
                self.reg[vx] = immediate; 
            }

            // Add immdiate and save - ADD vx, nn
            0x7000 => {
                let vx: usize = ((instr & 0x0F00) >> 8) as usize;
                let immediate: u8 = (instr & 0x00FF) as u8;
                self.reg[vx] = self.reg[vx].wrapping_add(immediate); 
            }

            // Arithmetic operations
            0x8000 => { self.arith_operations_execution(instr);}

            //Skip if reg not equal 
            0x9000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let vy = ((instr & 0x00F0) >> 4) as usize;
                if self.reg[vx] != self.reg[vy] {
                    self.pc += 2;             
                }
            }

            // load immediate to i - LD I, addr
            0xA000 => {
                let immediate = instr & 0x0FFF;
                self.i_reg = immediate;
            }

            // Branch = v0 + immediate
            0xB000 => {
                let immediate = instr & 0x0FFF;
                self.pc = (self.reg[0] as u16) + immediate;
            }

            // RND Vx, byte
            0xC000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let mut rng = rand::rng();
                let rand = rng.random_range(0..256) as u8;
                self.reg[vx] = rand & ((instr & 0x00FF) as u8);
            }

            // Dxyn - DRW Vx, Vy, nibble
            0xD000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                let vy = ((instr & 0x00F0) >> 4) as usize;
                let pos_x = self.reg[vx] as usize;
                let pos_y= self.reg[vy] as usize;
                let qtt = (instr & 0x000F) as usize;
                let mut indexer = self.i_reg as usize;
                let mut sprite_vec: Vec<u8> = Vec::new();

                for _ in 0..qtt {
                    sprite_vec.push(self.memory[indexer]);
                    indexer += 1;
                }

                return GpuInstruction::XorSprite(pos_x, pos_y, sprite_vec);
            }

            //E instructions
            0xE000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                match instr & 0x00FF {
                    // Skip if key is pressed
                    0x9E => {
                        if self.keypad_view[self.reg[vx] as usize] {
                            self.pc += 2;
                        }
                    }

                    //Skip if key is not pressed
                    0xA1 => {
                        if !self.keypad_view[self.reg[vx] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => println!("Invalid instruction 0x{:X}", instr)
                }
            }

            // F instructions
            0xF000 => {
                let vx = ((instr & 0x0F00) >> 8) as usize;
                match instr & 0x00FF {
                    // Load delay timer
                    0x07 => self.reg[vx] = self.delay_timer,
                    //Load pressed key
                    0x0A => {
                        if let Some(key) = self.check_if_key_is_pressed() {
                            self.reg[vx] = key as u8;
                        } else {
                            self.waiting_for_key = (true, vx);
                        }
                    }
                    // Set delay timer to vx
                    0x15 => self.delay_timer = self.reg[vx],
                    // Set sound timer to vx
                    0x18 => self.sound_timer = self.reg[vx],
                    //Ad vx to i
                    0x1E => self.i_reg = self.i_reg.wrapping_add(self.reg[vx] as u16),
                    //Set i to location of font with value of vx
                    0x29 => self.i_reg = self.reg[vx].wrapping_add(Self::FONT_START_ADDRES as u8) as u16,
                    // Bcd representation of vx
                    0x33 => {
                        let unitary = self.reg[vx] % 10;
                        let decimal = (self.reg[vx] / 10) % 10;
                        let centesimal = self.reg[vx] / 100;
                        self.memory[self.i_reg as usize] = centesimal;
                        self.memory[(self.i_reg + 1) as usize] = decimal;
                        self.memory[(self.i_reg + 2) as usize] = unitary;
                    }
                    // Store all registers to addres I
                    0x55 => {
                        let mut indexer = self.i_reg as usize;
                        for i in 0..(vx + 1) {
                            self.memory[indexer] = self.reg[i];
                            indexer += 1;
                        }
                    }
                    // Read to all registers starting at addres I
                    0x65 => {
                        let mut indexer = self.i_reg as usize;
                        for i in 0..(vx + 1) {
                            self.reg[i] = self.memory[indexer];
                            indexer += 1;
                        }
                    }

                    _ => println!("Invalid instruction 0x{:X}", instr)
                }
            }

            _ => {
                println!("Invalid instruction 0x{:X}", instr);
            }
        }

        return GpuInstruction::Nothing;
    }

    pub fn process(&mut self) -> GpuInstruction {
        self.delay_timer = self.delay_timer.wrapping_add(1);

        if self.waiting_for_key.0 {
            if let Some(key) = Self::check_if_key_is_pressed(self) {
                self.reg[self.waiting_for_key.1] = key as u8;
                self.waiting_for_key.0 = false;
            }
        }

        let left_8_bits= self.memory[self.pc as usize] as u16;
        let right_8_bits = self.memory[(self.pc + 1) as usize] as u16;
        let instr: u16 = (left_8_bits << 8) | right_8_bits; 
        self.pc += 2;
        //println!("what instruction {:X}", instr);

        self.decode_and_execute(instr)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const FONT_START_ADDRES : usize = 0x50;
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

    const START_ADDRES : usize = 0x200;
        
    #[test]
    fn test_create_new_cpu() {
        let keypad_array = [false; 16];
        let cpu = Cpu::new(Arc::new(keypad_array));
        assert_eq!(cpu.pc, START_ADDRES as u16);

        let mut indexer : usize = 0;
        for _ in 0..80 {
            assert_eq!(cpu.memory[FONT_START_ADDRES + indexer], FONT_DATA[indexer]);
            indexer += 1;
        }
    }

    #[test]
    fn test_clear_instruction() {
        let keypad_array = [false; 16];
        let mut cpu = Cpu::new(Arc::new(keypad_array));
        cpu.memory[START_ADDRES] = 0x02;
        cpu.memory[START_ADDRES + 1] = 0x40;

        let ret = cpu.process();
        assert_eq!(ret, GpuInstruction::Clear);
        assert_eq!(cpu.pc, (START_ADDRES + 2) as u16);
    }

    #[test]
    fn test_call() {
        let keypad_array = [false; 16];
        let mut cpu = Cpu::new(Arc::new(keypad_array));
        cpu.memory[START_ADDRES] = 0x20;
        cpu.memory[START_ADDRES + 1] = 0x04;

        cpu.process();
        assert_eq!(cpu.pc, 4);
        assert_eq!(cpu.stack.values[0], 0x202);
        assert_eq!(cpu.stack.top_index, 1);
    }

    #[test]
    fn test_ret() {
        let keypad_array = [false; 16];
        let mut cpu = Cpu::new(Arc::new(keypad_array));
        cpu.memory[START_ADDRES] = 0x20;
        cpu.memory[START_ADDRES + 1] = 0x04;

        cpu.memory[4] = 0x00;
        cpu.memory[4 + 1] = 0xEE;

        cpu.process();
        cpu.process();
        assert_eq!(cpu.pc, (START_ADDRES + 2) as u16);
        assert_eq!(cpu.stack.top_index, 0);
    }
}
