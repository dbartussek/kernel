use crate::{build::build, cli::QemuArgs, parameters::Parameters};
use std::{error::Error, path::Path};

pub fn run_qemu(
    parameters: &Parameters,
    args: &QemuArgs,
) -> Result<(), Box<dyn Error>> {
    build(parameters)?;

    let ovmf = Path::new("OVMF");
    let ovmf_code = ovmf.join("OVMF_CODE.fd");
    let ovmf_vars = ovmf.join("OVMF_VARS.fd");

    let mut qemu_args = vec![
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
        "1G".to_string(),
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
        // Connect serial 1 to stdio
        "-serial".to_string(),
        "stdio".to_string(),
        //
        // Connect serial 2 to file
        "-serial".to_string(),
        "file:serial2_log".to_string(),
        //
        // Enable the exit signal
        "-device".to_string(),
        "isa-debug-exit,iobase=0xf4,iosize=0x04".to_string(),
        //
        // Add a vga display
        "-vga".to_string(),
        "std".to_string(),
        //
        // Debug options:
        //
        // Print a log when the cpu resets (for triple faults)
        "-d".to_string(),
        "cpu_reset".to_string(),
        "-D".to_string(),
        "log.txt".to_string(),
        //
        // Accept gdb remote (target remote localhost:1234)
        "-s".to_string(),
    ];

    if args.gdb {
        // Wait for gdb to attach.
        // This will break execution on the first instruction
        qemu_args.push("-S".to_string());
    }

    let status = std::process::Command::new("qemu-system-x86_64")
        .args(qemu_args)
        .status()
        .unwrap();

    let kernel_status_code =
        status
            .code()
            .and_then(|v| if v & 1 == 1 { Some(v >> 1) } else { None });

    if let Some(kernel_status_code) = kernel_status_code {
        println!("\nkernel_status_code: 0x{:X}", kernel_status_code);

        if kernel_status_code != 0 {
            std::process::exit(kernel_status_code);
        }
    }

    Ok(())
}
