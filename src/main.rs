pub mod build;
pub mod cli;
pub mod parameters;
pub mod qemu;

use crate::cli::Command;
use crate::parameters::Parameters;
use crate::qemu::run_qemu;
use build::build;
use std::error::Error;
use structopt::StructOpt;

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
    }

    Ok(())
}
