# chip8-rs

An oversimplified and extremely incomplete CHIP-8 emulator.

## TODO

- [x] Support all available demos (more complete opcode emulation)
- [x] Support keyboard
- [x] Properly debounce keyboard (at 10x speed, bounces from hell)
- [x] Support sound (actually supported now, hell hath frozen over)

## Usage

```
$ cargo run --release roms\demos\some_rom.ch8
```

* F2 - restart emulation
* F4 - toggle debug (see console)
* F11 - emulation speed down
* F12 - emulation speed up
