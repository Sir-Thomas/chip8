const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u16 = 0x50;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const PIXEL_ON: u32 = 0xFFFFFFFF;

const FLAG_REGISTER: usize = 0xF;

struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    video: [u32; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            program_counter: START_ADDRESS,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            video: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
        }
    }

    fn load_fontset(&mut self) {
        let fontset = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for (i, &byte) in fontset.iter().enumerate() {
            self.memory[FONTSET_START_ADDRESS as usize + i] = byte;
        }
    }

    fn load_rom(&mut self, filename: &str) {
        let file = std::fs::read(filename).expect("Failed to open ROM file");

        for (i, &byte) in file.iter().enumerate() {
            self.memory[START_ADDRESS as usize + i] = byte;
        }
    }

    fn process_opcode(&mut self, opcode: u16) {
        let instruction_class = ((opcode & 0xF000) >> 12) as u8;
        let address = opcode & 0x0FFF;
        let vx = ((opcode & 0x0F00) >> 8) as u8;
        let vy = ((opcode & 0x00F0) >> 4) as u8;
        let byte = (opcode & 0x00FF) as u8;
        let nibble = (opcode & 0x000F) as u8;
        match instruction_class {
            0x0 => {
                match opcode {
                    // CLS: Clear the display
                    0x00e0 => self.video = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
                    // RET: Return from a subroutine
                    0x00ee => {
                        self.stack_pointer -= 1; // TODO: handle underflow
                        self.program_counter = self.stack[self.stack_pointer as usize];
                    },
                    _ => {} // TODO: error on invalid value
                }
            }
            0x1 => self.op_1nnn(address),
            0x2 => self.op_2nnn(address),
            0x3 => self.op_3xkk(vx, byte),
            0x4 => self.op_4xkk(vx, byte),
            0x5 => self.op_5xy0(vx, vy),
            0x6 => self.op_6xkk(vx, byte),
            0x7 => self.op_7xkk(vx, byte),
            0x8 => self.op_8xyn(vx, vy, nibble),
            0x9 => self.op_9xy0(vx, vy),
            0xA => self.op_annn(address),
            0xB => self.op_bnnn(address),
            0xC => self.op_cxkk(vx, byte),
            0xD => self.op_dxyn(vx, vy, nibble),
            0xE => self.op_exkk(vx, byte),
            0xF => self.op_fxkk(vx, byte),
            _ => {} //TODO: error on invalid value
        }
    }

    // JP addr: Jump to location nnn
    fn op_1nnn(&mut self, address: u16) {
        self.program_counter = address;
    }

    // CALL addr: Call subroutine at address nnn
    fn op_2nnn(&mut self, address: u16) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1; // TODO: handle overflow
        self.program_counter = address;
    }

    // SE Vx, byte
    fn op_3xkk(&mut self, vx: u8, byte: u8) {
        if self.registers[vx as usize] == byte {
            self.program_counter += 2;
        }
    }

    // SNE Vx, byte
    fn op_4xkk(&mut self, vx: u8, byte: u8) {
        if self.registers[vx as usize] != byte {
            self.program_counter += 2;
        }
    }

    // SE Vx, Vy
    fn op_5xy0(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.program_counter += 2;
        }
    }

    // LD Vx, byte
    fn op_6xkk(&mut self, vx: u8, byte: u8) {
        self.registers[vx as usize] = byte;
    }

    // ADD Vx, byte
    fn op_7xkk(&mut self, vx: u8, byte: u8) {
        (self.registers[vx as usize], _) = self.registers[vx as usize].overflowing_add(byte);
    }

    fn op_8xyn(&mut self, vx: u8, vy: u8, nibble: u8) {
        match nibble {
            0x0 => self.registers[vx as usize] = self.registers[vy as usize],
            0x1 => self.registers[vx as usize] |= self.registers[vy as usize],
            0x2 => self.registers[vx as usize] &= self.registers[vy as usize],
            0x3 => self.registers[vx as usize] ^= self.registers[vy as usize],
            0x4 => self.sum(vx, vy),
            0x5 => self.sub(vx, vy),
            0x6 => self.shr(vx),
            0x7 => self.subn(vx, vy),
            0xE => self.shl(vx),
            _ => {},
        }
    }

    fn sum(&mut self, vx: u8, vy: u8) {
        let (result, carry) = self.registers[vx as usize].overflowing_add(self.registers[vy as usize]);
        self.registers[FLAG_REGISTER] = carry as u8;
        self.registers[vx as usize] = result;
    }

    fn sub(&mut self, vx: u8, vy: u8) {
        let (result, carry) = self.registers[vx as usize].overflowing_sub(self.registers[vy as usize]);
        self.registers[FLAG_REGISTER] = carry as u8;
        self.registers[vx as usize] = result;
    }

    fn shr(&mut self, vx: u8) {
        self.registers[FLAG_REGISTER] = self.registers[vx as usize] & 0x1;
        self.registers[vx as usize] >>= 1;
    }

    fn subn(&mut self, vx: u8, vy: u8) {
        let (result, carry) = self.registers[vy as usize].overflowing_sub(self.registers[vx as usize]);
        self.registers[FLAG_REGISTER] = carry as u8;
        self.registers[vx as usize] = result;
    }

    fn shl(&mut self, vx: u8) {
        self.registers[FLAG_REGISTER] = (self.registers[vx as usize] & 0x80) >> 7;

        self.registers[vx as usize] <<= 1;
    }

    fn op_9xy0(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.program_counter += 2;
        }
    }

    fn op_annn(&mut self, address: u16) {
        self.index = address;
    }

    fn op_bnnn(&mut self, address: u16) {
        self.program_counter = self.registers[0] as u16 + address;
    }

    fn op_cxkk(&mut self, vx: u8, byte: u8) {
        self.registers[vx as usize] = rand::random::<u8>() & byte;
    }

    fn op_dxyn(&mut self, vx: u8, vy: u8, height: u8) {
        let x_pos = self.registers[vx as usize] as usize % DISPLAY_WIDTH;
        let y_pos = self.registers[vy as usize] as usize % DISPLAY_HEIGHT;

        self.registers[FLAG_REGISTER] = 0;

        for row in 0..height as usize {
            let byte = self.memory[self.index as usize + row as usize];
            for col in 0..8 {
                let sprite_pixel = byte & (0x80 >> col);
                if sprite_pixel != 0x0 {
                    let x = (x_pos + col) % DISPLAY_WIDTH;
                    let y = (y_pos + row) % DISPLAY_HEIGHT;
                    let screen_pixel = self.video[(y * DISPLAY_WIDTH + x) as usize];
                    if screen_pixel == PIXEL_ON {
                        self.registers[FLAG_REGISTER] = 1;
                    } 
                    self.video[(y * DISPLAY_WIDTH + x) as usize] = screen_pixel ^ PIXEL_ON;
                }
            }
        }
    }

    fn op_exkk(&mut self, vx: u8, byte: u8) {
        let key = self.registers[vx as usize];

        let pressed = match byte {
            0x9E => true,
            0xA1 => false,
            _ => return, // TODO: error on invalid value
        };

        if self.keypad[key as usize] == pressed {
            self.program_counter += 2;
        }
    }

    fn op_fxkk(&mut self, vx: u8, byte: u8) {
        match byte {
            0x07 => self.registers[vx as usize] = self.delay_timer,
            0x0A => self.wait_for_keypress(vx),
            0x15 => self.delay_timer = self.registers[vx as usize],
            0x18 => self.sound_timer = self.registers[vx as usize],
            0x1E => self.index += self.registers[vx as usize] as u16,
            0x29 => self.load_font(vx),
            0x33 => self.binary_coded_decimal(vx),
            0x55 => self.store_registers(vx),
            0x65 => self.load_registers(vx),
            _ => {} // TODO: error on invalid value
        }
    }

    fn wait_for_keypress(&mut self, vx: u8) {
        for i in 0..16 {
            if self.keypad[i as usize] {
                self.registers[vx as usize] = i;
                return;
            }
        }
        self.program_counter -= 2;
    }

    fn load_font(&mut self, vx: u8) {
        self.index = FONTSET_START_ADDRESS + 5 * self.registers[vx as usize] as u16;
    }

    fn binary_coded_decimal(&mut self, vx: u8) {
        let value = self.registers[vx as usize];
        self.memory[self.index as usize + 2] = value % 10;
        let value = value / 10;
        self.memory[self.index as usize + 1] = value % 10;
        let value = value / 10;
        self.memory[self.index as usize] = value % 10;
    }

    fn store_registers(&mut self, vx: u8) {
        for i in 0..=vx as usize {
            self.memory[self.index as usize + i] = self.registers[i];
        }
    }

    fn load_registers(&mut self, vx: u8) {
        for i in 0..=vx as usize {
            self.registers[i] = self.memory[self.index as usize + i];
        }
    }

    fn cycle(&mut self) {
        let msb = self.memory[self.program_counter as usize] as u16;
        let lsb = self.memory[self.program_counter as usize + 1] as u16;
        let opcode = (msb << 8) + lsb;

        self.program_counter += 2;

        self.process_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn process_input(&mut self) -> bool {
        false
    }
}

fn main() {
    print!("\x1B[2J\x1B[H");
    let mut args = std::env::args();
    let _ = args.next().unwrap_or_else(|| String::new());
    let rom: String = args.next().unwrap_or_else(|| String::from("roms/particle_demo.ch8"));
    let delay: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let _scale: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    let mut chip8 = Chip8::new();

    chip8.load_fontset();
    chip8.load_rom(&rom);

    let mut quit = false;

    let mut last_cycle_time = jiff::Timestamp::now();

    while !quit {
        quit = chip8.process_input();
        let current_time = jiff::Timestamp::now();
        let delta = current_time - last_cycle_time;
        if delta.get_milliseconds() > delay {
            last_cycle_time = current_time;
            chip8.cycle();
            display_video(&chip8.video);
            play_audio();
            update_keyboard();
        }
    }
}

fn display_video(video: &[u32; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
    print!("\x1B[2J\x1B[H");
    //print!("\x1B[H");
    
    for row in video.chunks_exact(DISPLAY_WIDTH) {
        for &pixel in row {
            print!("{}", if pixel == PIXEL_ON { "█" } else { " " });
        }
        println!();
    }
}

fn play_audio() {
}

fn update_keyboard() {
}
