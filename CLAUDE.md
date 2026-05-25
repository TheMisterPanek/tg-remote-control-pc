# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`tg-media-remote` is a Telegram bot that controls media playback on a Linux machine via `playerctl`. It is a Cargo workspace project built under the MIT license using strict Spec-Driven Development (SDD) and Behavior-Driven Development (BDD). All logic is verified via `cargo test` — the execution environment has no internet access, no real D-Bus, and no `playerctl` binary.

## Commands

```bash
# Build entire workspace
cargo build

# Run all tests (primary verification method — no runtime needed)
cargo test

# Run tests for a specific crate
cargo test -p remote-core
cargo test -p remote-os

# Run a single test by name
cargo test -p remote-core <test_name>

# Check for compile errors without building artifacts
cargo check

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Workspace Architecture

```
tg-media-remote/
├── Cargo.toml          # Workspace root — members: ["crates/remote-core", "crates/remote-os"]
├── src/main.rs         # Composition root: reads env, wires config + controller + bot
├── config.toml         # Runtime config (not committed); contains allowed_users = [...]
├── crates/
│   ├── remote-core/    # Platform-agnostic bot logic, traits, mocks, BDD tests
│   └── remote-os/      # Linux system integration (#[cfg(target_os = "linux")])
```

### `remote-core` — Core Domain

This crate must compile and test on any platform with no system dependencies.

Key modules:
- `src/media/types.rs` — `MediaStatus` (Playing/Paused/Stopped), `MediaMetadata` (title, artist, album, art_url)
- `src/media/trait.rs` — `MediaController` trait (object-safe, `Send + Sync + 'static`)
- `src/config.rs` — Parses `config.toml`; enforces non-empty `allowed_users`; must return explicit errors on malformed TOML or missing/empty whitelist
- `src/mock.rs` — `MockMediaController`: stateful spy with configurable return values and per-method call counters
- `src/bot/` — Teloxide handler pipeline, `/player` command formatter, inline keyboard serialization

### `remote-os` — Linux Integration

- Wraps `std::process::Command` to invoke `playerctl`
- All code guarded by `#[cfg(target_os = "linux")]`
- Implements `MediaController` for the real OS layer
- Stubs only — not tested at runtime in the isolated container

### `src/main.rs` — Composition Root

Reads `TELOXIDE_TOKEN` and `CONFIG_PATH` (or defaults) from environment, loads config, instantiates either the real `remote-os` controller (Linux) or a stub, and starts the teloxide dispatcher.

## Core Trait Contract

```rust
pub trait MediaController: Send + Sync + 'static {
    fn toggle_play_pause(&self) -> Result<MediaStatus, String>;
    fn next(&self) -> Result<(), String>;
    fn previous(&self) -> Result<(), String>;
    fn get_current_state(&self) -> Result<(MediaStatus, MediaMetadata), String>;
}
```

All bot handlers are generic over `T: MediaController`. Never couple bot logic to `playerctl` directly.

## Security Model

The bot implements a strict Telegram user ID whitelist. The dispatcher checks `allowed_users` from `config.toml` **before** invoking any handler. Unauthorized user IDs are silently dropped — no response, no media command, no outbound API call. An empty whitelist must be treated as a configuration error at startup.

## Testing Constraints

- `MockMediaController` must pre-configure return values, simulate errors like `"No players found"`, and expose call counts for `next`, `previous`, and `toggle_play_pause`.
- Config tests must cover: single user, multiple users, invalid TOML syntax, missing `allowed_users`, and empty `allowed_users`.
- Dispatcher tests must assert authorized users trigger handlers and unauthorized users produce zero side effects.
- UI/callback tests must verify inline button presses invoke the correct trait methods and reflect state transitions.

## Development Protocol (SDD Steps)

When implementing features, follow this strict order — do not skip ahead:

1. Config parser + whitelist validation tests
2. `MockMediaController` with spy capabilities
3. Teloxide handler pipeline + whitelist dispatch tests
4. `/player` UI formatting + inline keyboard + callback tests
5. `remote-os` `playerctl` stub (Linux-conditional)
