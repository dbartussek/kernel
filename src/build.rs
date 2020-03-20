use crate::parameters::{build_parameters::BuildParameters, Parameters};
use std::error::Error;

pub fn run_xtool(
    tool: &str,
    target: &str,
    manifest_path: Option<&str>,
    args: Vec<String>,
) {
    println!();

    let mut final_args =
        vec![tool.to_string(), "--target".to_string(), target.to_string()];

    if let Some(manifest_path) = manifest_path {
        final_args.push("--manifest-path".to_string());
        final_args.push(manifest_path.to_string());
    }

    final_args.extend(args.into_iter());

    let status = std::process::Command::new("cargo")
        .args(final_args)
        .status()
        .unwrap();

    println!();

    if !status.success() {
        std::process::exit(status.code().unwrap_or(-1));
    }
}

pub fn xbuild(parameters: &BuildParameters) {
    let manifest_path = parameters.manifest_path();

    run_xtool(
        "xbuild",
        &parameters.target.to_string(),
        manifest_path.as_ref().map(|s| s.to_str().unwrap()),
        vec![],
    )
}

pub fn build(parameters: &Parameters) -> Result<(), Box<dyn Error>> {
    xbuild(&parameters.uefi_loader_build_parameters);
    xbuild(&parameters.kernel_build_parameters);

    {
        // Copy efi loader
        let boot_directory = parameters.esp_directory.join("EFI/Boot");
        std::fs::create_dir_all(&boot_directory)?;

        let produced_file = parameters
            .uefi_loader_build_parameters
            .build_directory()
            .join(&parameters.uefi_loader_binary_name);
        let efi_output = boot_directory.join("BootX64.efi");

        std::fs::copy(produced_file, efi_output)?;
    }

    {
        // Copy kernel

        let produced_file = parameters
            .kernel_build_parameters
            .build_directory()
            .join(&parameters.kernel_binary_name);
        let kernel_output = parameters.esp_directory.join("kernel.elf");

        std::fs::copy(produced_file, kernel_output)?;
    }

    Ok(())
}
