use crate::bot::command::BotCommand;
use crate::config::Config;
use crate::media::controller::MediaController;
use crate::media::types::MediaStatus;

// ── Result type ───────────────────────────────────────────────────────────────

/// The outcome of dispatching a single Telegram update through the pipeline.
///
/// `Ignored` means the caller must make zero outbound API calls.
/// `Reply` means the caller should send the contained text back to the user.
#[derive(Debug, PartialEq)]
pub enum DispatchResult {
    Ignored,
    Reply(String),
}

// ── Dispatch ──────────────────────────────────────────────────────────────────

/// Dispatch a single Telegram update through the authorization + command pipeline.
///
/// Pure synchronous function — no teloxide dependency.  The thin adapter in
/// `main.rs` extracts `user_id` and `command` from a real teloxide `Update`
/// and delegates here; it then pattern-matches the result to decide whether
/// to call the Telegram API.
pub fn dispatch(
    user_id: i64,
    command: &BotCommand,
    config: &Config,
    controller: &dyn MediaController,
) -> DispatchResult {
    if !config.is_allowed(user_id) {
        return DispatchResult::Ignored;
    }
    DispatchResult::Reply(handle(command, controller))
}

fn handle(command: &BotCommand, controller: &dyn MediaController) -> String {
    match command {
        BotCommand::Play => match controller.toggle_play_pause() {
            Ok(status) => format_status(status),
            Err(msg) => format!("Error: {msg}"),
        },
        BotCommand::Next => match controller.next() {
            Ok(()) => "Skipped to next track.".to_string(),
            Err(msg) => format!("Error: {msg}"),
        },
        BotCommand::Previous => match controller.previous() {
            Ok(()) => "Went back to previous track.".to_string(),
            Err(msg) => format!("Error: {msg}"),
        },
        BotCommand::Player => match controller.get_current_state() {
            Ok((status, _meta)) => format!("Status: {}", format_status(status)),
            Err(msg) => format!("Error: {msg}"),
        },
    }
}

fn format_status(status: MediaStatus) -> String {
    match status {
        MediaStatus::Playing => "Playing".to_string(),
        MediaStatus::Paused  => "Paused".to_string(),
        MediaStatus::Stopped => "Stopped".to_string(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::media::types::{MediaMetadata, MediaStatus};
    use crate::mock::MockMediaController;

    fn allowed(ids: Vec<i64>) -> Config {
        Config { allowed_users: ids }
    }

    fn mock() -> MockMediaController {
        MockMediaController::new()
    }

    // ── Authorization gate ─────────────────────────────────────────────────────

    #[test]
    fn unauthorized_user_receives_ignored() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        let result = dispatch(999999999, &BotCommand::Play, &cfg, &m);
        assert_eq!(result, DispatchResult::Ignored);
    }

    #[test]
    fn unauthorized_user_produces_zero_controller_calls() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(999999999, &BotCommand::Play, &cfg, &m);
        assert_eq!(m.toggle_call_count(), 0);
    }

    #[test]
    fn unauthorized_user_with_next_command_produces_zero_calls() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(999999999, &BotCommand::Next, &cfg, &m);
        assert_eq!(m.next_call_count(), 0);
    }

    #[test]
    fn unauthorized_user_with_previous_command_produces_zero_calls() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(999999999, &BotCommand::Previous, &cfg, &m);
        assert_eq!(m.previous_call_count(), 0);
    }

    #[test]
    fn unauthorized_user_result_is_not_a_reply() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        let result = dispatch(999999999, &BotCommand::Play, &cfg, &m);
        assert!(
            !matches!(result, DispatchResult::Reply(_)),
            "unauthorized user must never receive a Reply"
        );
    }

    // ── Authorized user — correct handler triggered ────────────────────────────

    #[test]
    fn authorized_user_play_command_calls_toggle() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(111111111, &BotCommand::Play, &cfg, &m);
        assert_eq!(m.toggle_call_count(), 1);
    }

    #[test]
    fn authorized_user_next_command_calls_next() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(111111111, &BotCommand::Next, &cfg, &m);
        assert_eq!(m.next_call_count(), 1);
    }

    #[test]
    fn authorized_user_previous_command_calls_previous() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(111111111, &BotCommand::Previous, &cfg, &m);
        assert_eq!(m.previous_call_count(), 1);
    }

    #[test]
    fn authorized_user_player_command_returns_reply() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        let result = dispatch(111111111, &BotCommand::Player, &cfg, &m);
        assert!(matches!(result, DispatchResult::Reply(_)));
    }

    #[test]
    fn authorized_user_play_returns_playing_in_reply() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        m.set_toggle_result(Ok(MediaStatus::Playing));
        let DispatchResult::Reply(text) = dispatch(111111111, &BotCommand::Play, &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("Playing"), "reply was: {text}");
    }

    #[test]
    fn authorized_user_play_returns_paused_in_reply() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        m.set_toggle_result(Ok(MediaStatus::Paused));
        let DispatchResult::Reply(text) = dispatch(111111111, &BotCommand::Play, &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("Paused"), "reply was: {text}");
    }

    // ── Multiple authorized users ──────────────────────────────────────────────

    #[test]
    fn two_authorized_users_each_trigger_independently() {
        let cfg = allowed(vec![111111111, 222222222]);
        let m = mock();
        dispatch(111111111, &BotCommand::Play, &cfg, &m);
        dispatch(222222222, &BotCommand::Play, &cfg, &m);
        assert_eq!(m.toggle_call_count(), 2);
    }

    #[test]
    fn second_user_allowed_first_user_not() {
        let cfg = allowed(vec![222222222]);
        let m = mock();
        let r1 = dispatch(111111111, &BotCommand::Play, &cfg, &m);
        let r2 = dispatch(222222222, &BotCommand::Play, &cfg, &m);
        assert_eq!(r1, DispatchResult::Ignored);
        assert!(matches!(r2, DispatchResult::Reply(_)));
        assert_eq!(m.toggle_call_count(), 1);
    }

    // ── Controller error propagation ──────────────────────────────────────────

    #[test]
    fn controller_error_is_surfaced_as_reply_not_panic() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        m.set_toggle_result(Err("No players found".to_string()));
        let DispatchResult::Reply(text) = dispatch(111111111, &BotCommand::Play, &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("No players found"), "reply was: {text}");
    }

    #[test]
    fn next_controller_error_surfaced_as_reply() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        m.set_next_result(Err("No players found".to_string()));
        let DispatchResult::Reply(text) = dispatch(111111111, &BotCommand::Next, &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("No players found"), "reply was: {text}");
    }

    #[test]
    fn previous_controller_error_surfaced_as_reply() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        m.set_previous_result(Err("No players found".to_string()));
        let DispatchResult::Reply(text) = dispatch(111111111, &BotCommand::Previous, &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("No players found"), "reply was: {text}");
    }

    // ── Boundary / exclusive routing ──────────────────────────────────────────

    #[test]
    fn user_id_zero_is_unauthorized_unless_explicitly_allowed() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        let result = dispatch(0, &BotCommand::Play, &cfg, &m);
        assert_eq!(result, DispatchResult::Ignored);
    }

    #[test]
    fn negative_user_id_in_whitelist_is_allowed() {
        let cfg = allowed(vec![-100123456789]);
        let m = mock();
        let result = dispatch(-100123456789, &BotCommand::Play, &cfg, &m);
        assert!(matches!(result, DispatchResult::Reply(_)));
    }

    #[test]
    fn only_the_matching_method_is_called_per_command() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        dispatch(111111111, &BotCommand::Next, &cfg, &m);
        assert_eq!(m.next_call_count(), 1);
        assert_eq!(m.toggle_call_count(), 0);
        assert_eq!(m.previous_call_count(), 0);
    }

    // ── Player command formats state ───────────────────────────────────────────

    #[test]
    fn player_command_reply_contains_status() {
        let cfg = allowed(vec![111111111]);
        let m = mock();
        m.set_state_result(Ok((
            MediaStatus::Playing,
            MediaMetadata {
                title: "Song".to_string(),
                artist: "Artist".to_string(),
                album: None,
                art_url: None,
            },
        )));
        let DispatchResult::Reply(text) = dispatch(111111111, &BotCommand::Player, &cfg, &m) else {
            panic!("expected Reply");
        };
        assert!(text.contains("Playing"), "reply was: {text}");
    }
}
