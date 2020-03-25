use crate::{
    parameters::{
        build_parameters::BuildParameters, config::Config, Parameters,
    },
    xtool::run_xtool,
};
use std::error::Error;

fn xclippy(parameters: &BuildParameters) {
    let manifest_path = parameters.manifest_path();

    let mut args = vec![];

    if parameters.config == Config::Release {
        args.push("--release".to_string());
    }

    run_xtool(
        "xclippy",
        &parameters.target.to_string(),
        manifest_path.as_ref().map(|s| s.to_str().unwrap()),
        args,
    )
}

pub fn clippy(parameters: &Parameters) -> Result<(), Box<dyn Error>> {
    xclippy(&parameters.uefi_loader_build_parameters);
    xclippy(&parameters.kernel_build_parameters);

    Ok(())
}
