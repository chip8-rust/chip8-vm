//! Raw and high-level instruction abstractions

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


/// Raw instruction
///
/// Helper around the raw bits, not necessarily a valid instruction.
#[derive(Copy)]
pub struct RawInstruction {
    bits: u16
}

impl RawInstruction {
    /// Creates a new raw instruction without any checks of the `bits`
    pub fn new(bits: u16) -> RawInstruction {
        RawInstruction{ bits: bits }
    }

    /// The *raw bits*
    #[allow(dead_code)]
    pub fn bits(&self) -> u16 {
        self.bits
    }

    /// The *address bits* part
    pub fn addr(&self) -> Addr {
        self.bits & 0x0FFF
    }

    /// The *`Vx` register-index* part, i.e. `0xE` is `VE`
    pub fn x(&self) -> Vx {
        ((self.bits & 0x0F00) >> 8) as u8
    }

    /// The *`Vy` register-index* part, i.e. `0xE` is `VE`
    pub fn y(&self) -> Vy {
        ((self.bits & 0x00F0) >> 4) as u8
    }

    /// The *high nibble* part
    pub fn n_high(&self) -> Nibble {
        ((self.bits & 0xF000) >> 12) as u8
    }

    /// The *low nibble* part
    pub fn n_low(&self) -> Nibble {
        (self.bits & 0x000F) as u8
    }

    /// The *`k` byte* part
    pub fn k(&self) -> Byte {
        (self.bits & 0x00FF) as u8
    }
}

/// High-level instruction
///
/// A valid instruction that can be executed as-is.
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
    /// Creates a new instruction from raw bits,
    /// or `Instruction::Unknown` if no valid match could be found
    pub fn from_raw(raw: &RawInstruction) -> Instruction {
        use self::Instruction::*;

        match raw.n_high() {
            0x0 => {
                match raw.k() {
                    0xE0 => Clear,
                    0xEE => Return,
                    _ => Sys(raw.addr())
                }
            },
            0x1 => Jump(raw.addr()),
            0x2 => Call(raw.addr()),
            0x3 => SkipEqualK(raw.x(), raw.k()),
            0x4 => SkipNotEqualK(raw.x(), raw.k()),
            0x5 => SkipEqual(raw.x(), raw.y()),
            0x6 => SetK(raw.x(), raw.k()),
            0x7 => AddK(raw.x(), raw.k()),
            0x8 => {
                match raw.n_low() {
                    0x0 => Set(raw.x(), raw.y()),
                    0x1 => Or(raw.x(), raw.y()),
                    0x2 => And(raw.x(), raw.y()),
                    0x3 => XOr(raw.x(), raw.y()),
                    0x4 => Add(raw.x(), raw.y()),
                    0x5 => Sub(raw.x(), raw.y()),
                    0x6 => ShiftRight(raw.x(), raw.y()),
                    0x7 => SubInv(raw.x(), raw.y()),
                    0xE => ShiftLeft(raw.x(), raw.y()),
                    _ => Unknown
                }
            },
            0x9 => SkipNotEqual(raw.x(), raw.y()),
            0xA => LoadI(raw.addr()),
            0xB => LongJump(raw.addr()),
            0xC => Rand(raw.x(), raw.k()),
            0xD => Draw(raw.x(), raw.y(), raw.n_low()),
            0xE => {
                match raw.k() {
                    0x9E => SkipPressed(raw.x()),
                    0xA1 => SkipNotPressed(raw.x()),
                    _ => Unknown,
                }
            },
            0xF => {
                match raw.k() {
                    0x07 => GetTimer(raw.x()),
                    0x0A => WaitKey(raw.x()),
                    0x15 => SetTimer(raw.x()),
                    0x18 => SetSoundTimer(raw.x()),
                    0x1E => AddToI(raw.x()),
                    0x29 => LoadHexGlyph(raw.x()),
                    0x33 => StoreBCD(raw.x()),
                    0x55 => StoreRegisters(raw.x()),
                    0x65 => LoadRegisters(raw.x()),
                    _ => Unknown,
                }
            }
            _ => Unknown
        }
    }
}
