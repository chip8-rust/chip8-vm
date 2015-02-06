extern crate rand;

use std::old_io::{BufWriter, Reader};
// use std::num::Float;
use error::Ch8Error;
use ops::{Op, Instruction};
use std::slice::Chunks;

use rand::Rng;

use std::num::Float;

const RAM_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;
const CLOCK_HZ: f32 = 600.0;

const FONT_ADDR: usize = 0;
const FONT_HEIGHT: usize = 5;
const FONT_BYTES: usize = FONT_HEIGHT * 16;
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

pub struct Vm {
    reg: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    stack: [usize; 256],
    ram: [u8; RAM_SIZE],

    timer: u8,
    t_tick: f32,

    sound_timer: u8,
    st_tick: f32,

    screen: [u8; 64 * 32],
    keys: [u8; 16],
    waiting_on_key: Option<u8>,
}

impl Vm {
    pub fn new() -> Vm {
        let mut vm = Vm {
            reg: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            stack: [0; 256],
            ram: [0; RAM_SIZE],

            timer: 0,
            t_tick: 0.0,

            sound_timer: 0,
            st_tick: 0.0,

            screen: [0; 64 * 32],
            keys: [0; 16],
            waiting_on_key: None,
        };
        {
            let mut ram = BufWriter::new(&mut vm.ram[FONT_ADDR..(FONT_ADDR + FONT_BYTES)]);
            ram.write_all(FONT.as_slice()).unwrap();
        }
        vm
    }

    pub fn load_rom(&mut self, reader: &mut Reader) -> Result<usize, Ch8Error> {
        let rom = try!(reader.read_to_end());
        if rom.len() > (RAM_SIZE - PROGRAM_START) {
            println!("Rom size {}", rom.len());
            return Err(Ch8Error::Io("Rom was larger than available RAM", None))
        }
        let mut ram = BufWriter::new(&mut self.ram[PROGRAM_START..RAM_SIZE]);
        try!(ram.write_all(rom.as_slice()));
        return Ok(rom.len());
    }

    #[allow(dead_code)]
    pub fn dump_ram(&self, writer: &mut Writer) {
        writer.write_all(&self.ram).unwrap();
    }

    pub fn beeping(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn set_key(&mut self, idx: u8) {
        self.keys[idx as usize] = 1;
        if let Some(vx) = self.waiting_on_key {
            self.reg[vx as usize] = idx;
            self.waiting_on_key = None;
        }
    }

    pub fn unset_key(&mut self, idx: u8) {
        self.keys[idx as usize] = 0;
    }

    fn exec(&mut self, ins: &Instruction) -> bool {
        use ops::Instruction::*;
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
                let idle = self.pc-2 == addr as usize;
                self.pc = addr as usize;
                if idle { return true; }
            }
            Call(addr) => {
                self.sp+=1;
                self.stack[self.sp] = self.pc;
                self.pc = addr as usize;
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
                self.reg[vx as usize] += byte;
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
                self.reg[15] = (res > 255) as u8;

                self.reg[vx as usize] = res as u8;
            },
            Sub(vx, vy) => {
                let x = self.reg[vx as usize];
                let y = self.reg[vy as usize];

                // VF is Not Borrow i.e. x > y
                self.reg[15] = (x > y) as u8;

                self.reg[vx as usize] = x - y;
            },
            ShiftRight(vx, vy) => {
                let y = self.reg[vy as usize];

                // VF is lsb before shift
                self.reg[15] = 0x1 & y;

                self.reg[vx as usize] = y >> 1;
            },
            SubInv(vx, vy) => {
                let x = self.reg[vx as usize];
                let y = self.reg[vy as usize];

                // VF is Not Borrow i.e. y > x
                self.reg[15] = (y > x) as u8;

                self.reg[vx as usize] = y - x;
            },
            ShiftLeft(vx, vy) => {
                let y = self.reg[vy as usize];

                // VF is msb before shift
                self.reg[15] = y >> 7;

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
                self.i = addr as usize;
            },
            LongJump(addr) => {
                self.pc = (self.reg[0] as u16 + addr) as usize;
            },
            Rand(vx, byte) => {
                self.reg[vx as usize] = rand::thread_rng().gen::<u8>() & byte;
            }
            Draw(vx, vy, n) => {
                let x = self.reg[vx as usize] as usize;
                let y = self.reg[vy as usize] as usize;
                let i = self.i as usize;
                let n = n as usize;

                let sprite = &self.ram[i..i+n];

                self.reg[15] = 0;
                for (sy, byte) in sprite.iter().enumerate() {
                    let dy = (y + sy) % 32;
                    for sx in 0us..8 {
                        let px = (*byte >> (7 - sx)) & 0b00000001;
                        let dx = (x + sx) % 64;
                        let idx = dy * 64 + dx;
                        self.screen[idx] ^= px;

                        // Vf is if there was a collision
                        self.reg[15] |= (self.screen[idx] == 0 && px == 1) as u8;
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
                for i in 0us..3 {
                    let bcd = x / place;
                    self.ram[self.i + i] = bcd;
                    x -= bcd * place;
                    place /= 10;
                }
            }
            StoreRegisters(vx) => {
                let vx = vx as usize;
                let i = self.i as usize;

                let mut dst = &mut self.ram[i..i+vx+1];
                for (x,b) in dst.iter_mut().enumerate() {
                    *b = self.reg[x];
                }
                self.i += vx+1;
            },
            LoadRegisters(vx) => {
                let vx = vx as usize;
                let i = self.i as usize;

                let src = &self.ram[i..i+vx+1];
                for (x,b) in src.iter().enumerate() {
                    self.reg[x] = *b;
                }
                self.i += vx+1;
            },
            ref other => {
                println!("Instruction not implemented {:?} skipping...", other)
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
    pub fn step(&mut self, dt:f32) {

        let sub_steps = (CLOCK_HZ * dt).round() as usize;
        let ddt = dt / sub_steps as f32;

        for _ in 0..sub_steps {
            self.time_step(ddt);
            if self.waiting_on_key.is_some() {
                return;
            }

            let raw = {
                let codes = &self.ram[self.pc..self.pc+2];
                ((codes[0] as u16) << 8) | codes[1] as u16
            };
            let op = Op::new(raw);
            self.pc += 2;
            self.exec(&Instruction::from_op(&op));
        }
    }

    pub fn screen_rows<'a>(&'a self) -> Chunks<'a, u8> {
        self.screen.chunks(64)
    }

    #[allow(dead_code)]
    pub fn print_screen(&self) {
        for row in self.screen.chunks(64) {
            println!("");
            for byte in row.iter() {
                match *byte {
                    0x0 => print!("░"),
                    _ => print!("▓")
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn print_disassembly(&self) {
        for i in self.ram.chunks(2) {
            match i {
                [0, 0] => continue,
                [h, l] => {
                    let op = Op::new(((h as u16) << 8) | l as u16);
                    println!("raw: 0x{:X}", op.raw());
                    println!("instruction: {:?}", Instruction::from_op(&op));
                    println!("addr: 0x{:X}", op.addr());
                    println!("x: 0x{:X}", op.x());
                    println!("y: 0x{:X}", op.y());
                    println!("n_high: 0x{:X}", op.n_high());
                    println!("n_low: 0x{:X}", op.n_low());
                    println!("k: 0x{:X}\n", op.k());
                },
                _ => continue
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ops::*;

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
                    vm.reg[$reg_before] = $reg_before_val;
                )+
                vm.exec(&$ins);
                $(
                    assert!(vm.reg[$reg_after] == $reg_after_val);
                )+
                let overflow = 15;
                assert!(vm.reg[overflow] == $over, "overflow was {}, wanted {}", vm.reg[overflow], $over);
            }
        )
    }

    // Add
    reg_test!(
        add_vx {
        before: { 2 => 0xFE, 3 => 0x01 },
        after: { 2 => 0xFF, 3 => 0x01 },
        overflow: 0,
        ins: Instruction::Add(2,3)
    });

    reg_test!(
        add_vx_overflows {
        before: { 2 => 0xFF, 3 => 0x01 },
        after: { 2 => 0x00, 3 => 0x01 },
        overflow: 1,
        ins: Instruction::Add(2,3)
    });

    // AddK
    reg_test!(
        add_k {
        before: { 0 => 0x09 },
        after: { 0 => 0x0B },
        overflow: 0,
        ins: Instruction::AddK(0,2)
    });

    reg_test!(
        add_k_overflows {
        before: { 0 => 0xFF },
        after: { 0 => 0x01 },
        overflow: 0, // Un-intuitive but not spec'd to set overflow
        ins: Instruction::AddK(0,2)
    });

    // Sub
    reg_test!(
        sub {
        before: { 0 => 0x3, 1 => 0x2 },
        after:  { 0 => 0x1, 1 => 0x2 },
        overflow: 1, // Defined as not-borrowed
        ins: Instruction::Sub(0,1)
    });

    reg_test!(
        sub_borrow {
        before: { 0 => 0x3, 1 => 0x5 },
        after:  { 0 => 0xFE, 1 => 0x5 },
        overflow: 0, // Defined as not-borrowed
        ins: Instruction::Sub(0,1)
    });

    // SubInv
    reg_test!(
        sub_inv {
        before: { 0 => 0x2, 1 => 0x3 },
        after:  { 0 => 0x1, 1 => 0x3 },
        overflow: 1, // Defined as not-borrowed
        ins: Instruction::SubInv(0,1)
    });

    reg_test!(
        sub_inv_borrow {
        before: { 0 => 0x5, 1 => 0x3 },
        after:  { 0 => 0xFE, 1 => 0x3 },
        overflow: 0, // Defined as not-borrowed
        ins: Instruction::SubInv(0,1)
    });

    // ShiftLeft
    reg_test!(
        shiftl_vx_vy {
        before: { 2 => 0xBB, 3 => 0x02 },
        after:  { 2 => 0x04, 3 => 0x02 },
        overflow: 0,
        ins: Instruction::ShiftLeft(2,3)
    });

    reg_test!(
        shiftl_vx_inplace {
        before: { 2 => 0b0111_0111 },
        after:  { 2 => 0b1110_1110 },
        overflow: 0,
        ins: Instruction::ShiftLeft(2,2)
    });

    reg_test!(
        shiftl_vx_inplace_overflow {
        before: { 2 => 0b1111_1111 },
        after:  { 2 => 0b1111_1110 },
        overflow: 1,
        ins: Instruction::ShiftLeft(2,2)
    });

    // ShiftRight
    reg_test!(
        shiftr_vx_vy {
        before: { 2 => 0xBB, 3 => 0x04 },
        after:  { 2 => 0x02, 3 => 0x04 },
        overflow: 0,
        ins: Instruction::ShiftRight(2,3)
    });

    reg_test!(
        shiftr_vx_inplace {
        before: { 2 => 0b1110_1110 },
        after:  { 2 => 0b0111_0111 },
        overflow: 0,
        ins: Instruction::ShiftRight(2,2)
    });

    reg_test!(
        shiftr_vx_inplace_overflow {
        before: { 2 => 0b1111_1111 },
        after:  { 2 => 0b0111_1111 },
        overflow: 1,
        ins: Instruction::ShiftRight(2,2)
    });
}
