pub mod build;
pub mod parameters;
pub mod qemu;

use crate::parameters::Parameters;
use crate::qemu::run_qemu;
use build::build;
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let parameters = Parameters::default();

    run_qemu(&parameters)?;

    Ok(())
}
