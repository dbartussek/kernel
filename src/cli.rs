use structopt::*;

#[derive(Debug, StructOpt)]
pub enum Command {

    #[structopt(about = "Build the kernel image")]
    Build,

    #[structopt(about = "Build kernel and run in qemu")]
    Run,
}
