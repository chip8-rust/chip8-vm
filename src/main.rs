extern crate shader_version;
extern crate input;
extern crate event;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate quack;

use std::cell::RefCell;
use sdl2_window::Sdl2Window;
use opengl_graphics::{
    Gl,
};

use std::io;
use std::io::{File};
use std::time::duration::Duration;
use quack::Set;

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
    let rom_path = Path::new(format!("/Users/jakerr/Downloads/{}", roms[3]));

    let mut rom_file = File::open(&rom_path).unwrap();

    match vm.load_rom(&mut rom_file) {
        Ok(size) => println!("Loaded rom size: {}", size),
        Err(e) => println!("Error loading rom: {}", e)
    }

    let mut dump_file = File::create(&Path::new("/Users/jakerr/tmp/dump.ch8ram")).unwrap();
    vm.dump_ram(&mut dump_file);

    let (width, height) = (800, 400);
    let opengl = shader_version::OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        event::WindowSettings {
            title: "Chip8".to_string(),
            size: [width, height],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );

    let ref mut gl = Gl::new(opengl);
    let window = RefCell::new(window);
    for e in event::events(&window) {
        use event::{ PressEvent, RenderEvent };

        if let Some(args) = e.render_args() {
            use graphics::*;
            vm.step(0.016);
            gl.draw([0, 0, args.width as i32, args.height as i32], |&mut: c, gl| {
                graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
                let mut r = Rectangle::new([1.0, 1.0, 1.0, 1.0]);
                let off = Color([0.0, 0.0, 0.0, 1.0]);
                let on = Color([1.0, 1.0, 1.0, 1.0]);

                let w = args.width as f64 / 64.0;
                let h = args.height as f64 / 32.0;

                for (y,row) in vm.screen_rows().enumerate() {
                    for (x,byte) in row.iter().enumerate() {
                        let x = x as f64 * w;
                        let y = y as f64 * h;
                        r.set(match *byte { 0 => off, _ => on })
                        .draw([x, y, w, h], &c, gl);
                    }
                }
            });
        }
        if let Some(k) = e.press_args() {
            println!("Press {:?}", k);
        }
    }
}
