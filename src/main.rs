//TODO(remove)
#![allow(dead_code)]

use std::io::{File, BufWriter, Reader};
use error::Ch8Error;

const RAM_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;

mod error;

struct Vm {
    reg: [u8; 16],
    index: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 256],
    ram: [u8; RAM_SIZE],

    timer: u8,
    tone: u8,

    screen: [u8; 64 * 32],
    keys: [u8; 16],
}

impl Vm {
    fn new() -> Vm {
        Vm {
            reg: [0; 16],
            index: 0,
            pc: PROGRAM_START as u16,
            sp: 0,
            stack: [0; 256],
            ram: [0; RAM_SIZE],

            timer: 0,
            tone: 0,

            screen: [0; 64 * 32],
            keys: [0; 16],
        }
    }

    fn load_rom(&mut self, reader: &mut Reader) -> Result<usize, Ch8Error> {
        let rom = try!(reader.read_to_end());
        if rom.len() > (RAM_SIZE - PROGRAM_START) {
           return Err(Ch8Error::Io("Rom was larger than available RAM", None))
        }
        let mut ram = BufWriter::new(&mut self.ram[PROGRAM_START..RAM_SIZE]);
        try!(ram.write(rom.as_slice()));
        return Ok(rom.len());
    }

    fn dump_ram(&self, writer: &mut Writer) {
        writer.write(&self.ram).unwrap();
    }
}

struct Op {
    raw: u16
}

impl Op {
    fn addr(&self) -> Addr {
        self.raw & 0x0FFF
    }

    fn x(&self) -> Vx {
        ((self.raw & 0x0F00) >> 8) as u8
    }

    fn y(&self) -> Vy {
        ((self.raw & 0x00F0) >> 4) as u8
    }

    fn n_high(&self) -> Nibble {
        ((self.raw & 0xF000) >> 12) as u8
    }

    fn n_low(&self) -> Nibble {
        ((self.raw & 0xF000) >> 12) as u8
    }


    fn kk(&self) -> Byte {
        (self.raw & 0x00FF) as u8
    }
}

type Vx = u8;
type Vy = u8;
type Addr = u16;
type Byte = u8;
type Nibble = u8;

#[derive(Show)]
enum Instruction {
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
    SetTone(Vx),            // Fx18 - LD ST, Vx
    AddToI(Vx),             // Fx1E - ADD I, Vx
    LoadHexGlyph(Vx),       // Fx29 - LD F, Vx
    StoreBCD(Vx),           // Fx33 - LD B, Vx
    StoreRegisters(Vx),     // Fx55 - LD [I], Vx
    LoadRegisters(Vx),      // Fx65 - LD Vx, [I]
    Unknown,
}

impl Instruction {
    fn from_op(op: &Op) -> Instruction {
        use Instruction::*;
        match op.n_high() {
            0x0 => {
                match op.kk() {
                    0xE0 => Clear,
                    0xEE => Return,
                    _ => Sys(op.addr())
                }
            },
            0x1 => Jump(op.addr()),
            0x2 => Call(op.addr()),
            0x3 => SkipEqualK(op.x(), op.kk()),
            0x4 => SkipNotEqualK(op.x(), op.kk()),
            _ => Unknown
        }
    }
}

fn main() {
    let mut vm = Vm::new();

    let mut rom_file = File::open(&Path::new("/Users/jakerr/Downloads/IBM Logo.ch8")).unwrap();
    match vm.load_rom(&mut rom_file) {
        Ok(size) => println!("Loaded rom size: {}", size),
        Err(e) => println!("Error loading rom: {}", e)
    }

    let mut dump_file = File::create(&Path::new("/Users/jakerr/tmp/dump.ch8ram")).unwrap();
    vm.dump_ram(&mut dump_file);

    for i in vm.ram.chunks(2) {
        match i {
            [0, 0] => continue,
            [h, l] => {
                let op = Op{raw:((h as u16) << 8) | l as u16};
                println!("raw: 0x{:X}", op.raw);
                println!("instruction: {:?}", Instruction::from_op(&op));
                println!("addr: 0x{:X}", op.addr());
                println!("x: 0x{:X}", op.x());
                println!("y: 0x{:X}", op.y());
                println!("n_high: 0x{:X}", op.n_high());
                println!("n_low: 0x{:X}", op.n_low());
                println!("kk: 0x{:X}\n", op.kk());
            },
            _ => continue
        }
    }
}
