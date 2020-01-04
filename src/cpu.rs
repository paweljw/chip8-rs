use crate::opcode::Opcode;
use rand::random;
use std::fs::File;
use std::io::Read;

pub const GRAPHICS_WIDTH: usize = 64;
pub const GRAPHICS_HEIGHT: usize = 64;

const REGISTERS: usize = 16;

pub struct Cpu {
    program_counter: u16,
    index_register: u16,
    stack_pointer: u16,
    memory: [u16; 4096],
    delay_timer: u16,
    sound_timer: u16,
    register: [u16; REGISTERS],
    pub graphics: [[bool; GRAPHICS_WIDTH]; GRAPHICS_HEIGHT],
    pub draw_flag: bool,
    done: bool,
    debug: bool,
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
        }
    }

    pub fn fetch_opcode(&self) -> Opcode {
        self.opcode_at(usize::from(self.program_counter))
    }

    fn opcode_at(&self, at: usize) -> Opcode {
        let opcode = (self.memory[at] << 8) + self.memory[at + 1];

        Opcode::new(opcode)
    }

    pub fn load_program(&mut self, file_name: &str) {
        let mut file = File::open(file_name).expect("There was an issue reading the file.");
        let mut game_data = Vec::new();
        file.read_to_end(&mut game_data)
            .expect("Failure to read file");

        for i in &game_data {
            self.memory[usize::from(self.program_counter)] = u16::from(*i);
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

        match opcode.t() {
            0x1000 => self.jump(opcode.nnn()),
            0x3000 => self.ske(opcode.x(), opcode.kk()),
            0x4000 => self.skne(opcode.x(), opcode.kk()),
            0x6000 => self.load(opcode.x(), opcode.kk()),
            0x7000 => self.add(opcode.x(), opcode.kk()),
            0x8000 => match opcode.n() {
                0x4 => self.addr(opcode.x(), opcode.y()),
                _ => panic!("Unknown in math: {}", opcode),
            },
            0xA000 => self.loadi(opcode.nnn()),
            0xC000 => self.rand(opcode.x(), opcode.kk()),
            0xD000 => self.draw(opcode.x(), opcode.y(), opcode.n()),
            0xF000 => match opcode.kk() {
                0x15 => self.loadd(opcode.x()),
                _ => panic!("Unknown NN for opcode: {}", opcode),
            },
            _ => panic!("Unknown opcode: {}", opcode),
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
        self.graphics = [[false; GRAPHICS_WIDTH]; GRAPHICS_HEIGHT];
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

    fn rand(&mut self, register: u16, kk: u16) {
        self.register[usize::from(register)] = random::<u16>() & kk;
        self.program_counter += 2;
    }

    fn ske(&mut self, register: u16, kk: u16) {
        if self.register[usize::from(register)] == kk {
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    fn draw(&mut self, register_x: u16, register_y: u16, n: u16) {
        self.register[0xF] = 0;
        let pos_x = self.register[usize::from(register_x)];
        let pos_y = self.register[usize::from(register_y)];

        let sprite =
            &self.memory[usize::from(self.index_register)..usize::from(self.index_register + n)];

        for (i, byte) in sprite.iter().enumerate() {
            for num in 0..7 {
                if (byte & (0x80 >> num)) != 0 {
                    let position_y: usize = usize::from(pos_y) + i;
                    let position_x: usize = usize::from(pos_x + num);

                    let pixel = if self.graphics[position_y][position_x] {
                        1
                    } else {
                        0
                    };

                    if pixel == 1 {
                        self.register[0xF] = 1
                    }

                    self.graphics[position_y][position_x] = (pixel ^ 1) == 1;
                }
            }
        }

        self.draw_flag = true;
        self.program_counter += 2;
    }

    fn add(&mut self, x: u16, kk: u16) {
        self.register[usize::from(x)] += kk;
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

    fn load(&mut self, x: u16, kk: u16) {
        self.register[usize::from(x)] = kk;
        self.program_counter += 2;
    }

    fn loadd(&mut self, x: u16) {
        self.delay_timer = self.register[usize::from(x)];
        self.program_counter += 2;
    }

    fn skne(&mut self, x: u16, kk: u16) {
        if self.register[usize::from(x)] != kk {
            self.program_counter += 4;
        } else {
            self.program_counter += 2;
        }
    }

    fn addr(&mut self, x: u16, y: u16) {
        let res = self.register[x as usize] + self.register[y as usize];

        self.register[0xF] = 0;
        if res > 255 {
            self.register[0xF] = 1;
        }

        self.register[x as usize] = res & 0xFF;

        self.program_counter += 2;
    }
}
