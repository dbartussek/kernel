pub mod build;
pub mod clippy;

pub fn run_xtool(
    tool: &str,
    target: &str,
    manifest_path: Option<&str>,
    args: Vec<String>,
) {
    println!();

    let mut final_args =
        vec![tool.to_string(), "--target".to_string(), target.to_string()];

    if let Some(manifest_path) = manifest_path {
        final_args.push("--manifest-path".to_string());
        final_args.push(manifest_path.to_string());
    }

    final_args.extend(args.into_iter());

    let status = std::process::Command::new("cargo")
        .args(final_args)
        .status()
        .unwrap();

    println!();

    if !status.success() {
        std::process::exit(status.code().unwrap_or(-1));
    }
}
