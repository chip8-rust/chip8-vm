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
//! (`Vm`) and a few shared constants.
//!
//! The `error` module contains the `Chip8Error` implementation of
//! `std:error::Error` for any kinds of errors that might occur using
//! the `chip8_vm` crate.

// Silence 'core' feature warnings
// for `error:Error` and such
#![feature(core)]
// Silence 'io' feature warnings
// for `BufWriter` and such
#![feature(io)]

extern crate rand;

pub mod error;
pub mod instructions;
pub mod vm;

pub use instructions::*;
