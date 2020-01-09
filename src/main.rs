mod cpu;
mod opcode;

use cpu::*;

extern crate minifb;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use std::env;

const INITIAL_SPEED: u8 = 10;
const MIN_SPEED: u8 = 1;
const MAX_SPEED: u8 = 40;

const KEY_DEBOUNCE: u8 = 5;

fn main() {
    let mut cpu = Cpu::new();

    let filename = env::args()
        .nth(1)
        .expect("Pass a filename as first argument.");

    cpu.load_program(&filename);
    cpu.dump_program();

    let mut buffer: Vec<u32> = vec![0; GRAPHICS_WIDTH * GRAPHICS_HEIGHT];

    let mut speed = INITIAL_SPEED;
    let mut key_debounce = 0;

    let mut window = Window::new(
        &format!("CHIP-8 Emulator - speed {} - ESC to exit", speed),
        640,
        320,
        WindowOptions {
            resize: true,
            scale: Scale::FitScreen,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if key_debounce > 0 {
            key_debounce -= 1;
        } else {
            if window.is_key_down(Key::F11) && speed > MIN_SPEED {
                speed -= 1;
                key_debounce = KEY_DEBOUNCE;
                window.set_title(&format!("CHIP-8 Emulator - speed {} - ESC to exit", speed));
            }

            if window.is_key_down(Key::F12) && speed < MAX_SPEED {
                speed += 1;
                window.set_title(&format!("CHIP-8 Emulator - speed {} - ESC to exit", speed));
                key_debounce = KEY_DEBOUNCE;
            }

            if window.is_key_down(Key::F4) {
                cpu.toggle_debug();
                println!("Debug toggled");
                key_debounce = KEY_DEBOUNCE;
            }
        }

        if window.is_key_down(Key::F2) {
            cpu.restart();
        }

        for _ in 0..speed {
            cpu.reset_keys();
            window.get_keys().map(|keys| {
                for t in keys {
                    match t {
                        Key::X => cpu.set_key(0),
                        Key::Key1 => cpu.set_key(1),
                        Key::Key2 => cpu.set_key(2),
                        Key::Key3 => cpu.set_key(3),
                        Key::Q => cpu.set_key(4),
                        Key::W => cpu.set_key(5),
                        Key::E => cpu.set_key(6),
                        Key::A => cpu.set_key(7),
                        Key::S => cpu.set_key(8),
                        Key::D => cpu.set_key(9),
                        Key::Z => cpu.set_key(0xA),
                        Key::C => cpu.set_key(0xB),
                        Key::Key4 => cpu.set_key(0xC),
                        Key::R => cpu.set_key(0xD),
                        Key::F => cpu.set_key(0xE),
                        Key::V => cpu.set_key(0xF),
                        _ => (),
                    }
                }
            });
            cpu.steps(1);
        }

        if cpu.draw_flag {
            convert_graphics(&mut cpu, &mut buffer);
            cpu.draw_done();
            window.update_with_buffer(&buffer, 64, 32).unwrap();
        } else {
            window.update();
        }
    }
}

fn convert_graphics(cpu: &mut Cpu, buffer: &mut Vec<u32>) {
    for y in 0..32 {
        for x in 0..64 {
            buffer[y * 64 + x] = if cpu.graphics[y as usize][x as usize] {
                0x004e6563
            } else {
                0x00a0a293
            }
        }
    }
}
