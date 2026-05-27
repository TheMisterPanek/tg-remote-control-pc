use crate::bot::dispatch::{dispatch_callback_action, DispatchResult};
use crate::config::Config;
use crate::media::controller::MediaController;

// ── Callback data constants ───────────────────────────────────────────────────

pub const CALLBACK_TOGGLE: &str = "toggle";
pub const CALLBACK_NEXT: &str = "next";
pub const CALLBACK_PREV: &str = "prev";
pub const CALLBACK_VOL_UP: &str = "vol_up";
pub const CALLBACK_VOL_DOWN: &str = "vol_down";

// ── CallbackAction ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CallbackAction {
    TogglePlayPause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
}

impl CallbackAction {
    pub fn from_callback_data(data: &str) -> Option<Self> {
        match data {
            CALLBACK_TOGGLE => Some(Self::TogglePlayPause),
            CALLBACK_NEXT => Some(Self::Next),
            CALLBACK_PREV => Some(Self::Previous),
            CALLBACK_VOL_UP => Some(Self::VolumeUp),
            CALLBACK_VOL_DOWN => Some(Self::VolumeDown),
            _ => None,
        }
    }

    pub fn to_callback_data(&self) -> &'static str {
        match self {
            Self::TogglePlayPause => CALLBACK_TOGGLE,
            Self::Next => CALLBACK_NEXT,
            Self::Previous => CALLBACK_PREV,
            Self::VolumeUp => CALLBACK_VOL_UP,
            Self::VolumeDown => CALLBACK_VOL_DOWN,
        }
    }
}

// ── dispatch_callback ─────────────────────────────────────────────────────────

pub fn dispatch_callback(
    user_id: i64,
    callback_data: &str,
    config: &Config,
    controller: &dyn MediaController,
) -> DispatchResult {
    if !config.is_allowed(user_id) {
        return DispatchResult::Ignored;
    }
    let Some(action) = CallbackAction::from_callback_data(callback_data) else {
        return DispatchResult::Ignored;
    };
    dispatch_callback_action(&action, controller)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::dispatch::DispatchResult;
    use crate::bot::ui::PlayerReply;
    use crate::config::Config;
    use crate::media::types::{MediaMetadata, MediaStatus};
    use crate::mock::MockMediaController;

    fn cfg(ids: Vec<i64>) -> Config {
        Config { token: "t".to_string(), allowed_users: ids }
    }

    fn meta(title: &str) -> MediaMetadata {
        MediaMetadata {
            title: title.to_string(),
            artist: "Artist".to_string(),
            album: None,
            art_url: None,
        }
    }

    fn mock_with_state(status: MediaStatus) -> MockMediaController {
        let m = MockMediaController::new();
        m.set_state_result(Ok((status, meta("Track"))));
        m
    }

    // ── Round-trip ─────────────────────────────────────────────────────────────

    #[test]
    fn toggle_callback_data_round_trips() {
        assert_eq!(
            CallbackAction::from_callback_data("toggle"),
            Some(CallbackAction::TogglePlayPause)
        );
        assert_eq!(CallbackAction::TogglePlayPause.to_callback_data(), "toggle");
    }

    #[test]
    fn next_callback_data_round_trips() {
        assert_eq!(
            CallbackAction::from_callback_data("next"),
            Some(CallbackAction::Next)
        );
        assert_eq!(CallbackAction::Next.to_callback_data(), "next");
    }

    #[test]
    fn previous_callback_data_round_trips() {
        assert_eq!(
            CallbackAction::from_callback_data("prev"),
            Some(CallbackAction::Previous)
        );
        assert_eq!(CallbackAction::Previous.to_callback_data(), "prev");
    }

    #[test]
    fn volume_up_callback_data_round_trips() {
        assert_eq!(
            CallbackAction::from_callback_data("vol_up"),
            Some(CallbackAction::VolumeUp)
        );
        assert_eq!(CallbackAction::VolumeUp.to_callback_data(), "vol_up");
    }

    #[test]
    fn volume_down_callback_data_round_trips() {
        assert_eq!(
            CallbackAction::from_callback_data("vol_down"),
            Some(CallbackAction::VolumeDown)
        );
        assert_eq!(CallbackAction::VolumeDown.to_callback_data(), "vol_down");
    }

    #[test]
    fn unknown_callback_data_returns_none() {
        assert_eq!(CallbackAction::from_callback_data("garbage"), None);
        assert_eq!(CallbackAction::from_callback_data(""), None);
    }

    // ── Method dispatch ────────────────────────────────────────────────────────

    #[test]
    fn toggle_callback_invokes_toggle_play_pause_exactly_once() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        dispatch_callback(1, "toggle", &cfg, &m);
        assert_eq!(m.toggle_call_count(), 1);
        assert_eq!(m.next_call_count(), 0);
        assert_eq!(m.previous_call_count(), 0);
    }

    #[test]
    fn next_callback_invokes_next_exactly_once() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        dispatch_callback(1, "next", &cfg, &m);
        assert_eq!(m.next_call_count(), 1);
        assert_eq!(m.toggle_call_count(), 0);
        assert_eq!(m.previous_call_count(), 0);
    }

    #[test]
    fn previous_callback_invokes_previous_exactly_once() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        dispatch_callback(1, "prev", &cfg, &m);
        assert_eq!(m.previous_call_count(), 1);
        assert_eq!(m.toggle_call_count(), 0);
        assert_eq!(m.next_call_count(), 0);
    }

    #[test]
    fn vol_up_callback_invokes_volume_up_exactly_once() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        dispatch_callback(1, "vol_up", &cfg, &m);
        assert_eq!(m.volume_up_call_count(), 1);
        assert_eq!(m.volume_down_call_count(), 0);
        assert_eq!(m.toggle_call_count(), 0);
    }

    #[test]
    fn vol_down_callback_invokes_volume_down_exactly_once() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        dispatch_callback(1, "vol_down", &cfg, &m);
        assert_eq!(m.volume_down_call_count(), 1);
        assert_eq!(m.volume_up_call_count(), 0);
        assert_eq!(m.toggle_call_count(), 0);
    }

    // ── Return type ────────────────────────────────────────────────────────────

    #[test]
    fn toggle_callback_returns_player_reply() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        let result = dispatch_callback(1, "toggle", &cfg, &m);
        assert!(matches!(result, DispatchResult::PlayerReply(_)));
    }

    #[test]
    fn next_callback_returns_player_reply() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        let result = dispatch_callback(1, "next", &cfg, &m);
        assert!(matches!(result, DispatchResult::PlayerReply(_)));
    }

    #[test]
    fn previous_callback_returns_player_reply() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        let result = dispatch_callback(1, "prev", &cfg, &m);
        assert!(matches!(result, DispatchResult::PlayerReply(_)));
    }

    #[test]
    fn vol_up_callback_returns_player_reply() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        let result = dispatch_callback(1, "vol_up", &cfg, &m);
        assert!(matches!(result, DispatchResult::PlayerReply(_)));
    }

    #[test]
    fn vol_down_callback_returns_player_reply() {
        let cfg = cfg(vec![1]);
        let m = mock_with_state(MediaStatus::Playing);
        let result = dispatch_callback(1, "vol_down", &cfg, &m);
        assert!(matches!(result, DispatchResult::PlayerReply(_)));
    }

    #[test]
    fn vol_up_controller_error_surfaced_as_reply() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        m.set_volume_up_result(Err("No players found".to_string()));
        let DispatchResult::Reply(text) = dispatch_callback(1, "vol_up", &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("No players found"), "text was: {text}");
    }

    #[test]
    fn vol_down_controller_error_surfaced_as_reply() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        m.set_volume_down_result(Err("No players found".to_string()));
        let DispatchResult::Reply(text) = dispatch_callback(1, "vol_down", &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("No players found"), "text was: {text}");
    }

    // ── State transitions ──────────────────────────────────────────────────────

    #[test]
    fn state_transition_playing_to_paused() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        m.set_toggle_result(Ok(MediaStatus::Paused));
        m.set_state_result(Ok((MediaStatus::Paused, meta("Track"))));
        let DispatchResult::PlayerReply(PlayerReply { text, .. }) =
            dispatch_callback(1, "toggle", &cfg, &m)
        else {
            panic!("expected PlayerReply");
        };
        assert!(text.contains("Paused"), "text was: {text}");
    }

    #[test]
    fn state_transition_paused_to_playing() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        m.set_toggle_result(Ok(MediaStatus::Playing));
        m.set_state_result(Ok((MediaStatus::Playing, meta("Track"))));
        let DispatchResult::PlayerReply(PlayerReply { text, .. }) =
            dispatch_callback(1, "toggle", &cfg, &m)
        else {
            panic!("expected PlayerReply");
        };
        assert!(text.contains("Playing"), "text was: {text}");
    }

    // ── Error surfacing ────────────────────────────────────────────────────────

    #[test]
    fn callback_controller_error_surfaced_as_reply() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        m.set_toggle_result(Err("No players found".to_string()));
        let DispatchResult::Reply(text) = dispatch_callback(1, "toggle", &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("No players found"), "text was: {text}");
    }

    #[test]
    fn state_refresh_error_after_next_surfaced_as_reply() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        m.set_state_result(Err("D-Bus connection failed".to_string()));
        let DispatchResult::Reply(text) = dispatch_callback(1, "next", &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("D-Bus connection failed"), "text was: {text}");
    }

    // ── Authorization ──────────────────────────────────────────────────────────

    #[test]
    fn unauthorized_user_callback_returns_ignored() {
        let cfg = cfg(vec![111]);
        let m = MockMediaController::new();
        let result = dispatch_callback(999, "toggle", &cfg, &m);
        assert_eq!(result, DispatchResult::Ignored);
        assert_eq!(m.toggle_call_count(), 0);
        assert_eq!(m.state_call_count(), 0);
    }

    #[test]
    fn unknown_callback_data_returns_ignored() {
        let cfg = cfg(vec![1]);
        let m = MockMediaController::new();
        let result = dispatch_callback(1, "garbage", &cfg, &m);
        assert_eq!(result, DispatchResult::Ignored);
        assert_eq!(m.toggle_call_count(), 0);
    }
}
