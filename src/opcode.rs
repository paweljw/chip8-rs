use std::fmt;

pub struct Opcode {
    pub opcode: u16,
}

impl Opcode {
    pub fn new(opcode: u16) -> Opcode {
        Opcode { opcode }
    }

    pub fn x(&self) -> u8 {
        ((self.opcode & 0x0f00) >> 8) as u8
    }

    pub fn y(&self) -> u8 {
        ((self.opcode & 0x00f0) >> 4) as u8
    }

    pub fn n(&self) -> u8 {
        (self.opcode & 0x000f) as u8
    }

    pub fn kk(&self) -> u8 {
        (self.opcode & 0x00ff) as u8
    }

    pub fn nnn(&self) -> u16 {
        self.opcode & 0x0fff
    }

    pub fn t(&self) -> u16 {
        self.opcode & 0xf000
    }

    fn mnemonic(&self) -> String {
        return match self.t() {
            0x0000 => match self.opcode {
                0x00e0 => String::from("CLR"),
                0x00ee => String::from("RTS"),
                _ => format!("SYS {}", self.nnn()),
            },
            0x1000 => format!("JUMP  {:#06x}", self.nnn()),
            0x2000 => format!("CALL  {:#06x}", self.nnn()),
            0x3000 => format!("SKE   V{:#}, {:#06x}", self.x(), self.kk()),
            0x4000 => format!("SKNE  V{:#}, {:#06x}", self.x(), self.kk()),
            0x5000 => format!("SKRE  V{:#}, V{:#}", self.x(), self.y()),
            0x6000 => format!("LOAD  V{:#}, {:#06x}", self.x(), self.kk()),
            0x7000 => format!("ADD   V{:#}, {:#06x}", self.x(), self.kk()),
            0x8000 => match self.n() {
                0x0 => format!("MOVE  V{:#}, V{:#}", self.x(), self.y()),
                0x1 => format!("OR    V{:#}, V{:#}", self.x(), self.y()),
                0x2 => format!("AND   V{:#}, V{:#}", self.x(), self.y()),
                0x3 => format!("XOR   V{:#}, V{:#}", self.x(), self.y()),
                0x4 => format!("ADDR  V{:#}, V{:#}", self.x(), self.y()),
                0x5 => format!("SUB   V{:#}, V{:#}", self.x(), self.y()),
                0x6 => format!("SHR   V{:#}, V{:#}", self.x(), self.y()),
                0x7 => format!("SSUB  V{:#}, V{:#}", self.x(), self.y()),
                0xE => format!("SHL   V{:#}, V{:#}", self.x(), self.y()),
                _ => String::from("UNUS"),
            },
            0x9000 => format!("SKRNE V{:#}, {:#06x}", self.x(), self.y()),
            0xA000 => format!("LOADI {:#06x}", self.nnn()),
            0xB000 => format!("JUMPI {:#06x}", self.nnn()),
            0xC000 => format!("RAND  V{:#}, {:#06x}", self.x(), self.kk()),
            0xD000 => format!("DRAW  V{:#}, V{:#}, {:#06x}", self.x(), self.y(), self.n()),
            0xE000 => match self.kk() {
                0x9E => format!("SKPR  V{:#}", self.x()),
                0xA1 => format!("SKUP  V{:#}", self.x()),
                _ => String::from("UNUS"),
            },
            0xF000 => match self.kk() {
                0x07 => format!("MOVED V{:#}", self.x()),
                0x0A => format!("KEYD  V{:#}", self.x()),
                0x15 => format!("LOADD V{:#}", self.x()),
                0x18 => format!("LOADS V{:#}", self.x()),
                0x1E => format!("ADDI  V{:#}", self.x()),
                0x29 => format!("LDSPR V{:#}", self.x()),
                0x33 => format!("BCD   V{:#}", self.x()),
                0x55 => format!("STOR  V{:#}", self.x()),
                0x65 => format!("READ  V{:#}", self.x()),
                _ => String::from("UNUS"),
            },
            _ => String::from("UNUS"),
        };
    }
}

impl fmt::LowerHex for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#06x}", self.opcode)
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#06x}\t{}", self.opcode, self.mnemonic())
    }
}
