//! Virtual machine implementation

extern crate rand;

use std::io::{Read, Write, BufWriter};
use error::Chip8Error;
use instructions::Register;
use instructions::{RawInstruction, Instruction};
use std::slice::Chunks;

use rand::Rng;

/// Size of the RAM in bytes
const RAM_SIZE: usize = 4096;
/// Depth of the stack
const STACK_SIZE: usize = 256;
/// Number of data registers, i.e. `V0` .. `VF`
const NUM_DATA_REGISTERS: usize = 16;
/// Memory address for programm (ROM) start
const PROGRAM_START: usize = 0x200;
/// CPU clock speed
const CLOCK_HZ: f32 = 600.0;

/// Memory address of built-in font sprites
const FONT_ADDR: usize = 0;
/// Number of rows in one font sprite
const FONT_HEIGHT: usize = 5;
/// Size of one font sprite
const FONT_BYTES: usize = FONT_HEIGHT * 16;
/// Data of the built-in font
const FONT: [u8; FONT_BYTES] = [
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
/// Width of the screen in pixels
const SCREEN_WIDTH: usize = 64;
/// Height of the screen in pixels
const SCREEN_HEIGHT: usize = 32;
/// Total number of pixels of the screen
const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Number of keys on the keypad
const NUM_KEYS: usize = 16;

/// Virtual machine
///
/// The virtual machine manages state like its registers,
/// the RAM, stack, screen pixels, pressed keys as well as
/// timers and some internal state.
// This would require "impl<T> Clone for [T; ARBITRARY_CONSTANT] where T: Copy"
// One can probably do this with a macro, but for now I'm too lazy.
//#[derive(Clone, Copy)]
pub struct Vm {
    reg: [u8; NUM_DATA_REGISTERS],
    i: usize,
    pc: usize,
    sp: usize,
    stack: [usize; STACK_SIZE],
    ram: [u8; RAM_SIZE],

    timer: u8,
    t_tick: f32,

    sound_timer: u8,
    st_tick: f32,

    screen: [u8; SCREEN_PIXELS],
    keys: [u8; NUM_KEYS],
    waiting_on_key: Option<Register>,
}

impl Vm {
    /// Creates a new `Vm` instance with default state
    pub fn new() -> Vm {
        let mut vm = Vm {
            reg: [0; NUM_DATA_REGISTERS],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            stack: [0; STACK_SIZE],
            ram: [0; RAM_SIZE],

            timer: 0,
            t_tick: 0.0,

            sound_timer: 0,
            st_tick: 0.0,

            screen: [0; SCREEN_PIXELS],
            keys: [0; NUM_KEYS],
            waiting_on_key: None,
        };
        {
            let mut ram = BufWriter::new(&mut vm.ram[FONT_ADDR..(FONT_ADDR + FONT_BYTES)]);
            ram.write_all(FONT.as_ref()).unwrap();
            debug!("Initialized VM with built-in font");
        }
        vm
    }

    /// Loads the ROM contents from `reader` into RAM at the program start address
    pub fn load_rom(&mut self, reader: &mut Read) -> Result<usize, Chip8Error> {
        let mut rom = Vec::new();
        try!(reader.read_to_end(&mut rom));
        let rom_len = rom.len();
        let available_ram = RAM_SIZE - PROGRAM_START;
        if rom_len > available_ram {
            error!("ROM size ({}) is larger than available RAM ({})!", rom_len, available_ram);
            return Err(Chip8Error::Io("ROM was larger than available RAM", None))
        }
        // TODO: ROM needs to contain at least one instruction to be valid
        let mut ram = BufWriter::new(&mut self.ram[PROGRAM_START..RAM_SIZE]);
        try!(ram.write_all(rom.as_ref()));
        debug!("Loaded ROM of size {}", rom_len);
        return Ok(rom_len);
    }

    #[allow(dead_code)]
    pub fn dump_ram(&self, writer: &mut Write) {
        writer.write_all(&self.ram).unwrap();
    }

    /// Returns `True` if the sound timer is active
    pub fn beeping(&self) -> bool {
        self.sound_timer > 0
    }

    /// Marks the key with index `idx` as being set
    pub fn set_key(&mut self, idx: u8) {
        debug!("Set key {}", idx);
        self.keys[idx as usize] = 1;
        if let Some(vx) = self.waiting_on_key {
            debug!("No longer waiting on key");
            self.reg[vx as usize] = idx;
            self.waiting_on_key = None;
        }
    }

    /// Marks they key with index `idx` as being unset
    pub fn unset_key(&mut self, idx: u8) {
        debug!("Unset key {}", idx);
        self.keys[idx as usize] = 0;
    }

    fn exec(&mut self, ins: &Instruction) -> bool {
        use instructions::Instruction::*;

        match *ins {
            // Sys(addr) intentionally left unimplemented.

            Clear => {
                for b in self.screen.iter_mut() {
                    *b = 0;
                }
            },
            Return => {
                self.pc = self.stack[self.sp];
                self.sp-=1;
            },
            Jump(addr) => {
                let idle = self.pc-2 == addr.bits as usize;
                self.pc = addr.bits as usize;
                if idle { return true; }
            }
            Call(addr) => {
                self.sp+=1;
                self.stack[self.sp] = self.pc;
                self.pc = addr.bits as usize;
            },
            SkipEqualK(vx, k) => {
                if self.reg[vx as usize] == k {
                    self.pc += 2;
                }
            },
            SkipNotEqualK(vx, k) => {
                if self.reg[vx as usize] != k {
                    self.pc += 2;
                }
            },
            SkipEqual(vx, vy) => {
                let x = self.reg[vx as usize];
                let y = self.reg[vy as usize];
                if x == y {
                    self.pc += 2;
                }
            },
            SetK(vx, byte) => {
                self.reg[vx as usize] = byte;
            },
            AddK(vx, byte) => {
                self.reg[vx as usize] = self.reg[vx as usize].wrapping_add(byte);
            },
            Set(vx, vy) => self.reg[vx as usize] = self.reg[vy as usize],
            Or(vx, vy)  => self.reg[vx as usize] |= self.reg[vy as usize],
            And(vx, vy) => self.reg[vx as usize] &= self.reg[vy as usize],
            XOr(vx, vy) => self.reg[vx as usize] ^= self.reg[vy as usize],
            Add(vx, vy) => {
                let x = self.reg[vx as usize] as u16;
                let y = self.reg[vy as usize] as u16;
                let res = x + y;

                // VF is carryover
                self.reg[Register::VF as usize] = (res > 255) as u8;

                self.reg[vx as usize] = res as u8;
            },
            Sub(vx, vy) => {
                let x = self.reg[vx as usize];
                let y = self.reg[vy as usize];

                // VF is Not Borrow i.e. x > y
                self.reg[Register::VF as usize] = (x > y) as u8;

                self.reg[vx as usize] = x.wrapping_sub(y);
            },
            ShiftRight(vx, vy) => {
                let y = self.reg[vy as usize];

                // VF is lsb before shift
                self.reg[Register::VF as usize] = 0x1 & y;

                self.reg[vx as usize] = y >> 1;
            },
            SubInv(vx, vy) => {
                let x = self.reg[vx as usize];
                let y = self.reg[vy as usize];

                // VF is Not Borrow i.e. y > x
                self.reg[Register::VF as usize] = (y > x) as u8;

                self.reg[vx as usize] = y.wrapping_sub(x);
            },
            ShiftLeft(vx, vy) => {
                let y = self.reg[vy as usize];

                // VF is msb before shift
                self.reg[Register::VF as usize] = y >> 7;

                self.reg[vx as usize] = y << 1;
            }
            SkipNotEqual(vx, vy) => {
                let x = self.reg[vx as usize];
                let y = self.reg[vy as usize];
                if x != y {
                    self.pc += 2;
                }
            },
            LoadI(addr) => {
                self.i = addr.bits as usize;
            },
            LongJump(addr) => {
                self.pc = (self.reg[Register::V0 as usize] as u16 + addr.bits) as usize;
            },
            Rand(vx, byte) => {
                self.reg[vx as usize] = rand::thread_rng().gen::<u8>() & byte;
            }
            Draw(vx, vy, n) => {
                let x = self.reg[vx as usize] as usize;
                let y = self.reg[vy as usize] as usize;
                let i = self.i;
                let n = n.bits as usize;

                let sprite = &self.ram[i..i+n];

                self.reg[Register::VF as usize] = 0;
                for (sy, byte) in sprite.iter().enumerate() {
                    let dy = (y + sy) % SCREEN_HEIGHT;
                    for sx in 0usize..8 {
                        let px = (*byte >> (7 - sx)) & 0b00000001;
                        let dx = (x + sx) % SCREEN_WIDTH;
                        let idx = dy * SCREEN_WIDTH + dx;
                        self.screen[idx] ^= px;

                        // Vf is if there was a collision
                        self.reg[Register::VF as usize] |= (self.screen[idx] == 0 && px == 1) as u8;
                    }
                }
            },
            SkipPressed(vx) => {
                let idx = self.reg[vx as usize];
                if self.keys[idx as usize] == 1 {
                    self.pc += 2;
                }
            }
            SkipNotPressed(vx) => {
                let idx = self.reg[vx as usize];
                if self.keys[idx  as usize] != 1 {
                    self.pc += 2;
                }
            }
            GetTimer(vx) => {
                self.reg[vx as usize] = self.timer;
            },
            WaitKey(vx) => {
                self.waiting_on_key = Some(vx);
            },
            SetTimer(vx) => {
                self.timer = self.reg[vx as usize];
                self.t_tick = 1.0 / 60.0;
            },
            SetSoundTimer(vx) => {
                self.sound_timer = self.reg[vx as usize];
                self.st_tick = 1.0 / 60.0;
            },
            AddToI(vx) => {
                self.i += self.reg[vx as usize] as usize;
            },
            LoadHexGlyph(vx) => {
                let x = self.reg[vx as usize];
                self.i = FONT_ADDR + x as usize * FONT_HEIGHT;
            }
            StoreBCD(vx) => {
                let mut x = self.reg[vx as usize];

                let mut place = 100;
                for i in 0usize..3 {
                    let bcd = x / place;
                    self.ram[self.i + i] = bcd;
                    x -= bcd * place;
                    place /= 10;
                }
            }
            StoreRegisters(vx) => {
                let vx = vx as usize;
                let i = self.i;

                let mut dst = &mut self.ram[i..i+vx+1];
                for (x,b) in dst.iter_mut().enumerate() {
                    *b = self.reg[x];
                }
                self.i += vx+1;
            },
            LoadRegisters(vx) => {
                let vx = vx as usize;
                let i = self.i;

                let src = &self.ram[i..i+vx+1];
                for (x,b) in src.iter().enumerate() {
                    self.reg[x] = *b;
                }
                self.i += vx+1;
            },
            ref other => {
                debug!("Instruction not implemented {:?} skipping...", other)
            }
        }
        return false;
    }

    fn time_step(&mut self, dt:f32) {
        if self.timer > 0 {
            self.t_tick -= dt;
            if self.t_tick <= 0.0 {
                self.timer -= 1;
                self.t_tick = 1.0 / 60.0;
            }
        }

        if self.sound_timer > 0 {
            self.st_tick -= dt;
            if self.st_tick <= 0.0 {
                self.sound_timer -= 1;
                self.st_tick = 1.0 / 60.0;
            }
        }
    }

    // dt: Time in seconds since last step
    /// Executes remaining instructions since the last step
    pub fn step(&mut self, dt:f32) {

        let sub_steps = (CLOCK_HZ * dt).round() as usize;
        let ddt = dt / sub_steps as f32;

        for step in 0..sub_steps {
            trace!("Executing step {}/{}", step, sub_steps);
            self.time_step(ddt);
            if self.waiting_on_key.is_some() {
                debug!("Cancel remaining execution steps while waiting for key");
                return;
            }

            let raw = {
                let codes = &self.ram[self.pc..self.pc+2];
                ((codes[0] as u16) << 8) | codes[1] as u16
            };
            let raw_ins = RawInstruction::new(raw);
            self.pc += 2;
            self.exec(&Instruction::from_raw(&raw_ins));
        }
    }

    /// Returns the pixel rows of the screen
    pub fn screen_rows<'a>(&'a self) -> Chunks<'a, u8> {
        self.screen.chunks(SCREEN_WIDTH)
    }

    /// Prints the current screen pixels to `stdout`
    #[allow(dead_code)]
    pub fn print_screen(&self) {
        for row in self.screen.chunks(SCREEN_WIDTH) {
            println!("");
            for byte in row.iter() {
                match *byte {
                    0x0 => print!("░"),
                    _ => print!("▓")
                }
            }
        }
    }

    /// Prints a disassembly of the entire RAM to `stdout`
    #[allow(dead_code)]
    pub fn print_disassembly(&self) {
        for i in self.ram.chunks(2) {
            match i {
                [0, 0] => continue,
                [h, l] => {
                    let raw_ins = RawInstruction::new(((h as u16) << 8) | l as u16);
                    println!("raw bits: 0x{:X}", raw_ins.bits());
                    println!("instruction: {:?}", Instruction::from_raw(&raw_ins));
                    println!("addr: 0x{:X}", raw_ins.addr().bits);
                    println!("x: 0x{:X}", raw_ins.x() as u8);
                    println!("y: 0x{:X}", raw_ins.y() as u8);
                    println!("n_high: 0x{:X}", raw_ins.n_high().bits);
                    println!("n_low: 0x{:X}", raw_ins.n_low().bits);
                    println!("k: 0x{:X}\n", raw_ins.k());
                },
                _ => continue
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use instructions::*;
    use instructions::Register::*;

    macro_rules! reg_test {
        (
            $name:ident {
                before: {$($reg_before:expr => $reg_before_val:expr),+},
                after:  {$($reg_after:expr => $reg_after_val:expr),+},
                overflow: $over:expr,
                ins: $ins:expr
            }
        ) =>
        (
            //let mut vm = Vm::new();
            #[test]
            fn $name() {
                let mut vm = Vm::new();
                $(
                    vm.reg[$reg_before as usize] = $reg_before_val;
                )+
                vm.exec(&$ins);
                $(
                    assert!(vm.reg[$reg_after as usize] == $reg_after_val);
                )+
                assert!(vm.reg[VF as usize] == $over, "overflow was {}, wanted {}", vm.reg[VF as usize], $over);
            }
        )
    }

    // Add
    reg_test!(
        add_vx {
        before: { V2 => 0xFE, V3 => 0x01 },
        after: { V2 => 0xFF, V3 => 0x01 },
        overflow: 0,
        ins: Instruction::Add(V2, V3)
    });

    reg_test!(
        add_vx_overflows {
        before: { V2 => 0xFF, V3 => 0x01 },
        after: { V2 => 0x00, V3 => 0x01 },
        overflow: 1,
        ins: Instruction::Add(V2, V3)
    });

    // AddK
    reg_test!(
        add_k {
        before: { V0 => 0x09 },
        after: { V0 => 0x0B },
        overflow: 0,
        ins: Instruction::AddK(V0, 2)
    });

    reg_test!(
        add_k_overflows {
        before: { V0 => 0xFF },
        after: { V0 => 0x01 },
        overflow: 0, // Un-intuitive but not spec'd to set overflow
        ins: Instruction::AddK(V0, 2)
    });

    // Sub
    reg_test!(
        sub {
        before: { V0 => 0x3, V1 => 0x2 },
        after:  { V0 => 0x1, V1 => 0x2 },
        overflow: 1, // Defined as not-borrowed
        ins: Instruction::Sub(V0, V1)
    });

    reg_test!(
        sub_borrow {
        before: { V0 => 0x3, V1 => 0x5 },
        after:  { V0 => 0xFE, V1 => 0x5 },
        overflow: 0, // Defined as not-borrowed
        ins: Instruction::Sub(V0, V1)
    });

    // SubInv
    reg_test!(
        sub_inv {
        before: { V0 => 0x2, V1 => 0x3 },
        after:  { V0 => 0x1, V1 => 0x3 },
        overflow: 1, // Defined as not-borrowed
        ins: Instruction::SubInv(V0, V1)
    });

    reg_test!(
        sub_inv_borrow {
        before: { V0 => 0x5, V1 => 0x3 },
        after:  { V0 => 0xFE, V1 => 0x3 },
        overflow: 0, // Defined as not-borrowed
        ins: Instruction::SubInv(V0, V1)
    });

    // ShiftLeft
    reg_test!(
        shiftl_vx_vy {
        before: { V2 => 0xBB, V3 => 0x02 },
        after:  { V2 => 0x04, V3 => 0x02 },
        overflow: 0,
        ins: Instruction::ShiftLeft(V2, V3)
    });

    reg_test!(
        shiftl_vx_inplace {
        before: { V2 => 0b0111_0111 },
        after:  { V2 => 0b1110_1110 },
        overflow: 0,
        ins: Instruction::ShiftLeft(V2, V2)
    });

    reg_test!(
        shiftl_vx_inplace_overflow {
        before: { V2 => 0b1111_1111 },
        after:  { V2 => 0b1111_1110 },
        overflow: 1,
        ins: Instruction::ShiftLeft(V2, V2)
    });

    // ShiftRight
    reg_test!(
        shiftr_vx_vy {
        before: { V2 => 0xBB, V3 => 0x04 },
        after:  { V2 => 0x02, V3 => 0x04 },
        overflow: 0,
        ins: Instruction::ShiftRight(V2, V3)
    });

    reg_test!(
        shiftr_vx_inplace {
        before: { V2 => 0b1110_1110 },
        after:  { V2 => 0b0111_0111 },
        overflow: 0,
        ins: Instruction::ShiftRight(V2, V2)
    });

    reg_test!(
        shiftr_vx_inplace_overflow {
        before: { V2 => 0b1111_1111 },
        after:  { V2 => 0b0111_1111 },
        overflow: 1,
        ins: Instruction::ShiftRight(V2, V2)
    });

    #[test]
    fn oversized_rom() {
        use std::io::Cursor;

        let mut vm = Vm::new();
        let rom = vec![0; super::RAM_SIZE + 1];

        assert!(vm.load_rom(&mut Cursor::new(rom)).is_err());
    }
}
