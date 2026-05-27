use remote_core::media::controller::MediaController;
use remote_core::media::types::{MediaMetadata, MediaStatus};
use std::process::Command;

pub struct DbusController;

impl Default for DbusController {
    fn default() -> Self {
        Self::new()
    }
}

impl DbusController {
    pub fn new() -> Self {
        Self
    }

    fn run(args: &[&str]) -> Result<String, String> {
        let out = Command::new("dbus-send")
            .args(args)
            .output()
            .map_err(|e| format!("failed to run dbus-send: {e}"))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            Err(if err.is_empty() { "dbus-send: non-zero exit".to_string() } else { err })
        }
    }

    fn find_active_player() -> Result<String, String> {
        let output = Self::run(&[
            "--session", "--print-reply",
            "--dest=org.freedesktop.DBus",
            "/",
            "org.freedesktop.DBus.ListNames",
        ])?;
        let players: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("org.mpris.MediaPlayer2.") {
                    line.find("string \"")
                        .map(|pos| line[pos + 8..].trim_end_matches('"').to_string())
                } else {
                    None
                }
            })
            .collect();
        if players.is_empty() {
            return Err("No MPRIS players found".to_string());
        }
        // Prefer stable names (no PID suffix) over instance-specific ones
        players.iter()
            .find(|p| !p.contains(".instance"))
            .or_else(|| players.first())
            .cloned()
            .ok_or_else(|| "No MPRIS players found".to_string())
    }

    fn parse_playback_status(output: &str) -> Result<MediaStatus, String> {
        for line in output.lines() {
            if let Some(pos) = line.find("string \"") {
                let after = &line[pos + 8..];
                if let Some(end) = after.find('"') {
                    return match &after[..end] {
                        "Playing" => Ok(MediaStatus::Playing),
                        "Paused"  => Ok(MediaStatus::Paused),
                        "Stopped" => Ok(MediaStatus::Stopped),
                        other     => Err(format!("unexpected PlaybackStatus: {other}")),
                    };
                }
            }
        }
        Err("PlaybackStatus not found in dbus output".to_string())
    }

    fn parse_string_field(output: &str, key: &str) -> Option<String> {
        let search = format!("\"{}\"", key);
        let lines: Vec<&str> = output.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(&search) {
                for subsequent in &lines[i + 1..] {
                    if subsequent.contains("dict entry(") {
                        break;
                    }
                    if let Some(pos) = subsequent.find("string \"") {
                        let after = &subsequent[pos + 8..];
                        if let Some(end) = after.find('"') {
                            let value = after[..end].to_string();
                            return if value.is_empty() { None } else { Some(value) };
                        }
                    }
                }
                break;
            }
        }
        None
    }

    fn parse_double_field(output: &str) -> Result<f64, String> {
        for line in output.lines() {
            if let Some(pos) = line.find("double ") {
                let rest = line[pos + 7..].trim();
                if let Ok(v) = rest.parse::<f64>() {
                    return Ok(v);
                }
            }
        }
        Err("Volume not found in dbus output".to_string())
    }

    fn get_volume(player: &str) -> Result<f64, String> {
        let output = Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties.Get",
            "string:org.mpris.MediaPlayer2.Player",
            "string:Volume",
        ])?;
        Self::parse_double_field(&output)
    }

    fn set_volume(player: &str, volume: f64) -> Result<(), String> {
        Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties.Set",
            "string:org.mpris.MediaPlayer2.Player",
            "string:Volume",
            &format!("variant:double:{volume:.4}"),
        ]).map(|_| ())
    }

    fn get_status(player: &str) -> Result<MediaStatus, String> {
        let output = Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties.Get",
            "string:org.mpris.MediaPlayer2.Player",
            "string:PlaybackStatus",
        ])?;
        Self::parse_playback_status(&output)
    }
}

impl MediaController for DbusController {
    fn toggle_play_pause(&self) -> Result<MediaStatus, String> {
        let player = Self::find_active_player()?;
        Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player.PlayPause",
        ])?;
        Self::get_status(&player)
    }

    fn next(&self) -> Result<(), String> {
        let player = Self::find_active_player()?;
        Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player.Next",
        ]).map(|_| ())
    }

    fn previous(&self) -> Result<(), String> {
        let player = Self::find_active_player()?;
        Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player.Previous",
        ]).map(|_| ())
    }

    fn get_current_state(&self) -> Result<(MediaStatus, MediaMetadata), String> {
        let player = Self::find_active_player()?;
        let status = Self::get_status(&player)?;
        let meta_out = Self::run(&[
            "--session", "--print-reply",
            &format!("--dest={player}"),
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties.Get",
            "string:org.mpris.MediaPlayer2.Player",
            "string:Metadata",
        ])?;
        let title   = Self::parse_string_field(&meta_out, "xesam:title").unwrap_or_default();
        let artist  = Self::parse_string_field(&meta_out, "xesam:artist").unwrap_or_default();
        let album   = Self::parse_string_field(&meta_out, "xesam:album");
        let art_url = Self::parse_string_field(&meta_out, "mpris:artUrl");
        Ok((status, MediaMetadata { title, artist, album, art_url }))
    }

    fn volume_up(&self) -> Result<(), String> {
        let player = Self::find_active_player()?;
        let v = Self::get_volume(&player)?;
        Self::set_volume(&player, (v + 0.05).min(1.0))
    }

    fn volume_down(&self) -> Result<(), String> {
        let player = Self::find_active_player()?;
        let v = Self::get_volume(&player)?;
        Self::set_volume(&player, (v - 0.05).max(0.0))
    }
}
