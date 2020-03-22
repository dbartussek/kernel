pub mod build_parameters;
pub mod config;

use crate::{cli::BuildArgs, parameters::build_parameters::BuildParameters};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Parameters {
    pub esp_directory: PathBuf,

    pub uefi_loader_build_parameters: BuildParameters,
    pub uefi_loader_binary_name: String,

    pub kernel_build_parameters: BuildParameters,
    pub kernel_binary_name: String,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            esp_directory: PathBuf::from("esp"),

            uefi_loader_build_parameters: BuildParameters::uefi_default(),
            uefi_loader_binary_name: "uefi_loader.efi".to_string(),

            kernel_build_parameters: BuildParameters::kernel_default(),
            kernel_binary_name: "kernel_core".to_string(),
        }
    }
}

impl Parameters {
    pub fn apply_cli(&mut self, args: &BuildArgs) {
        self.uefi_loader_build_parameters.apply_cli(args);
        self.kernel_build_parameters.apply_cli(args);
    }
}
