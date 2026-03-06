use app_core::cli::{parse_command, run_command, BackendCommand};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = parse_command(&args);
    let config_path = workspace_root().join("config").join("local.paths.json");

    if command == BackendCommand::Help {
        println!("Usage: cargo run --bin backend-harness -- [diagnostics]");
    }

    let output = run_command(command, config_path)?;
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("workspace root should be available")
}
