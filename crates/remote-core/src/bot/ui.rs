use crate::bot::callback::{CALLBACK_NEXT, CALLBACK_PREV, CALLBACK_TOGGLE, CALLBACK_VOL_DOWN, CALLBACK_VOL_UP};
use crate::media::types::{MediaMetadata, MediaStatus};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct InlineButton {
    pub label: String,
    pub callback_data: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineKeyboard {
    pub rows: Vec<Vec<InlineButton>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerReply {
    pub text: String,
    pub keyboard: InlineKeyboard,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn build_keyboard() -> InlineKeyboard {
    InlineKeyboard {
        rows: vec![
            vec![
                InlineButton { label: "⏮".to_string(), callback_data: CALLBACK_PREV.to_string() },
                InlineButton { label: "⏯".to_string(), callback_data: CALLBACK_TOGGLE.to_string() },
                InlineButton { label: "⏭".to_string(), callback_data: CALLBACK_NEXT.to_string() },
            ],
            vec![
                InlineButton { label: "🔉".to_string(), callback_data: CALLBACK_VOL_DOWN.to_string() },
                InlineButton { label: "🔊".to_string(), callback_data: CALLBACK_VOL_UP.to_string() },
            ],
        ],
    }
}

pub fn format_player_reply(status: MediaStatus, meta: MediaMetadata) -> PlayerReply {
    let mut lines = vec![
        format!("Now Playing: {}", meta.title),
        format!("Artist: {}", meta.artist),
    ];
    if let Some(album) = meta.album {
        lines.push(format!("Album: {album}"));
    }
    lines.push(format!("Status: {}", format_status(&status)));
    PlayerReply { text: lines.join("\n"), keyboard: build_keyboard() }
}

pub(crate) fn format_status(status: &MediaStatus) -> &'static str {
    match status {
        MediaStatus::Playing => "Playing",
        MediaStatus::Paused => "Paused",
        MediaStatus::Stopped => "Stopped",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::types::{MediaMetadata, MediaStatus};

    fn meta(title: &str, artist: &str, album: Option<&str>) -> MediaMetadata {
        MediaMetadata {
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.map(|s| s.to_string()),
            art_url: None,
        }
    }

    fn reply(status: MediaStatus, title: &str, artist: &str, album: Option<&str>) -> PlayerReply {
        format_player_reply(status, meta(title, artist, album))
    }

    // ── Text formatting ────────────────────────────────────────────────────────

    #[test]
    fn player_reply_text_contains_title() {
        let r = reply(MediaStatus::Playing, "Bohemian Rhapsody", "Queen", None);
        assert!(r.text.contains("Bohemian Rhapsody"), "text was: {}", r.text);
    }

    #[test]
    fn player_reply_text_contains_artist() {
        let r = reply(MediaStatus::Playing, "Song", "Queen", None);
        assert!(r.text.contains("Queen"), "text was: {}", r.text);
    }

    #[test]
    fn player_reply_text_contains_status_playing() {
        let r = reply(MediaStatus::Playing, "Song", "Artist", None);
        assert!(r.text.contains("Playing"), "text was: {}", r.text);
    }

    #[test]
    fn player_reply_text_contains_status_paused() {
        let r = reply(MediaStatus::Paused, "Song", "Artist", None);
        assert!(r.text.contains("Paused"), "text was: {}", r.text);
    }

    #[test]
    fn player_reply_text_contains_status_stopped() {
        let r = reply(MediaStatus::Stopped, "Song", "Artist", None);
        assert!(r.text.contains("Stopped"), "text was: {}", r.text);
    }

    #[test]
    fn player_reply_text_contains_album_when_present() {
        let r = reply(MediaStatus::Playing, "Song", "Artist", Some("A Night at the Opera"));
        assert!(r.text.contains("A Night at the Opera"), "text was: {}", r.text);
    }

    #[test]
    fn player_reply_text_omits_album_line_when_none() {
        let r = reply(MediaStatus::Playing, "Song", "Artist", None);
        assert!(!r.text.contains("Album:"), "text was: {}", r.text);
    }

    // ── Keyboard ───────────────────────────────────────────────────────────────

    fn all_buttons(kb: &InlineKeyboard) -> Vec<&InlineButton> {
        kb.rows.iter().flatten().collect()
    }

    #[test]
    fn keyboard_has_two_rows() {
        let kb = build_keyboard();
        assert_eq!(kb.rows.len(), 2);
    }

    #[test]
    fn keyboard_first_row_has_three_buttons() {
        let kb = build_keyboard();
        assert_eq!(kb.rows[0].len(), 3);
    }

    #[test]
    fn keyboard_second_row_has_two_buttons() {
        let kb = build_keyboard();
        assert_eq!(kb.rows[1].len(), 2);
    }

    #[test]
    fn keyboard_first_row_order_is_prev_toggle_next() {
        let kb = build_keyboard();
        assert_eq!(kb.rows[0][0].callback_data, "prev");
        assert_eq!(kb.rows[0][1].callback_data, "toggle");
        assert_eq!(kb.rows[0][2].callback_data, "next");
    }

    #[test]
    fn keyboard_second_row_order_is_vol_down_vol_up() {
        let kb = build_keyboard();
        assert_eq!(kb.rows[1][0].callback_data, "vol_down");
        assert_eq!(kb.rows[1][1].callback_data, "vol_up");
    }

    #[test]
    fn keyboard_first_row_labels_contain_media_symbols() {
        let kb = build_keyboard();
        assert!(kb.rows[0][0].label.contains('⏮'));
        assert!(kb.rows[0][1].label.contains('⏯'));
        assert!(kb.rows[0][2].label.contains('⏭'));
    }

    #[test]
    fn keyboard_second_row_labels_contain_volume_symbols() {
        let kb = build_keyboard();
        assert!(kb.rows[1][0].label.contains('🔉'));
        assert!(kb.rows[1][1].label.contains('🔊'));
    }

    #[test]
    fn keyboard_contains_toggle_button() {
        let kb = build_keyboard();
        assert!(all_buttons(&kb).iter().any(|b| b.callback_data == "toggle"));
    }

    #[test]
    fn keyboard_contains_next_button() {
        let kb = build_keyboard();
        assert!(all_buttons(&kb).iter().any(|b| b.callback_data == "next"));
    }

    #[test]
    fn keyboard_contains_previous_button() {
        let kb = build_keyboard();
        assert!(all_buttons(&kb).iter().any(|b| b.callback_data == "prev"));
    }

    #[test]
    fn keyboard_contains_volume_up_button() {
        let kb = build_keyboard();
        assert!(all_buttons(&kb).iter().any(|b| b.callback_data == "vol_up"));
    }

    #[test]
    fn keyboard_contains_volume_down_button() {
        let kb = build_keyboard();
        assert!(all_buttons(&kb).iter().any(|b| b.callback_data == "vol_down"));
    }
}
