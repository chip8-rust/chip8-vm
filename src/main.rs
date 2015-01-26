use std::io;
use std::io::{File};
use std::time::duration::Duration;

use vm::Vm;

mod error;
mod ops;
mod vm;

fn main() {
    let mut vm = Vm::new();

    let roms = [
        "IBM Logo.ch8",           // 0
        "Chip8 Picture.ch8",      // 1
        "Fishie [Hap, 2005].ch8", // 2
        "zerod.ch8",              // 3
        "sierp.ch8",              // 4
        "pong.ch8",               // 5
    ];
    let rom_path = Path::new(format!("/Users/jakerr/Downloads/{}", roms[5]));

    let mut rom_file = File::open(&rom_path).unwrap();

    match vm.load_rom(&mut rom_file) {
        Ok(size) => println!("Loaded rom size: {}", size),
        Err(e) => println!("Error loading rom: {}", e)
    }

    let mut dump_file = File::create(&Path::new("/Users/jakerr/tmp/dump.ch8ram")).unwrap();
    vm.dump_ram(&mut dump_file);

    loop {
        if vm.step(0.016) { break; }
        io::timer::sleep(Duration::milliseconds(16));
        println!("");
        vm.print_screen();
    }
}
