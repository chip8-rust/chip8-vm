// TODO revisit these after 1.0.0beta
#![feature(collections)]
#![feature(core)]
#![feature(io)]
#![feature(os)]
#![feature(path)]
#![feature(rand)]

extern crate shader_version;
extern crate input;
extern crate event;
extern crate graphics;
extern crate sdl2_window;
extern crate window;
extern crate opengl_graphics;
extern crate quack;

use std::cell::RefCell;
use sdl2_window::Sdl2Window;
use window::WindowSettings;
use opengl_graphics::{
    Gl,
};

use std::os;
use std::old_io::File;
// use std::time::duration::Duration;
use quack::Set;
// use input::keyboard::Key;
use input::Button;

use vm::Vm;

mod error;
mod ops;
mod vm;

const TITLE: &'static str = "Chip8";
const BEEP_TITLE: &'static str = "♬ Chip8 ♬";

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
    let mut rom_path = Path::new(format!("/Users/jakerr/Downloads/{}", roms[5]));
    if os::args().len() > 1 {
        rom_path = Path::new(os::args()[1].clone());
    }

    let mut rom_file = File::open(&rom_path).unwrap();

    match vm.load_rom(&mut rom_file) {
        Ok(size) => println!("Loaded rom size: {}", size),
        Err(e) => {
            println!("Error loading rom: {}", e);
            return;
        }
    }

    let mut dump_file = File::create(&Path::new("/Users/jakerr/tmp/dump.ch8ram")).unwrap();
    vm.dump_ram(&mut dump_file);

    let (width, height) = (800, 400);
    let opengl = shader_version::OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        WindowSettings {
            title: TITLE.to_string(),
            size: [width, height],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );

    let ref mut gl = Gl::new(opengl);
    let window = RefCell::new(window);

    fn keymap(k: Option<Button>) -> Option<u8> {
        use input::Key::*;
        if let Some(Button::Keyboard(k)) = k {
            return match k {
                D1 => Some(0x1),
                D2 => Some(0x2),
                D3 => Some(0x3),

                Q  => Some(0x4),
                W  => Some(0x5),
                E  => Some(0x6),

                A  => Some(0x7),
                S  => Some(0x8),
                D  => Some(0x9),

                Z  => Some(0xA),
                X  => Some(0x0),
                C  => Some(0xB),

                D4 => Some(0xC),
                R  => Some(0xD),
                F  => Some(0xE),
                V  => Some(0xF),

                _ => None
            }
        }
        return None
    }

    for e in event::events(&window) {
        use event::{ ReleaseEvent, UpdateEvent, PressEvent, RenderEvent };

        if let Some(args) = e.update_args() {
            vm.step(args.dt as f32);
            if vm.beeping() {
                (*window.borrow_mut()).window.set_title(BEEP_TITLE);
            } else {
                (*window.borrow_mut()).window.set_title(TITLE);
            }
        }
        if let Some(args) = e.render_args() {
            use graphics::*;
            gl.draw([0, 0, args.width as i32, args.height as i32], |&mut: c, gl| {
                graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
                let r = Rectangle::new([1.0, 1.0, 1.0, 1.0]);
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
        if let Some(keynum) = keymap(e.press_args()) {
            vm.set_key(keynum);
        }
        if let Some(keynum) = keymap(e.release_args()) {
            vm.unset_key(keynum);
        }
    }
}
