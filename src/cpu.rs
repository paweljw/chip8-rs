use crate::opcode::Opcode;
use rand::random;
use std::fs::File;
use std::io::Read;

pub const GRAPHICS_WIDTH: usize = 64;
pub const GRAPHICS_HEIGHT: usize = 32;
pub const FONTSET_BYTES_PER_CHAR: u16 = 5;

const REGISTERS: usize = 16;

const FONTSET: [u8; 80] = [
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

pub struct Cpu {
    program_counter: u16,
    index_register: u16,
    stack_pointer: u16,
    memory: [u8; 4096],
    delay_timer: u8,
    sound_timer: u8,
    register: [u8; REGISTERS],
    pub graphics: [[bool; GRAPHICS_WIDTH]; GRAPHICS_HEIGHT],
    pub draw_flag: bool,
    done: bool,
    debug: bool,
    stack: Vec<u16>,
    keys: [bool; 16],
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            program_counter: 0x200,
            index_register: 0,
            stack_pointer: 0,
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
            register: [0; REGISTERS],
            graphics: [[false; GRAPHICS_WIDTH]; GRAPHICS_HEIGHT],
            draw_flag: false,
            done: false,
            debug: false,
            stack: Vec::<u16>::new(),
            keys: [false; 16],
        }
    }

    pub fn set_keys(&mut self, keys: &Vec<bool>) {
        for (i, &elem) in keys.iter().enumerate() {
            self.keys[i] = elem;
        }
    }

    pub fn reset_keys(&mut self) {
        for i in 0..16 {
            self.keys[i] = false;
        }
    }

    pub fn fetch_opcode(&self) -> Opcode {
        self.opcode_at(self.program_counter as usize)
    }

    fn opcode_at(&self, at: usize) -> Opcode {
        let opcode: u16 = ((self.memory[at] as u16) << 8) + self.memory[at + 1] as u16;

        Opcode::new(opcode)
    }

    pub fn load_program(&mut self, file_name: &str) {
        let mut file = File::open(file_name).expect("There was an issue reading the file.");
        let mut game_data = Vec::new();
        file.read_to_end(&mut game_data)
            .expect("Failure to read file");

        for i in 0..80 {
            self.memory[i as usize] = FONTSET[i as usize];
        }

        for i in &game_data {
            self.memory[self.program_counter as usize] = *i as u8;
            self.program_counter += 1;
        }

        self.program_counter = 0x200;
    }

    pub fn dump_program(&self) {
        let mut i = 0x200;

        while i < 0x1000 {
            let opcode = self.opcode_at(i);
            if opcode.opcode == 0 {
                break;
            }
            println!("{:#06x}: {}", i, opcode);
            i += 2;
        }
    }

    pub fn steps(&mut self, steps: u8) {
        for _ in 0..steps {
            self.step();
        }
    }

    pub fn step(&mut self) {
        if self.done {
            return;
        }

        let opcode = self.fetch_opcode();

        if self.debug {
            println!("{:#06x}: {}", self.program_counter, opcode);
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;

            if self.sound_timer == 0 {
                println!("BEEP!")
            }
        }

        match opcode.t() {
            0x0000 => match opcode.kk() {
                0x00e0 => self.clr(),
                0x00ee => self.ret(),
                _ => panic!("Unknown in 00: {}", opcode),
            },
            0x1000 => self.jump(opcode.nnn()),
            0x2000 => self.call(opcode.nnn()),
            0x3000 => self.ske(opcode.x(), opcode.kk()),
            0x4000 => self.skne(opcode.x(), opcode.kk()),
            0x5000 => self.skre(opcode.x(), opcode.y()),
            0x6000 => self.load(opcode.x(), opcode.kk()),
            0x7000 => self.add(opcode.x(), opcode.kk()),
            0x8000 => match opcode.n() {
                0x0 => self.mov(opcode.x(), opcode.y()),
                0x1 => self.or(opcode.x(), opcode.y()),
                0x2 => self.and(opcode.x(), opcode.y()),
                0x3 => self.xor(opcode.x(), opcode.y()),
                0x4 => self.addr(opcode.x(), opcode.y()),
                0x5 => self.sub(opcode.x(), opcode.y()),
                0x6 => self.shr(opcode.x()),
                0x7 => self.ssub(opcode.x(), opcode.y()),
                0xE => self.shl(opcode.x()),
                _ => panic!("Unknown in math: {}", opcode),
            },
            0x9000 => self.skrne(opcode.x(), opcode.y()),
            0xA000 => self.loadi(opcode.nnn()),
            0xB000 => self.jumpi(opcode.nnn()),
            0xC000 => self.rand(opcode.x(), opcode.kk()),
            0xD000 => self.draw(opcode.x(), opcode.y(), opcode.n()),
            0xE000 => match opcode.kk() {
                0x9E => self.skpr(opcode.x()),
                0xA1 => self.skup(opcode.x()),
                _ => panic!("Unknown in keys: {}", opcode),
            },
            0xF000 => match opcode.kk() {
                0x07 => self.moved(opcode.x()),
                0x15 => self.loadd(opcode.x()),
                0x18 => self.loads(opcode.x()),
                0x29 => self.ldspr(opcode.x()),
                0x33 => self.bcd(opcode.x()),
                0x0A => self.keyd(opcode.x()),
                0x1e => self.addi(opcode.x()),
                0x55 => self.mstor(opcode.x()),
                0x65 => self.mread(opcode.x()),
                _ => panic!("Unknown NN for opcode: {}", opcode),
            },
            _ => panic!("Unknown opcode: {}", opcode),
        }
    }

    pub fn draw_done(&mut self) {
        self.draw_flag = false;
    }

    pub fn restart(&mut self) {
        self.program_counter = 0x200;
        self.index_register = 0;
        self.stack_pointer = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.register = [0; REGISTERS];
        self.clr();
        self.draw_flag = false;
        self.done = false;
    }

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug;
    }

    fn loadi(&mut self, nnn: u16) {
        self.index_register = nnn;
        self.program_counter += 2;
    }

    fn rand(&mut self, x: u8, kk: u8) {
        self.register[x as usize] = random::<u8>() & kk;
        self.program_counter += 2;
    }

    fn ske(&mut self, register: u8, kk: u8) {
        if self.register[usize::from(register)] == kk {
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    fn draw(&mut self, register_x: u8, register_y: u8, n: u8) {
        self.register[0xF] = 0;
        let pos_x = self.register[usize::from(register_x)];
        let pos_y = self.register[usize::from(register_y)];

        let sprite = &self.memory
            [usize::from(self.index_register)..usize::from(self.index_register + n as u16)];

        for (row, byte) in sprite.iter().enumerate() {
            for col in 0..8 {
                let bit = (byte >> (7 - col)) & 0x1;

                let position_x: usize = (pos_x + col) as usize % GRAPHICS_WIDTH;
                let position_y: usize = (pos_y + row as u8) as usize % GRAPHICS_HEIGHT;

                let pixel = if self.graphics[position_y][position_x] {
                    1
                } else {
                    0
                };

                if bit == 1 && pixel == 1 {
                    self.register[0xF] = 1;
                }

                self.graphics[position_y][position_x] = (pixel ^ bit) == 1;
            }
        }

        self.draw_flag = true;
        self.program_counter += 2;
    }

    fn add(&mut self, x: u8, kk: u8) {
        self.register[x as usize] += kk;
        self.program_counter += 2;
    }

    fn jump(&mut self, nnn: u16) {
        if self.program_counter == nnn {
            self.done = true;
            if self.debug {
                println!("Detected infinite loop, DONE");
            }
            return;
        }

        self.program_counter = nnn;
    }

    fn load(&mut self, x: u8, kk: u8) {
        self.register[x as usize] = kk;
        self.program_counter += 2;
    }

    fn loadd(&mut self, x: u8) {
        self.delay_timer = self.register[x as usize];
        self.program_counter += 2;
    }

    fn skne(&mut self, x: u8, kk: u8) {
        if self.register[x as usize] != kk {
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    // Sigh, I know. There has to be a better way than to upcast them to a longer type
    // just to see whether the sum involves a carry.
    // Oh well.
    fn addr(&mut self, x: u8, y: u8) {
        let res: u16 = self.register[x as usize] as u16 + self.register[y as usize] as u16;

        self.register[0xF] = 0;
        if res > 0xFF {
            self.register[0xF] = 1;
        }

        self.register[x as usize] = res as u8 & 0xFF;

        self.program_counter += 2;
    }

    fn clr(&mut self) {
        self.graphics = [[false; GRAPHICS_WIDTH]; GRAPHICS_HEIGHT];
        self.program_counter += 2;
    }

    fn ret(&mut self) {
        let ret_addr = self.stack.pop().expect("Illegal jump with empty stack");
        self.program_counter = ret_addr;
    }

    fn call(&mut self, nnn: u16) {
        self.stack.push(self.program_counter + 2);
        self.program_counter = nnn;
    }

    fn moved(&mut self, x: u8) {
        self.register[x as usize] = self.delay_timer;
        self.program_counter += 2;
    }

    fn addi(&mut self, x: u8) {
        let val = self.register[x as usize] as u16 + self.index_register;

        self.register[0xF] = 0;
        if val > 0xfff {
            self.register[0xF] = 1;
        }

        self.index_register = val & 0xfff;
        self.program_counter += 2;
    }

    fn mov(&mut self, x: u8, y: u8) {
        self.register[x as usize] = self.register[y as usize];
        self.program_counter += 2;
    }

    fn and(&mut self, x: u8, y: u8) {
        self.register[x as usize] = self.register[x as usize] & self.register[y as usize];
        self.program_counter += 2;
    }

    fn mread(&mut self, x: u8) {
        for offset in 0..(x + 1) {
            self.register[offset as usize] =
                self.memory[(self.index_register + offset as u16) as usize];
        }

        self.program_counter += 2;
    }

    fn shl(&mut self, x: u8) {
        self.register[0xF] = (self.register[x as usize] >> 7) & 0x1;
        self.register[x as usize] = self.register[x as usize] << 1;
        self.program_counter += 2;
    }

    fn shr(&mut self, x: u8) {
        self.register[0xF] = self.register[x as usize] & 0x1;

        self.register[x as usize] = self.register[x as usize] >> 1;
        self.program_counter += 2;
    }

    fn sub(&mut self, x: u8, y: u8) {
        let vx: u8 = self.register[x as usize] as u8;
        let vy: u8 = self.register[y as usize] as u8;

        self.register[0xF] = 0;
        if vy > vx {
            self.register[0xF] = 1;
        }

        self.register[x as usize] = vx - vy;
        self.program_counter += 2;
    }

    fn mstor(&mut self, x: u8) {
        for offset in 0..(x + 1) {
            self.memory[(self.index_register + offset as u16) as usize] =
                self.register[offset as usize];
        }

        self.program_counter += 2;
    }

    fn skre(&mut self, x: u8, y: u8) {
        if self.register[x as usize] == self.register[y as usize] {
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    fn loads(&mut self, x: u8) {
        self.sound_timer = self.register[x as usize];
        self.program_counter += 2;
    }

    fn xor(&mut self, x: u8, y: u8) {
        self.register[x as usize] = self.register[x as usize] ^ self.register[y as usize];
        self.program_counter += 2;
    }

    fn skrne(&mut self, x: u8, y: u8) {
        if self.register[x as usize] != self.register[y as usize] {
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    fn or(&mut self, x: u8, y: u8) {
        self.register[x as usize] = self.register[x as usize] | self.register[y as usize];
        self.program_counter += 2;
    }

    fn ssub(&mut self, x: u8, y: u8) {
        let vx: u8 = self.register[x as usize] as u8;
        let vy: u8 = self.register[y as usize] as u8;

        self.register[0xF] = 0;
        if vx > vy {
            self.register[0xF] = 1;
        }

        self.register[x as usize] = vy - vx;
        self.program_counter += 2;
    }

    fn jumpi(&mut self, nnn: u16) {
        self.program_counter = self.register[0x0] as u16 + nnn;
    }

    fn skpr(&mut self, x: u8) {
        if self.keys[self.register[x as usize] as usize] {
            self.reset_keys();
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    fn skup(&mut self, x: u8) {
        if self.keys[self.register[x as usize] as usize] {
            self.reset_keys();
            self.program_counter += 2;
        } else {
            self.program_counter += 4;
        }
    }

    fn ldspr(&mut self, x: u8) {
        self.index_register = self.register[x as usize] as u16 * FONTSET_BYTES_PER_CHAR;
        self.program_counter += 2;
    }

    fn bcd(&mut self, x: u8) {
        self.memory[self.index_register as usize] =
            ((self.register[x as usize] as u16 % 1000) / 100) as u8;
        self.memory[self.index_register as usize + 1] = (self.register[x as usize] % 100) / 10;
        self.memory[self.index_register as usize + 2] = self.register[x as usize] % 100;
        self.program_counter += 2;
    }

    fn keyd(&mut self, x: u8) {
        for (i, &item) in self.keys.iter().enumerate() {
            if item {
                self.register[x as usize] = i as u8;
                self.program_counter += 2;
                return;
            }
        }
    }
}
