use std::env;
use walkdir::WalkDir;

fn main() {
    let mut build = nasm_rs::Build::new();

    for f in WalkDir::new("src")
        .into_iter()
        .map(|r| r.unwrap())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name().to_str().unwrap().ends_with(".asm")
        })
    {
        build.file(f.path());
    }

    let target = env::var("TARGET").expect("TARGET must be set");

    if cfg!(target_env = "msvc") && target.contains("unknown-bare") {
        println!("Patching target_env msvc");
        build.archiver_is_msvc(false);
    }

    build.compile("interrupt_handler_asm");
}
