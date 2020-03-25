pub mod cli;
pub mod disassemble;
pub mod parameters;
pub mod qemu;
pub mod xtool;

use crate::{
    cli::Command,
    parameters::Parameters,
    qemu::run_qemu,
    xtool::{build::build, clippy::clippy},
};
use disassemble::disassemble;
use std::{error::Error, path::Path};
use structopt::StructOpt;
use walkdir::WalkDir;

pub fn main() -> Result<(), Box<dyn Error>> {
    let command: Command = StructOpt::from_args();

    let mut parameters = Parameters::default();

    if let Some(build_args) = command.get_build_args() {
        parameters.apply_cli(build_args);
    }

    match command {
        Command::Build(_) => {
            build(&parameters)?;
        },
        Command::Clippy(_) => {
            clippy(&parameters)?;
        },
        Command::Run(args) => {
            run_qemu(&parameters, &args)?;
        },
        Command::Disassemble(_) => {
            build(&parameters)?;

            for entry in WalkDir::new(&parameters.esp_directory)
                .into_iter()
                .filter(|entry| {
                    entry
                        .as_ref()
                        .ok()
                        .map(|entry| entry.file_type().is_file())
                        .unwrap_or(true)
                })
            {
                let entry = entry?;
                let path: &Path = entry.path();

                let mut destination = Path::new("dis").to_path_buf();

                if let Some(parent) = path.parent() {
                    destination.push(parent);
                }

                destination.push(format!(
                    "{}.asm",
                    path.file_name().unwrap().to_str().unwrap()
                ));

                disassemble(path, destination);
            }
        },
    }

    Ok(())
}
