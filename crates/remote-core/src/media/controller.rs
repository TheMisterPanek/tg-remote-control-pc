use crate::media::types::{MediaMetadata, MediaStatus};

/// Abstraction over a media player backend.
///
/// All bot handlers are generic over `T: MediaController`. The real
/// implementation lives in `remote-os` and wraps `playerctl`.
/// Tests use `MockMediaController` from `remote-core::mock`.
pub trait MediaController: Send + Sync + 'static {
    /// Toggle between play and pause. Returns the new `MediaStatus`.
    fn toggle_play_pause(&self) -> Result<MediaStatus, String>;
    /// Skip to the next track.
    fn next(&self) -> Result<(), String>;
    /// Go back to the previous track.
    fn previous(&self) -> Result<(), String>;
    /// Return the current playback status and track metadata.
    fn get_current_state(&self) -> Result<(MediaStatus, MediaMetadata), String>;
}
