//! Raw and high-level instruction abstractions

use std::num::{FromPrimitive, ToPrimitive};

/// A register index/name
///
/// There are 16 data registers, `V0`..`VF`
#[derive(FromPrimitive, Copy, Debug)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xA,
    VB = 0xB,
    VC = 0xC,
    VD = 0xD,
    VE = 0xE,
    VF = 0xF,
}

impl ToPrimitive for Register {
    fn to_i64(&self) -> Option<i64> {
        Some(*self as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(*self as u64)
    }
}

/// First register in an opcode
pub type Vx = Register;
/// Second register in an opcode
pub type Vy = Register;

/// A byte
///
/// Valid values are within `0x0` .. `0xFF`.
pub type Byte = u8;

/// A nibble (hex digit)
///
/// Valid values are within `0x0` .. `0xF`.
#[derive(Copy, Debug)]
pub struct Nibble {
    pub bits: u8,
}

impl Nibble {
    /// Creates a new `Nibble`, ignoring any high bits
    pub fn new(bits: u8) -> Nibble {
        Nibble { bits: bits & 0x0F }
    }
}

/// Absolute memory address
///
/// Valid addresses are within `0x0` .. `0xFFF`.
#[derive(Copy, Debug)]
pub struct Addr {
    pub bits: u16,
}

impl Addr {
    /// Creates a new `Addr`, ignoring any high bits
    pub fn new(bits: u16) -> Addr {
        Addr { bits: bits & 0x0FFF }
    }
}


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
        Addr::new(self.bits & 0x0FFF)
    }

    /// The *`Vx` register-index* part, i.e. `0xE` is `VE`
    pub fn x(&self) -> Vx {
        FromPrimitive::from_u8(((self.bits & 0x0F00) >> 8) as u8).unwrap()
    }

    /// The *`Vy` register-index* part, i.e. `0xE` is `VE`
    pub fn y(&self) -> Vy {
        FromPrimitive::from_u8(((self.bits & 0x00F0) >> 4) as u8).unwrap()
    }

    /// The *high nibble* part
    pub fn n_high(&self) -> Nibble {
        Nibble::new(((self.bits & 0xF000) >> 12) as u8)
    }

    /// The *low nibble* part
    pub fn n_low(&self) -> Nibble {
        Nibble::new((self.bits & 0x000F) as u8)
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
    /// Jumps to machine subroutine at `Addr`.
    ///
    /// Note that this is intentionally not implemented in the `Vm`.
    Sys(Addr),              // 0nnn - SYS addr
    /// Clears the screen.
    ///
    /// Sets all pixels to the unlit state.
    Clear,                  // 00E0 - CLS
    /// Returns from a subroutine
    Return,                 // 00EE - RET
    /// Jumps to `Addr`
    Jump(Addr),             // 1nnn - JP addr
    /// Calls subroutine at `Addr`
    Call(Addr),             // 2nnn - CALL addr
    /// Skips the next instructions if `Vx` equals `Byte`
    SkipEqualK(Vx, Byte),   // 3xkk - SE Vx, byte
    /// Skips the next instruction if `Vx` does not equal `Byte`
    SkipNotEqualK(Vx, Byte),// 4xkk - SNE Vx, byte
    /// Skips the next instruction if `Vx` and `Vy` are equal
    SkipEqual(Vx, Vy),      // 5xy0 - SE Vx, Vy
    /// Sets `Vy` to `Byte`
    SetK(Vx, Byte),         // 6xkk - LD Vx, byte
    /// Adds `Byte` to `Vx`, then stores the result in `Vx`
    AddK(Vx, Byte),         // 7xkk - ADD Vx, byte
    /// Stores `Vy` in `Vx`
    Set(Vx, Vy),            // 8xy0 - LD Vx, Vy
    /// Performs a bitwise OR (`|`) of `Vx` and `Vy`, then stores the result in `Vx`
    Or(Vx, Vy),             // 8xy1 - OR Vx, Vy
    /// Performs a bitwise AND (`&`) of `Vx` and `Vy`, then stores the result in `Vx`
    And(Vx, Vy),            // 8xy2 - AND Vx, Vy
    /// Performs a bitwise XOR (`^`) of `Vx` and `Vy`, then stores the result in `Vx`
    XOr(Vx, Vy),            // 8xy3 - XOR Vx, Vy
    /// Adds `Vx` and `Vy`, then stores the result in in `Vx`.
    ///
    /// `VF` is set to `1` on overflow, `0` otherwise.
    Add(Vx, Vy),            // 8xy4 - ADD Vx, Vy
    /// Subtracts `Vy` from `Vx`, then stores the result in `Vx`.
    ///
    /// `VF` is set to `1` if `Vx` is larger than `Vy` prior subtraction, `0` otherwise.
    Sub(Vx, Vy),            // 8xy5 - SUB Vx, Vy
    /// Shifts `Vy` right by one bit, then stores the result in `Vx`.
    ///
    /// Stores the least-significant bit prior shift of `Vy` in `VF`.
    ShiftRight(Vx, Vy),     // 8xy6 - SHR Vx {, Vy}
    /// Subtracts `Vx` from `Vy`, then stores the result in `Vx`.
    ///
    /// `VF` is set to `1` if `Vx` is larger than `Vy` prior subtraction, `0` otherwise.
    ///
    /// Note that this is the same as `Sub` with inverted register operands.
    SubInv(Vx, Vy),         // 8xy7 - SUBN Vx, Vy
    /// Shifts `Vy` left by one bit, then stores the result in `Vx`.
    ///
    /// Stores the most-significant bit prior shift of `Vy` in `VF`.
    ShiftLeft(Vx, Vy),      // 8xyE - SHL Vx {, Vy}
    /// Skips the next instruction if `Vx` and `Vy` are not equal
    SkipNotEqual(Vx, Vy),   // 9xy0 - SNE Vx, Vy
    /// Sets the `I` register to `Addr`
    LoadI(Addr),            // Annn - LD I, addr
    /// Jumps to `V0 + Addr`
    LongJump(Addr),         // Bnnn - JP V0, addr
    /// Sets `Vx` to a random byte ANDed with `Byte`
    Rand(Vx, Byte),         // Cxkk - RND Vx, byte
    /// Draws the sprite with `Nibble` bytes of data from the `I` register at position `(Vx, Vy)`.
    ///
    /// Sets `VF` to `1` if any pixels are set to unlit state, `0` otherwise.
    ///
    /// Note that sprites wrap around onto the opposite side of the screen.
    Draw(Vx, Vy, Nibble),   // Dxyn - DRW Vx, Vy, nibble
    /// Skips the next instruction if key `Vx` is pressed
    SkipPressed(Vx),        // Ex9E - SKP Vx
    /// Skips the next instruction if key `Vx` is not pressed
    SkipNotPressed(Vx),     // ExA1 - SKNP Vx
    /// Stores the value of the `delay timer` in `Vx`
    GetTimer(Vx),           // Fx07 - LD Vx, DT
    /// Stops execution until a key is pressed, then stores that key in `Vx`
    WaitKey(Vx),            // Fx0A - LD Vx, K
    /// Sets the `delay timer` to `Vx`
    SetTimer(Vx),           // Fx15 - LD DT, Vx
    /// Sets the `sound timer` to `Vx`
    SetSoundTimer(Vx),      // Fx18 - LD ST, Vx
    /// Adds `Vx` and the `I` register, then stores the result in `I`
    AddToI(Vx),             // Fx1E - ADD I, Vx
    /// Stores the address of the hexadecimal digit `Vx` in the `I` register
    LoadHexGlyph(Vx),       // Fx29 - LD F, Vx
    /// Stores the binary-coded decimal representation of `Vx` at address `I`, `I + 1` and `I + 2`
    StoreBCD(Vx),           // Fx33 - LD B, Vx
    /// Stores the registers `V0` to `Vx` inclusive at address `I`.
    ///
    /// Register `I` is set to `I + Vx + 1` afterwards.
    StoreRegisters(Vx),     // Fx55 - LD [I], Vx
    /// Reads the registers `V0` to `Vx` inclusive from address `I`.
    ///
    /// Register `I` is set to `I + Vx + 1` afterwards.
    LoadRegisters(Vx),      // Fx65 - LD Vx, [I]
    /// Placeholder for an unknown or illegal instruction.
    ///
    /// Note that this is not a real CHIP-8 instruction.
    Unknown,
}

impl Instruction {
    /// Creates a new instruction from raw bits,
    /// or `Instruction::Unknown` if no valid match could be found
    pub fn from_raw(raw: &RawInstruction) -> Instruction {
        use self::Instruction::*;

        match raw.n_high().bits {
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
                match raw.n_low().bits {
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
