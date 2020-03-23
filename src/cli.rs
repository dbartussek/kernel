use structopt::*;

#[derive(Debug, StructOpt)]
pub struct BuildArgs {
    #[structopt(long)]
    pub release: bool,
}

#[derive(Debug, StructOpt)]
pub struct QemuArgs {
    #[structopt(flatten)]
    pub build: BuildArgs,

    #[structopt(long)]
    pub gdb: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(about = "Build the kernel image")]
    Build(BuildArgs),

    #[structopt(about = "Build kernel and run in qemu")]
    Run(QemuArgs),

    #[structopt(about = "Build kernel and disassemble")]
    Disassemble(BuildArgs),
}

impl Command {
    pub fn get_build_args(&self) -> Option<&BuildArgs> {
        Some(match self {
            Command::Build(b) | Command::Disassemble(b) => b,
            Command::Run(q) => &q.build,
        })
    }
}
