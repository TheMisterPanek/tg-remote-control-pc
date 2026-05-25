use crate::media::{
    controller::MediaController,
    types::{MediaMetadata, MediaStatus},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

// ── Mock ──────────────────────────────────────────────────────────────────────

pub struct MockMediaController {
    toggle_result: Mutex<Result<MediaStatus, String>>,
    next_result: Mutex<Result<(), String>>,
    previous_result: Mutex<Result<(), String>>,
    state_result: Mutex<Result<(MediaStatus, MediaMetadata), String>>,

    toggle_calls: AtomicUsize,
    next_calls: AtomicUsize,
    previous_calls: AtomicUsize,
}

impl MockMediaController {
    pub fn new() -> Self {
        Self {
            toggle_result: Mutex::new(Ok(MediaStatus::Stopped)),
            next_result: Mutex::new(Ok(())),
            previous_result: Mutex::new(Ok(())),
            state_result: Mutex::new(Ok((
                MediaStatus::Stopped,
                MediaMetadata {
                    title: String::new(),
                    artist: String::new(),
                    album: None,
                    art_url: None,
                },
            ))),
            toggle_calls: AtomicUsize::new(0),
            next_calls: AtomicUsize::new(0),
            previous_calls: AtomicUsize::new(0),
        }
    }

    pub fn set_toggle_result(&self, result: Result<MediaStatus, String>) {
        *self.toggle_result.lock().unwrap() = result;
    }

    pub fn set_next_result(&self, result: Result<(), String>) {
        *self.next_result.lock().unwrap() = result;
    }

    pub fn set_previous_result(&self, result: Result<(), String>) {
        *self.previous_result.lock().unwrap() = result;
    }

    pub fn set_state_result(&self, result: Result<(MediaStatus, MediaMetadata), String>) {
        *self.state_result.lock().unwrap() = result;
    }

    pub fn toggle_call_count(&self) -> usize {
        self.toggle_calls.load(Ordering::SeqCst)
    }

    pub fn next_call_count(&self) -> usize {
        self.next_calls.load(Ordering::SeqCst)
    }

    pub fn previous_call_count(&self) -> usize {
        self.previous_calls.load(Ordering::SeqCst)
    }
}

impl Default for MockMediaController {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaController for MockMediaController {
    fn toggle_play_pause(&self) -> Result<MediaStatus, String> {
        self.toggle_calls.fetch_add(1, Ordering::SeqCst);
        self.toggle_result.lock().unwrap().clone()
    }

    fn next(&self) -> Result<(), String> {
        self.next_calls.fetch_add(1, Ordering::SeqCst);
        self.next_result.lock().unwrap().clone()
    }

    fn previous(&self) -> Result<(), String> {
        self.previous_calls.fetch_add(1, Ordering::SeqCst);
        self.previous_result.lock().unwrap().clone()
    }

    fn get_current_state(&self) -> Result<(MediaStatus, MediaMetadata), String> {
        self.state_result.lock().unwrap().clone()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_metadata() -> MediaMetadata {
        MediaMetadata {
            title: String::new(),
            artist: String::new(),
            album: None,
            art_url: None,
        }
    }

    // ── Default state ──────────────────────────────────────────────────────────

    #[test]
    fn new_mock_has_zero_call_counts() {
        let mock = MockMediaController::new();
        assert_eq!(mock.toggle_call_count(), 0);
        assert_eq!(mock.next_call_count(), 0);
        assert_eq!(mock.previous_call_count(), 0);
    }

    #[test]
    fn new_mock_toggle_returns_stopped_by_default() {
        let mock = MockMediaController::new();
        assert_eq!(mock.toggle_play_pause(), Ok(MediaStatus::Stopped));
    }

    #[test]
    fn new_mock_next_returns_ok_by_default() {
        let mock = MockMediaController::new();
        assert_eq!(mock.next(), Ok(()));
    }

    #[test]
    fn new_mock_previous_returns_ok_by_default() {
        let mock = MockMediaController::new();
        assert_eq!(mock.previous(), Ok(()));
    }

    #[test]
    fn new_mock_get_current_state_returns_stopped_by_default() {
        let mock = MockMediaController::new();
        let (status, meta) = mock.get_current_state().unwrap();
        assert_eq!(status, MediaStatus::Stopped);
        assert_eq!(meta, default_metadata());
    }

    // ── Configuring return values ──────────────────────────────────────────────

    #[test]
    fn toggle_can_be_configured_to_return_playing() {
        let mock = MockMediaController::new();
        mock.set_toggle_result(Ok(MediaStatus::Playing));
        assert_eq!(mock.toggle_play_pause(), Ok(MediaStatus::Playing));
    }

    #[test]
    fn toggle_can_be_configured_to_return_paused() {
        let mock = MockMediaController::new();
        mock.set_toggle_result(Ok(MediaStatus::Paused));
        assert_eq!(mock.toggle_play_pause(), Ok(MediaStatus::Paused));
    }

    #[test]
    fn state_can_be_configured_with_metadata() {
        let mock = MockMediaController::new();
        let meta = MediaMetadata {
            title: "Bohemian Rhapsody".to_string(),
            artist: "Queen".to_string(),
            album: Some("A Night at the Opera".to_string()),
            art_url: None,
        };
        mock.set_state_result(Ok((MediaStatus::Playing, meta.clone())));
        let (status, returned) = mock.get_current_state().unwrap();
        assert_eq!(status, MediaStatus::Playing);
        assert_eq!(returned, meta);
    }

    // ── Error simulation ───────────────────────────────────────────────────────

    #[test]
    fn toggle_can_simulate_no_players_found_error() {
        let mock = MockMediaController::new();
        mock.set_toggle_result(Err("No players found".to_string()));
        assert_eq!(mock.toggle_play_pause(), Err("No players found".to_string()));
    }

    #[test]
    fn next_can_simulate_no_players_found_error() {
        let mock = MockMediaController::new();
        mock.set_next_result(Err("No players found".to_string()));
        assert_eq!(mock.next(), Err("No players found".to_string()));
    }

    #[test]
    fn previous_can_simulate_no_players_found_error() {
        let mock = MockMediaController::new();
        mock.set_previous_result(Err("No players found".to_string()));
        assert_eq!(mock.previous(), Err("No players found".to_string()));
    }

    #[test]
    fn get_current_state_can_simulate_error() {
        let mock = MockMediaController::new();
        mock.set_state_result(Err("No players found".to_string()));
        assert_eq!(
            mock.get_current_state(),
            Err("No players found".to_string())
        );
    }

    // ── Call count tracking ────────────────────────────────────────────────────

    #[test]
    fn toggle_call_count_increments_on_each_call() {
        let mock = MockMediaController::new();
        mock.toggle_play_pause().ok();
        mock.toggle_play_pause().ok();
        mock.toggle_play_pause().ok();
        assert_eq!(mock.toggle_call_count(), 3);
    }

    #[test]
    fn next_call_count_increments_on_each_call() {
        let mock = MockMediaController::new();
        mock.next().ok();
        mock.next().ok();
        assert_eq!(mock.next_call_count(), 2);
    }

    #[test]
    fn previous_call_count_increments_on_each_call() {
        let mock = MockMediaController::new();
        mock.previous().ok();
        assert_eq!(mock.previous_call_count(), 1);
    }

    #[test]
    fn call_counts_are_independent_per_method() {
        let mock = MockMediaController::new();
        mock.toggle_play_pause().ok();
        mock.next().ok();
        mock.next().ok();
        // previous never called
        assert_eq!(mock.toggle_call_count(), 1);
        assert_eq!(mock.next_call_count(), 2);
        assert_eq!(mock.previous_call_count(), 0);
    }

    #[test]
    fn call_count_increments_even_when_method_returns_error() {
        let mock = MockMediaController::new();
        mock.set_toggle_result(Err("No players found".to_string()));
        mock.toggle_play_pause().ok();
        mock.toggle_play_pause().ok();
        assert_eq!(mock.toggle_call_count(), 2);
    }

    // ── Trait object compatibility ─────────────────────────────────────────────

    #[test]
    fn mock_is_usable_as_media_controller_trait_object() {
        let mock: Box<dyn MediaController> = Box::new(MockMediaController::new());
        assert!(mock.toggle_play_pause().is_ok());
        assert!(mock.next().is_ok());
        assert!(mock.previous().is_ok());
        assert!(mock.get_current_state().is_ok());
    }

    #[test]
    fn mock_satisfies_send_sync_bounds() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<MockMediaController>();
    }
}
