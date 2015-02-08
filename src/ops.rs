//! Opcode and instruction abstractions

/// First register in an opcode
///
/// There are 16 registers, `V0`..`VF`
pub type Vx = u8;
/// Second register in an opcode
pub type Vy = u8;
/// Absolute memory address
///
/// Valid addresses are within `0x0` .. `0xFFF`.
pub type Addr = u16;
/// A byte
///
/// Valid values are within `0x0` .. `0xFF`.
pub type Byte = u8;
/// A nibble (hex digit)
///
/// Valid values are within `0x0` .. `0xF`.
pub type Nibble = u8;


/// Assembly opcode
#[derive(Copy)]
pub struct Op {
    raw: u16
}

impl Op {
    pub fn new(raw: u16) -> Op {
        Op{ raw: raw }
    }

    #[allow(dead_code)]
    pub fn raw(&self) -> u16 {
        self.raw
    }

    pub fn addr(&self) -> Addr {
        self.raw & 0x0FFF
    }

    pub fn x(&self) -> Vx {
        ((self.raw & 0x0F00) >> 8) as u8
    }

    pub fn y(&self) -> Vy {
        ((self.raw & 0x00F0) >> 4) as u8
    }

    pub fn n_high(&self) -> Nibble {
        ((self.raw & 0xF000) >> 12) as u8
    }

    pub fn n_low(&self) -> Nibble {
        (self.raw & 0x000F) as u8
    }


    pub fn k(&self) -> Byte {
        (self.raw & 0x00FF) as u8
    }
}

/// Machine instruction
#[derive(Copy,Debug)]
pub enum Instruction {
    Sys(Addr),              // 0nnn - SYS addr
    Clear,                  // 00E0 - CLS
    Return,                 // 00EE - RET
    Jump(Addr),             // 1nnn - JP addr
    Call(Addr),             // 2nnn - CALL addr
    SkipEqualK(Vx, Byte),   // 3xkk - SE Vx, byte
    SkipNotEqualK(Vx, Byte),// 4xkk - SNE Vx, byte
    SkipEqual(Vx, Vy),      // 5xy0 - SE Vx, Vy
    SetK(Vx, Byte),         // 6xkk - LD Vx, byte
    AddK(Vx, Byte),         // 7xkk - ADD Vx, byte
    Set(Vx, Vy),            // 8xy0 - LD Vx, Vy
    Or(Vx, Vy),             // 8xy1 - OR Vx, Vy
    And(Vx, Vy),            // 8xy2 - AND Vx, Vy
    XOr(Vx, Vy),            // 8xy3 - XOR Vx, Vy
    Add(Vx, Vy),            // 8xy4 - ADD Vx, Vy
    Sub(Vx, Vy),            // 8xy5 - SUB Vx, Vy
    ShiftRight(Vx, Vy),     // 8xy6 - SHR Vx {, Vy}
    SubInv(Vx, Vy),         // 8xy7 - SUBN Vx, Vy
    ShiftLeft(Vx, Vy),      // 8xyE - SHL Vx {, Vy}
    SkipNotEqual(Vx, Vy),   // 9xy0 - SNE Vx, Vy
    LoadI(Addr),            // Annn - LD I, addr
    LongJump(Addr),         // Bnnn - JP V0, addr
    Rand(Vx, Byte),         // Cxkk - RND Vx, byte
    Draw(Vx, Vy, Nibble),   // Dxyn - DRW Vx, Vy, nibble
    SkipPressed(Vx),        // Ex9E - SKP Vx
    SkipNotPressed(Vx),     // ExA1 - SKNP Vx
    GetTimer(Vx),           // Fx07 - LD Vx, DT
    WaitKey(Vx),            // Fx0A - LD Vx, K
    SetTimer(Vx),           // Fx15 - LD DT, Vx
    SetSoundTimer(Vx),      // Fx18 - LD ST, Vx
    AddToI(Vx),             // Fx1E - ADD I, Vx
    LoadHexGlyph(Vx),       // Fx29 - LD F, Vx
    StoreBCD(Vx),           // Fx33 - LD B, Vx
    StoreRegisters(Vx),     // Fx55 - LD [I], Vx
    LoadRegisters(Vx),      // Fx65 - LD Vx, [I]
    Unknown,
}

impl Instruction {
    pub fn from_op(op: &Op) -> Instruction {
        use ops::Instruction::*;
        match op.n_high() {
            0x0 => {
                match op.k() {
                    0xE0 => Clear,
                    0xEE => Return,
                    _ => Sys(op.addr())
                }
            },
            0x1 => Jump(op.addr()),
            0x2 => Call(op.addr()),
            0x3 => SkipEqualK(op.x(), op.k()),
            0x4 => SkipNotEqualK(op.x(), op.k()),
            0x5 => SkipEqual(op.x(), op.y()),
            0x6 => SetK(op.x(), op.k()),
            0x7 => AddK(op.x(), op.k()),
            0x8 => {
                match op.n_low() {
                    0x0 => Set(op.x(), op.y()),
                    0x1 => Or(op.x(), op.y()),
                    0x2 => And(op.x(), op.y()),
                    0x3 => XOr(op.x(), op.y()),
                    0x4 => Add(op.x(), op.y()),
                    0x5 => Sub(op.x(), op.y()),
                    0x6 => ShiftRight(op.x(), op.y()),
                    0x7 => SubInv(op.x(), op.y()),
                    0xE => ShiftLeft(op.x(), op.y()),
                    _ => Unknown
                }
            },
            0x9 => SkipNotEqual(op.x(), op.y()),
            0xA => LoadI(op.addr()),
            0xB => LongJump(op.addr()),
            0xC => Rand(op.x(), op.k()),
            0xD => Draw(op.x(), op.y(), op.n_low()),
            0xE => {
                match op.k() {
                    0x9E => SkipPressed(op.x()),
                    0xA1 => SkipNotPressed(op.x()),
                    _ => Unknown,
                }
            },
            0xF => {
                match op.k() {
                    0x07 => GetTimer(op.x()),
                    0x0A => WaitKey(op.x()),
                    0x15 => SetTimer(op.x()),
                    0x18 => SetSoundTimer(op.x()),
                    0x1E => AddToI(op.x()),
                    0x29 => LoadHexGlyph(op.x()),
                    0x33 => StoreBCD(op.x()),
                    0x55 => StoreRegisters(op.x()),
                    0x65 => LoadRegisters(op.x()),
                    _ => Unknown,
                }
            }
            _ => Unknown
        }
    }
}
