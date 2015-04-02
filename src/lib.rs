//! Virtual machine for the CHIP-8 programming language
//!
//! This crate implements a virtual machine for the CHIP-8
//! programming language.
//! It can be used as a backend for CHIP-8 emulators, debuggers
//! and so on.
//!
//!
//! The code is split into the `instructions` module, which provides
//! the translation from raw bits (`RawInstruction`) into valid
//! instructions (`Instruction`).
//!
//! The `vm` module contains the actual virtual machine implementation
//! (`Vm`).
//!
//! The `error` module contains the `Chip8Error` implementation of
//! `std:error::Error` for any kinds of errors that might occur using
//! the `chip8_vm` crate.

// Silence 'core' feature warnings
// for `error:Error` and such
#![feature(core)]

#![feature(slice_patterns)]

extern crate rand;

#[macro_use]
extern crate log;

pub mod error;
pub mod instructions;
pub mod vm;

pub use instructions::*;

/// Returns the version of this crate in the format `MAJOR.MINOR.PATCH`.
pub fn version() -> &'static str {
    // TODO: There's also an optional _PRE part
    concat!(
        env!("CARGO_PKG_VERSION_MAJOR"), ".",
        env!("CARGO_PKG_VERSION_MINOR"), ".",
        env!("CARGO_PKG_VERSION_PATCH"),
    )
}
