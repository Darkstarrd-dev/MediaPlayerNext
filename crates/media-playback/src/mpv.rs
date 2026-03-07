use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MpvOpenRequest {
    pub media_url: String,
    pub start_paused: bool,
    pub title: Option<String>,
}

pub fn build_mpv_open_args(request: &MpvOpenRequest) -> Vec<String> {
    let mut args = vec![
        "--idle=yes".to_string(),
        format!(
            "--pause={}",
            if request.start_paused { "yes" } else { "no" }
        ),
        request.media_url.clone(),
    ];

    if let Some(title) = &request.title {
        args.insert(2, format!("--force-media-title={title}"));
    }

    args
}

pub trait MpvLauncher {
    fn launch(&self, mpv_path: &Path, request: &MpvOpenRequest) -> Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct MpvProcessLauncher;

impl MpvLauncher for MpvProcessLauncher {
    fn launch(&self, mpv_path: &Path, request: &MpvOpenRequest) -> Result<()> {
        Command::new(mpv_path)
            .args(build_mpv_open_args(request))
            .spawn()
            .with_context(|| format!("spawn mpv: {}", mpv_path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{build_mpv_open_args, MpvOpenRequest};

    #[test]
    fn builds_mpv_open_args() {
        let args = build_mpv_open_args(&MpvOpenRequest {
            media_url: "media://asset/asset_video_001".to_string(),
            start_paused: true,
            title: Some("Asset Video".to_string()),
        });

        assert_eq!(args[0], "--idle=yes");
        assert_eq!(args[1], "--pause=yes");
        assert_eq!(args[2], "--force-media-title=Asset Video");
        assert_eq!(args[3], "media://asset/asset_video_001");
    }
}
