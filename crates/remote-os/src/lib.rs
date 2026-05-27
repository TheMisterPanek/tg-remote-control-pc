//! Linux system integration for `tg-media-remote`.
//!
//! Wraps `dbus-send` via [`std::process::Command`] and implements
//! [`remote_core::media::controller::MediaController`] for the real OS layer.
//! All code in this crate is conditionally compiled for Linux only.

#[cfg(target_os = "linux")]
pub mod dbus;
