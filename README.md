Chip-8 interpreter in Rust
==
This is an educational/toy project for me to get some more practice coding in
Rust / organizing modules, testing etc. Feedback is more than welcome!

Status
==
* All 35 original Chip-8 instructions are implemented.
* Graphics are implemented with [Piston](http://www.piston.rs/).
* Sound is not supported but is faked by updating the title bar with a note
symbol when sound should be playing.

Usage
==

```
> cargo build
> cargo run [path_to_ch8_rom]
```

Controls are mapped to these 16 buttons:

  1  |  2  |  3  |  4
-----|-----|-----|-----
  Q  |  W  |  E  |  R
  A  |  S  |  D  |  F
  Z  |  X  |  C  |  V

Spec
==
I used these two resources as the spec for my vm:
* [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
* [Mastering Chip-8 By Matthew Mikolay](http://mattmik.com/chip8.html)
They were both incredibly helpful so thanks to the authors!

Licence
==
MIT
