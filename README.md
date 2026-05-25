# tg-media-remote

A Telegram bot that controls media playback on a Linux desktop via [`playerctl`](https://github.com/altdesktop/playerctl).

Send `/player`, `/play`, `/next`, or `/previous` from Telegram — the bot forwards the command to whatever MPRIS-compatible media player is currently active (Spotify, VLC, Firefox, etc.).

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-cargo%20test-green)](#development)

---

## Features

- Toggle play/pause, skip forward/back from anywhere via Telegram
- Strict user-ID whitelist — only your account can control your machine
- Zero response to unauthorized users (no reply, no media command, no API call)
- Platform-agnostic core — bot logic compiles and tests on any OS
- Pluggable `MediaController` trait — swap `playerctl` for any backend

---

## Requirements

| Requirement | Notes |
|---|---|
| Linux | `playerctl` integration is Linux-only |
| Rust 1.70+ | `cargo build` / `cargo test` |
| [`playerctl`](https://github.com/altdesktop/playerctl) | Must be on `$PATH` at runtime |
| Telegram bot token | Obtain via [@BotFather](https://t.me/BotFather) |

---

## Quick Start

### 1. Create a Telegram bot

Open [@BotFather](https://t.me/BotFather) and run `/newbot`. Copy the token it gives you.

### 2. Find your Telegram user ID

Message [@userinfobot](https://t.me/userinfobot) — it will reply with your numeric user ID.

### 3. Create `config.toml`

```toml
# config.toml  —  do NOT commit this file
allowed_users = [123456789]          # your Telegram user ID(s)
```

Multiple users:

```toml
allowed_users = [123456789, 987654321]
```

> **Security note:** an empty `allowed_users` array is a startup error — the bot refuses to run.

### 4. Build and run

```bash
cargo build --release

TELOXIDE_TOKEN="your-bot-token" \
CONFIG_PATH="./config.toml" \
./target/release/tg-media-remote
```

`CONFIG_PATH` defaults to `./config.toml` if unset.

### 5. Start a chat

Send `/player` to your bot. It will show the current playback status. Use `/play`, `/next`, or `/previous` to control playback.

---

## Bot Commands

| Command | Action |
|---|---|
| `/player` | Show current player status |
| `/play` | Toggle play / pause |
| `/next` | Skip to next track |
| `/previous` | Go back to previous track (`/prev` also works) |

---

## Configuration Reference

`config.toml` is read once at startup. The bot exits with an error if the file is missing, malformed, or contains an empty whitelist.

```toml
# Required — list of Telegram user IDs permitted to send commands.
# Must contain at least one entry.
allowed_users = [123456789]
```

---

## Architecture

```
tg-media-remote/
├── src/main.rs             # Composition root — reads env, wires everything
├── crates/
│   ├── remote-core/        # Platform-agnostic bot logic (no system deps)
│   │   ├── src/config.rs   # TOML parser + whitelist validation
│   │   ├── src/media/      # MediaController trait + types
│   │   ├── src/bot/        # Dispatch pipeline + command parsing
│   │   └── src/mock.rs     # MockMediaController for tests
│   └── remote-os/          # Linux integration (playerctl via std::process::Command)
│       └── src/playerctl.rs
└── config.toml             # Runtime config (not committed)
```

### Crates

**`remote-core`** — all bot logic, no system dependencies. Compiles and tests on any platform. Contains the `MediaController` trait, the whitelist dispatch pipeline, and the `MockMediaController` used in every test.

**`remote-os`** — Linux-only (`#[cfg(target_os = "linux")]`). Wraps `std::process::Command` to invoke `playerctl`. Implements `MediaController` for the real OS layer.

### Security model

The whitelist check is the very first thing the dispatch function does. Unauthorized user IDs are silently dropped — no reply, no media command, no outbound Telegram API call. This is enforced structurally: the `DispatchResult::Ignored` variant tells the caller to make zero API calls.

---

## Development

```bash
# Run all tests (no system dependencies required)
cargo test

# Run tests for a specific crate
cargo test -p remote-core
cargo test -p remote-os

# Check for compile errors
cargo check

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
```

All tests run in complete isolation — no Telegram connection, no D-Bus, no `playerctl` binary required.

### Project status

| Step | Description | Status |
|------|-------------|--------|
| 1 | Config parser + whitelist validation | Complete |
| 2 | `MockMediaController` spy | Complete |
| 3 | Handler pipeline + whitelist dispatch | Complete |
| 4 | `/player` UI + inline keyboard + callbacks | In progress |
| 5 | `remote-os` playerctl integration | Pending |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

[MIT](LICENSE) — Copyright © 2026 Pavel
