use std::io::{BufWriter, Reader};
use error::Ch8Error;
use ops::{Op, Instruction};
const RAM_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;

pub struct Vm {
    reg: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    stack: [usize; 256],
    ram: [u8; RAM_SIZE],

    timer: u8,
    tone: u8,

    screen: [u8; 64 * 32],
    keys: [u8; 16],
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            reg: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            stack: [0; 256],
            ram: [0; RAM_SIZE],

            timer: 0,
            tone: 0,

            screen: [0; 64 * 32],
            keys: [0; 16],
        }
    }

    pub fn load_rom(&mut self, reader: &mut Reader) -> Result<usize, Ch8Error> {
        let rom = try!(reader.read_to_end());
        if rom.len() > (RAM_SIZE - PROGRAM_START) {
           return Err(Ch8Error::Io("Rom was larger than available RAM", None))
        }
        let mut ram = BufWriter::new(&mut self.ram[PROGRAM_START..RAM_SIZE]);
        try!(ram.write(rom.as_slice()));
        return Ok(rom.len());
    }

    pub fn dump_ram(&self, writer: &mut Writer) {
        writer.write(&self.ram).unwrap();
    }

    fn exec(&mut self, op: &Op) -> bool {
        use ops::Instruction::*;
        let ins = Instruction::from_op(op);
        println!("Executing instruction: 0x{:X} {:?}   v: {:?}", self.pc, ins, self.reg);
        match ins {
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
                self.reg[15] =  0x80 & y;

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
            AddToI(vx) => {
                self.i += self.reg[vx as usize] as usize;
            }
            StoreRegisters(vx) => {
                let vx = vx as usize;
                let i = self.i as usize;

                let mut dst = &mut self.ram[i..i+vx+1];
                for (x,b) in dst.iter_mut().enumerate() {
                  println!("store reg {} to {}", x, i);
                    *b = self.reg[x];
                }
            },
            LoadRegisters(vx) => {
                let vx = vx as usize;
                let i = self.i as usize;

                let src = &self.ram[i..i+vx+1];
                for (x,b) in src.iter().enumerate() {
                  println!("read reg {} from {}", x, i);
                    self.reg[x] = *b;
                }
            },
            other => {
                println!("Instruction not implemented {:?} skipping...", other)
            }
        }
        return false;
    }

    pub fn step(&mut self) -> bool {
        let raw = {
            let codes = &self.ram[self.pc..self.pc+2];
            ((codes[0] as u16) << 8) | codes[1] as u16
        };
        let op = Op::new(raw);
        self.pc += 2;
        let idle = self.exec(&op);
        return idle;
    }

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
