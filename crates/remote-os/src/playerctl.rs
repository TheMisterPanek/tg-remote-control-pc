use remote_core::media::controller::MediaController;
use remote_core::media::types::{MediaMetadata, MediaStatus};
use std::process::Command;

pub struct PlayerctlController;

impl PlayerctlController {
    pub fn new() -> Self {
        Self
    }

    fn run(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new("playerctl")
            .args(args)
            .output()
            .map_err(|e| format!("failed to run playerctl: {e}"))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            Err(if err.is_empty() { "playerctl: non-zero exit".to_string() } else { err })
        }
    }

    fn status(&self) -> Result<MediaStatus, String> {
        match self.run(&["status"])?.as_str() {
            "Playing" => Ok(MediaStatus::Playing),
            "Paused"  => Ok(MediaStatus::Paused),
            "Stopped" => Ok(MediaStatus::Stopped),
            other     => Err(format!("unexpected playerctl status: {other}")),
        }
    }
}

impl MediaController for PlayerctlController {
    fn toggle_play_pause(&self) -> Result<MediaStatus, String> {
        self.run(&["play-pause"])?;
        self.status()
    }

    fn next(&self) -> Result<(), String> {
        self.run(&["next"]).map(|_| ())
    }

    fn previous(&self) -> Result<(), String> {
        self.run(&["previous"]).map(|_| ())
    }

    fn get_current_state(&self) -> Result<(MediaStatus, MediaMetadata), String> {
        let status = self.status()?;
        let title  = self.run(&["metadata", "title"]).unwrap_or_default();
        let artist = self.run(&["metadata", "artist"]).unwrap_or_default();
        let album  = self.run(&["metadata", "album"]).ok().filter(|s| !s.is_empty());
        let art_url = self.run(&["metadata", "mpris:artUrl"]).ok().filter(|s| !s.is_empty());
        Ok((status, MediaMetadata { title, artist, album, art_url }))
    }
}
