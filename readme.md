
#### Dependencies
- Rust nightly
- LLVM lld
- qemu for `run`
- readelf for `disassemble`

#### Building
The root crate is the builder. Just use `cargo run -- {command} {args} (--release)`.

Possible commands are:
- `build` compiles the kernel and UEFI loader and copies them into the esp directory. 
This directory can be used as a fat32 partition to boot on an x86_64 UEFI system  
- `clippy` runs xlippy on the project
- `run` first runs build, then starts the kernel in qemu
- `disassemble` builds and disassembles all components
