/// The playback state of the active media player.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaStatus {
    /// A track is currently playing.
    Playing,
    /// Playback is paused.
    Paused,
    /// No track is active.
    Stopped,
}

/// Metadata for the currently active track.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub art_url: Option<String>,
}
