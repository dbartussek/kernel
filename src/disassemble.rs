use std::path::Path;

pub fn disassemble<S, D>(source: S, destination: D)
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    let destination = destination.as_ref();

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let result = std::process::Command::new("objdump")
        .args(vec![
            //
            // Intel syntax. Because it makes sense
            "-M".to_string(),
            "intel".to_string(),
            //
            // Disassemble
            "-d".to_string(),
            //
            // Target
            source.as_ref().to_str().unwrap().to_string(),
        ])
        .output()
        .unwrap();
    std::fs::write(destination, result.stdout).unwrap();
}
