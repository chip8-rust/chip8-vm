Virtual Machine for the CHIP-8 programming language
==
This crate implements a virtual machine of the CHIP-8 programming language. It
can be used as the back end for CHIP-8 emulator projects / debuggers etc.

Status
==
[![travis-badge][]][travis] [![appveyor-badge][]][appveyor]
* All 35 original Chip-8 instructions are implemented.

Usage
==
This crate does not depend on additional system libraries, it only depends on
other Rust crates.  It can be used as a back end for your own emulator by
adding it as a dependency in your `Cargo.toml`.

To depend on the latest released version that we host on [crates.io] (https://crates.io/crates/chip8_vm):

```toml
[dependencies]
chip8_vm = "0.*"
```

To depend on the development version:
```toml
[dependencies.chip8_vm]
git = "https://github.com/chip8-rust/chip8-vm"
```

See an example integration with a UI in the [chip8_ui](https://github.com/chip8-rust/chip8-ui/blob/master/src/main.rs) crate code.
For further information, take a look at the [`chip8_vm` rustdoc](https://chip8-rust.github.io/chip8-vm/chip8_vm).

Spec
==
These two resources were used as the spec for this vm:
* [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
* [Mastering Chip-8 By Matthew Mikolay](http://mattmik.com/chip8.html)

They were both incredibly helpful. Our thanks to the authors!

Licence
==
MIT

[travis-badge]: https://img.shields.io/travis/chip8-rust/chip8-vm/master.svg?label=linux%20build
[travis]: https://travis-ci.org/chip8-rust/chip8-vm
[appveyor-badge]: https://img.shields.io/appveyor/ci/robo9k/chip8-vm/master.svg?label=windows%20build
[appveyor]: https://ci.appveyor.com/project/robo9k/chip8-vm/branch/master
