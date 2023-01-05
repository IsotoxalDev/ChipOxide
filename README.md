# ChipOxide
A crate to create a Chip 8 Emulator for any backend.

# what is Chip 8?
Chip 8 is an interpreter, which has internal workings similar to that of a hardware console. It is said to be the starting point for getting into emulator development.

### what do you mean by a crate to creaete a chip 8 Emulator?
Exactly what it sounds like, with this crate one can create a backend for the chip 8 core to any device. One needs to create an IO struct which would interact with the core and intrepret a program.

### Available backends
  1. [Terminal](https://github.com/IsotoxalDev/ChipOxide/blob/main/examples/terminal.rs) (Through [cossterm](https://crates.io/crates/crossterm))

### How to get started?
You can go through the [Eamples](https://github.com/IsotoxalDev/ChipOxide/tree/main/examples) directory. The documentaion on docs.rs would provide a starting point.

### Resources used
 1. https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
 2. http://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf
