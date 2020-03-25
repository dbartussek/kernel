use crate::{
    parameters::{
        build_parameters::BuildParameters, config::Config, Parameters,
    },
    xtool::run_xtool,
};
use std::error::Error;

fn xbuild(parameters: &BuildParameters) {
    let manifest_path = parameters.manifest_path();

    let mut args = vec![];

    if parameters.config == Config::Release {
        args.push("--release".to_string());
    }

    run_xtool(
        "xbuild",
        &parameters.target.to_string(),
        manifest_path.as_ref().map(|s| s.to_str().unwrap()),
        args,
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
