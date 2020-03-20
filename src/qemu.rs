use crate::{build::build, parameters::Parameters};
use std::{error::Error, path::Path};

pub fn run_qemu(parameters: &Parameters) -> Result<(), Box<dyn Error>> {
    build(parameters)?;

    let ovmf = Path::new("OVMF");
    let ovmf_code = ovmf.join("OVMF_CODE.fd");
    let ovmf_vars = ovmf.join("OVMF_VARS.fd");

    let qemu_args = vec![
        //
        // Disable default devices
        "-nodefaults".to_string(),
        //
        // Use a modern machine, with acceleration if possible.
        "-machine".to_string(),
        "q35,accel=kvm:tcg".to_string(),
        //
        // Allocate memory
        "-m".to_string(),
        "128M".to_string(),
        //
        // Set up OVMF.
        "-drive".to_string(),
        format!(
            "if=pflash,format=raw,file={},readonly=on",
            ovmf_code
                .to_str()
                .expect("Cannot represent OVMF_CODE path as str")
        ),
        "-drive".to_string(),
        format!(
            "if=pflash,format=raw,file={},readonly=on",
            ovmf_vars
                .to_str()
                .expect("Cannot represent OVMF_VARS path as str")
        ),
        //
        // Mount the esp directory
        "-drive".to_string(),
        format!(
            "format=raw,file=fat:rw:{}",
            parameters
                .esp_directory
                .to_str()
                .expect("Cannot represent esp_directory as str")
        ),
        //
        // Connect serial to stdio
        "-serial".to_string(),
        "stdio".to_string(),
        //
        // Enable the exit signal
        "-device".to_string(),
        "isa-debug-exit,iobase=0xf4,iosize=0x04".to_string(),
        //
        // Add a vga display
        "-vga".to_string(),
        "std".to_string(),
    ];

    std::process::Command::new("qemu-system-x86_64")
        .args(qemu_args)
        .status()
        .unwrap();

    Ok(())
}