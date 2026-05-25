//! Platform-agnostic core of `tg-media-remote`.
//!
//! This crate contains all bot logic, configuration parsing, and the
//! [`media::controller::MediaController`] trait. It has no system dependencies
//! and compiles on any platform. Tests run in complete isolation — no Telegram
//! connection, no D-Bus, and no `playerctl` binary required.

pub mod bot;
pub mod config;
pub mod media;
pub mod mock;
