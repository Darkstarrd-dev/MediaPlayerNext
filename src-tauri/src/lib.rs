mod runtime_check;

use runtime_check::{run_runtime_smoke_check, RuntimeSmokeCheckResult};
use std::env;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! MediaPlayerNext host is ready.")
}

#[tauri::command]
fn runtime_smoke_check(ffmpeg_path: String, mpv_path: String) -> Result<RuntimeSmokeCheckResult, String> {
    run_runtime_smoke_check(&ffmpeg_path, &mpv_path).map_err(|error| error.to_string())
}

pub fn runtime_smoke_check_entry() -> anyhow::Result<()> {
    let mut ffmpeg_path: Option<String> = None;
    let mut mpv_path: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ffmpeg-path" => {
                ffmpeg_path = args.next();
            }
            "--mpv-path" => {
                mpv_path = args.next();
            }
            _ => {}
        }
    }

    let ffmpeg_path = ffmpeg_path.ok_or_else(|| anyhow::anyhow!("missing --ffmpeg-path"))?;
    let mpv_path = mpv_path.ok_or_else(|| anyhow::anyhow!("missing --mpv-path"))?;
    let result = run_runtime_smoke_check(&ffmpeg_path, &mpv_path)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, runtime_smoke_check])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
