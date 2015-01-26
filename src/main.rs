//TODO(remove)
#![allow(dead_code)]
#![allow(unstable)]

use std::io;
use std::io::{File};
use std::time::duration::Duration;

use vm::Vm;

mod error;
mod ops;
mod vm;

fn main() {
    let mut vm = Vm::new();

    //let mut rom_file = File::open(&Path::new("/Users/jakerr/Downloads/IBM Logo.ch8")).unwrap();
     let mut rom_file = File::open(&Path::new("/Users/jakerr/Downloads/Chip8 Picture.ch8")).unwrap();
    // let mut rom_file = File::open(&Path::new("/Users/jakerr/Downloads/Fishie [Hap, 2005].ch8")).unwrap();
    // let mut rom_file = File::open(&Path::new("/Users/jakerr/Downloads/zerod.ch8.txt")).unwrap();
    // let mut rom_file = File::open(&Path::new("/Users/jakerr/Downloads/sierp.ch8")).unwrap();
    match vm.load_rom(&mut rom_file) {
        Ok(size) => println!("Loaded rom size: {}", size),
        Err(e) => println!("Error loading rom: {}", e)
    }

    let mut dump_file = File::create(&Path::new("/Users/jakerr/tmp/dump.ch8ram")).unwrap();
    vm.dump_ram(&mut dump_file);

    loop {
        if vm.step() { break; }
        // io::timer::sleep(Duration::milliseconds(80));
        // vm.print_screen();
    }
    io::timer::sleep(Duration::milliseconds(300));
    vm.print_screen();
    //vm.print_disassembly();
}
