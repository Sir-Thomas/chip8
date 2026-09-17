use std::fs::read;

use rand::random;

const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: usize = 0x50;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const PIXEL_ON: u32 = 0xFFFF_FFFF;

const FLAG_REGISTER: usize = 0xF;

pub struct Chip8 {
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
    pub const fn new() -> Self {
        Self {
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

    pub fn load_fontset(&mut self) {
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

        if let Some(destination) = self.memory.get_mut(FONTSET_START_ADDRESS..) {
            destination.iter_mut()
                .zip(fontset.iter().copied())
                .for_each(|(slot, byte)| *slot = byte);
        }
    }

    pub fn load_rom(&mut self, filename: &str) {
        if let Ok(file) = read(filename) && let Some(destination) = self.memory.get_mut(usize::from(START_ADDRESS)..) {
            destination.iter_mut()
                .zip(file.iter().copied())
                .for_each(|(slot, byte)| *slot = byte);
        }
    }

    fn process_opcode(&mut self, opcode: u16) {
        let instruction_class = (opcode & 0xF000) >> 12;
        let address = opcode & 0x0FFF;
        let vx: u8 = ((opcode & 0x0F00) >> 8).truncate();
        let vy: u8 = ((opcode & 0x00F0) >> 4).truncate();
        let byte = (opcode & 0x00FF).truncate();
        let nibble = (opcode & 0x000F).truncate();
        match instruction_class {
            0x0 => {
                match opcode {
                    // CLS: Clear the display
                    0x00e0 => self.video = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
                    // RET: Return from a subroutine
                    0x00ee => {
                        assert!(
                            (usize::from(self.stack_pointer)) > 0,
                            "Stack Underflow - RET with no matching CALL"
                        );
                        self.stack_pointer = self.stack_pointer.saturating_sub(1);
                        if let Some(pc) = self.stack.get(usize::from(self.stack_pointer)) {
                            self.program_counter = *pc;
                        }
                    },
                    _ => {} //unreachable!("Invalid Opcode") // Ignore instead of panic for certain
                            //ROMS - not sure if this is necessary
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
            _ => unreachable!("Instruction out of 4 bit range: {instruction_class:#x}")
        }
    }

    // JP addr: Jump to location nnn
    const fn op_1nnn(&mut self, address: u16) {
        self.program_counter = address;
    }

    // CALL addr: Call subroutine at address nnn
    fn op_2nnn(&mut self, address: u16) {
        assert!(
            usize::from(self.stack_pointer) < self.stack.len(),
            "Stack Overflow - Too many nested subroutines",
        );
        if let Some(address) = self.stack.get_mut(usize::from(self.stack_pointer)) {
            *address = self.program_counter;
        }
        self.stack_pointer = self.stack_pointer.saturating_add(1);
        self.program_counter = address;
    }

    // SE Vx, byte
    fn op_3xkk(&mut self, vx: u8, byte: u8) {
        if self.register(vx) == byte {
            self.program_counter = self.program_counter.saturating_add(2);
        }
    }

    // SNE Vx, byte
    fn op_4xkk(&mut self, vx: u8, byte: u8) {
        if self.register(vx) != byte {
            self.program_counter = self.program_counter.saturating_add(2);
        }
    }

    // SE Vx, Vy
    fn op_5xy0(&mut self, vx: u8, vy: u8) {
        if self.register(vx) == self.register(vy) {
            self.program_counter = self.program_counter.saturating_add(2);
        }
    }

    // LD Vx, byte
    fn op_6xkk(&mut self, vx: u8, byte: u8) {
        *self.register_mut(vx) = byte;
    }

    // ADD Vx, byte
    fn op_7xkk(&mut self, vx: u8, byte: u8) {
        (*self.register_mut(vx), _) = self.register(vx).overflowing_add(byte);
    }

    fn op_8xyn(&mut self, vx: u8, vy: u8, nibble: u8) {
        match nibble {
            0x0 => *self.register_mut(vx) = self.register(vy),
            0x1 => *self.register_mut(vx) |= self.register(vy),
            0x2 => *self.register_mut(vx) &= self.register(vy),
            0x3 => *self.register_mut(vx) ^= self.register(vy),
            0x4 => self.sum(vx, vy),
            0x5 => self.sub(vx, vy),
            0x6 => self.shr(vx),
            0x7 => self.subn(vx, vy),
            0xE => self.shl(vx),
            _ => {},
        }
    }

    fn sum(&mut self, vx: u8, vy: u8) {
        let (result, carry) = self.register(vx).overflowing_add(self.register(vy));
        *self.register_mut(FLAG_REGISTER) = u8::from(carry);
        *self.register_mut(vx) = result;
    }

    fn sub(&mut self, vx: u8, vy: u8) {
        let (result, carry) = self.register(vx).overflowing_sub(self.register(vy));
        *self.register_mut(FLAG_REGISTER) = u8::from(carry);
        *self.register_mut(vx) = result;
    }

    fn shr(&mut self, vx: u8) {
        *self.register_mut(FLAG_REGISTER) = self.register(vx) & 0x1;
        *self.register_mut(usize::from(vx)) >>= 1;
    }

    fn subn(&mut self, vx: u8, vy: u8) {
        let (result, carry) = self.register(vy).overflowing_sub(self.register(vx));
        *self.register_mut(FLAG_REGISTER) = u8::from(carry);
        *self.register_mut(vx) = result;
    }

    fn shl(&mut self, vx: u8) {
        *self.register_mut(FLAG_REGISTER) = (self.register(vx) & 0x80) >> 7;

        *self.register_mut(vx) <<= 1;
    }

    fn op_9xy0(&mut self, vx: u8, vy: u8) {
        if self.register(vx) != self.register(vy) {
            self.program_counter = self.program_counter.saturating_add(2);
        }
    }

    const fn op_annn(&mut self, address: u16) {
        self.index = address;
    }

    fn op_bnnn(&mut self, address: u16) {
        self.program_counter = u16::from(self.register(0u8)).saturating_add(address);
    }

    fn op_cxkk(&mut self, vx: u8, byte: u8) {
        *self.register_mut(vx) = random::<u8>() & byte;
    }

    fn op_dxyn(&mut self, vx: u8, vy: u8, height: u8) {
        let x_pos = usize::from(self.register(vx)) % DISPLAY_WIDTH;
        let y_pos = usize::from(self.register(vy)) % DISPLAY_HEIGHT;

        *self.register_mut(FLAG_REGISTER) = 0;

        for row in 0..usize::from(height) {
            let byte = self.read(usize::from(self.index).saturating_add(row));
            for col in 0..8 {
                let sprite_pixel = byte & (0x80 >> col);
                if sprite_pixel == 0x0 {
                    continue
                }
                let x = x_pos.saturating_add(col) % DISPLAY_WIDTH;
                let y = y_pos.saturating_add(row) % DISPLAY_HEIGHT;
                let screen_pixel = self.pixel(y.saturating_mul(DISPLAY_WIDTH).saturating_add(x));
                if screen_pixel == PIXEL_ON {
                    *self.register_mut(FLAG_REGISTER) = 1;
                } 
                *self.pixel_mut(y.saturating_mul(DISPLAY_WIDTH).saturating_add(x)) = screen_pixel ^ PIXEL_ON;
            }
        }
    }

    fn op_exkk(&mut self, vx: u8, byte: u8) {
        let key = self.register(vx);

        let pressed = match byte {
            0x9E => true,
            0xA1 => false,
            _ => unreachable!("Invalid Opcode")
        };

        if self.key(key) == pressed {
            self.program_counter = self.program_counter.saturating_add(2);
        }
    }

    fn op_fxkk(&mut self, vx: u8, byte: u8) {
        match byte {
            0x07 => *self.register_mut(vx) = self.delay_timer,
            0x0A => self.wait_for_keypress(vx),
            0x15 => self.delay_timer = self.register(vx),
            0x18 => self.sound_timer = self.register(vx),
            0x1E => self.index = self.index.saturating_add(u16::from(self.register(vx))),
            0x29 => self.load_font(vx),
            0x33 => self.binary_coded_decimal(vx),
            0x55 => self.store_registers(vx),
            0x65 => self.load_registers(vx),
            _ => unreachable!("Invalid Opcode: F{vx:X}{byte:X}")
        }
    }

    fn wait_for_keypress(&mut self, vx: u8) {
        for key_index in 0..16 {
            if self.key(key_index) {
                *self.register_mut(vx) = key_index;
                return;
            }
        }
        self.program_counter = self.program_counter.saturating_sub(2);
    }

    fn load_font(&mut self, vx: u8) {
        self.index = FONTSET_START_ADDRESS
            .saturating_add(5usize.saturating_mul(usize::from(self.register(vx))))
            .truncate();
    }

    fn binary_coded_decimal(&mut self, vx: u8) {
        let value = self.register(vx);
        *self.memory_mut(self.index.saturating_add(2)) = value % 10;
        let value = value / 10;
        *self.memory_mut(self.index.saturating_add(1)) = value % 10;
        let value = value / 10;
        *self.memory_mut(self.index) = value % 10;
    }

    fn store_registers(&mut self, vx: u8) {
        for i in 0..=vx {
            *self.memory_mut(self.index.saturating_add(u16::from(i))) = self.register(i);
        }
    }

    fn load_registers(&mut self, vx: u8) {
        for i in 0..=vx {
            *self.register_mut(i) = self.read(self.index.saturating_add(u16::from(i)));
        }
    }

    fn register_mut<I: Into<usize>>(&mut self, index: I) -> &mut u8 {
        self.registers.get_mut(index.into())
            .expect("Index is always 0-15 because of bit mask")
    }

    fn register<I: Into<usize>>(&self, index: I) -> u8 {
        *self.registers.get(index.into())
            .expect("Index is always 0-15 because of bit mask")
    }

    fn pixel_mut<I: Into<usize>>(&mut self, index: I) -> &mut u32 {
        self.video.get_mut(index.into())
            .expect("Index is bounded by 12 bit address")
    }

    fn pixel<I: Into<usize>>(&self, index: I) -> u32 {
        *self.video.get(index.into())
            .expect("Index is bounded by 12 bit address")
    }

    fn memory_mut<I: Into<usize>>(&mut self, index: I) -> &mut u8 {
        self.memory.get_mut(index.into())
            .expect("Index is bounded by 12 bit address")
    }

    fn read<I: Into<usize>>(&self, index: I) -> u8 {
        *self.memory.get(index.into())
            .expect("Index is bounded by 12 bit address")
    }

    fn key<I: Into<usize>>(&self, index: I) -> bool {
        *self.keypad.get(index.into())
            .expect("Index is always 0-15 because of bit mask")
    }

    pub fn cycle(&mut self) {
        let msb = u16::from(self.read(self.program_counter));
        let lsb = u16::from(self.read(self.program_counter.saturating_add(1)));
        let opcode = (msb << 8).saturating_add(lsb);

        self.program_counter = self.program_counter.saturating_add(2);

        self.process_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer = self.delay_timer.saturating_sub(1);
        }

        if self.sound_timer > 0 {
            self.sound_timer = self.sound_timer.saturating_sub(1);
        }
    }

    pub const fn get_video(&self) -> &[u32; DISPLAY_WIDTH * DISPLAY_HEIGHT] {
        &self.video
    }

    pub const fn process_input(&mut self, input: [bool; 16]) {
        self.keypad = input;
    }
}
