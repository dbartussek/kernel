pub mod build;
pub mod cli;
pub mod disassemble;
pub mod parameters;
pub mod qemu;

use crate::cli::Command;
use crate::parameters::Parameters;
use crate::qemu::run_qemu;
use build::build;
use disassemble::disassemble;
use std::error::Error;
use std::path::Path;
use structopt::StructOpt;
use walkdir::WalkDir;

pub fn main() -> Result<(), Box<dyn Error>> {
    let command: Command = StructOpt::from_args();

    let parameters = Parameters::default();

    match command {
        Command::Build => {
            build(&parameters)?;
        }
        Command::Run => {
            run_qemu(&parameters)?;
        }
        Command::Disassemble => {
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
        }
    }

    Ok(())
}
