# tg-media-remote — Project Roadmap

A Telegram bot that controls media playback on a Linux machine via `playerctl`.
Built as a Cargo workspace using Spec-Driven Development (SDD) and BDD test suites.

---

## Implementation Phases

Development follows a strict ordered protocol. Each step must be fully tested before
the next begins. Steps are verified exclusively via `cargo test` — no runtime or
D-Bus is assumed.

---

### Step 1 — Config parser + whitelist validation `remote-core/src/config.rs`

**Status: Complete**

#### Deliverables
- `Config` struct with `allowed_users: Vec<i64>` field
- `ConfigError` enum: `ParseError(String)`, `MissingWhitelist`, `EmptyWhitelist`
- `Config::from_toml(content: &str) -> Result<Config, ConfigError>`
- `Config::is_allowed(user_id: i64) -> bool` helper

#### Test coverage
- [x] Single user accepted
- [x] Multiple users accepted
- [x] Extra keys in TOML are ignored
- [x] Large Telegram user IDs (up to 10 digits)
- [x] Negative user IDs (channel/group IDs)
- [x] Malformed TOML returns `ParseError`
- [x] Wrong value type (strings instead of integers) returns `ParseError`
- [x] Completely empty TOML returns `MissingWhitelist`
- [x] Missing `allowed_users` key returns `MissingWhitelist`
- [x] Empty `allowed_users` array returns `EmptyWhitelist`
- [x] Error messages are descriptive and mention the relevant key
- [x] `is_allowed` returns true for known users, false for unknown
- [x] Round-trip: parse then check authorization
- [x] `ConfigError` implements `std::error::Error`

---

### Step 2 — `MockMediaController` with spy capabilities `remote-core/src/mock.rs`

**Status: Complete**

#### Deliverables
- `MockMediaController` struct implementing `MediaController`
- Configurable return values per method via `set_*_result()` setters
- Per-method call counters via `*_call_count()` getters
- Interior mutability via `Mutex` (return values) and `AtomicUsize` (counters)

#### API surface
```rust
// Configuration
mock.set_toggle_result(Ok(MediaStatus::Playing));
mock.set_toggle_result(Err("No players found".to_string()));
mock.set_next_result(Ok(()));
mock.set_previous_result(Err("No players found".to_string()));
mock.set_state_result(Ok((MediaStatus::Playing, metadata)));

// Spy
mock.toggle_call_count()   // usize
mock.next_call_count()     // usize
mock.previous_call_count() // usize
```

#### Test coverage
- [x] All counters start at zero
- [x] Default `toggle_play_pause` returns `Ok(Stopped)`
- [x] Default `next` / `previous` return `Ok(())`
- [x] Default `get_current_state` returns `Ok((Stopped, empty metadata))`
- [x] `toggle` configurable to `Playing` and `Paused`
- [x] `get_current_state` configurable with rich metadata
- [x] All four methods can simulate `Err("No players found")`
- [x] `toggle` call count increments per call
- [x] `next` call count increments per call
- [x] `previous` call count increments per call
- [x] Counters are independent across methods
- [x] Counter increments even when method returns an error
- [x] Usable as `Box<dyn MediaController>` trait object
- [x] Satisfies `Send + Sync + 'static` bounds

---

### Step 3 — Teloxide handler pipeline + whitelist dispatch `remote-core/src/bot/`

**Status: Pending**

#### Deliverables
- `src/bot/mod.rs` — module root, exports dispatcher builder
- `src/bot/handler.rs` — command handlers generic over `T: MediaController`
- `src/bot/dispatch.rs` — middleware that enforces whitelist before any handler runs
- `src/main.rs` updated — wires `Config`, `MockMediaController`/OS controller, dispatcher

#### Behaviour spec
- Authorized user sends `/player` → handler is invoked, bot replies
- Unauthorized user sends any message → silently dropped; zero handler calls; zero outbound Telegram API calls
- Handler calls the correct `MediaController` method and returns the result to the user

#### Test coverage (planned)
- [ ] Authorized user triggers the handler
- [ ] Unauthorized user produces zero `MockMediaController` calls
- [ ] Unauthorized user produces zero bot replies
- [ ] Multiple authorized users each trigger independently
- [ ] Handler propagates controller errors as user-facing messages

---

### Step 4 — `/player` UI formatting + inline keyboard + callbacks `remote-core/src/bot/`

**Status: Pending**

#### Deliverables
- `/player` reply formatter: renders `MediaStatus` and `MediaMetadata` into a human-readable message
- Inline keyboard builder: three buttons — ⏮ Previous, ⏯ Play/Pause, ⏭ Next
- Callback query handler: maps each button press to the correct `MediaController` method
- State-transition rendering: after a button press, the message reflects the new `MediaStatus`

#### Behaviour spec
- `/player` command displays current track info with inline keyboard
- Pressing ⏯ calls `toggle_play_pause`; updated status shown
- Pressing ⏭ calls `next`; state refreshed
- Pressing ⏮ calls `previous`; state refreshed
- Error responses from the controller are shown as human-friendly messages, not panics

#### Test coverage (planned)
- [ ] `/player` output contains title, artist, and status
- [ ] Inline keyboard contains all three buttons with correct callback data
- [ ] ⏯ callback invokes `toggle_play_pause` exactly once
- [ ] ⏭ callback invokes `next` exactly once
- [ ] ⏮ callback invokes `previous` exactly once
- [ ] State transitions: Playing → Paused → Playing
- [ ] Controller error is surfaced as a user-facing message

---

### Step 5 — `remote-os` playerctl stub (Linux-conditional) `crates/remote-os/`

**Status: Pending**

#### Deliverables
- `crates/remote-os/src/playerctl.rs` — `PlayerctlController` struct
- Implements `MediaController` using `std::process::Command` to invoke `playerctl`
- All code gated by `#[cfg(target_os = "linux")]`
- `src/main.rs` updated — selects `PlayerctlController` on Linux, stub elsewhere

#### playerctl command mapping
| Trait method | playerctl invocation |
|---|---|
| `toggle_play_pause` | `playerctl play-pause` + `playerctl status` |
| `next` | `playerctl next` |
| `previous` | `playerctl previous` |
| `get_current_state` | `playerctl status` + `playerctl metadata` |

#### Error handling
- Non-zero exit code → `Err("No players found")` or stderr message
- `playerctl` not on PATH → `Err("playerctl not found")`
- Output parse failure → `Err("failed to parse playerctl output: ...")`

#### Test coverage (planned)
- [ ] Compile-only check: `PlayerctlController` implements `MediaController` on Linux
- [ ] `#[cfg(target_os = "linux")]` gate verified — crate compiles on all platforms
- [ ] Runtime tests skipped in CI (no D-Bus / playerctl binary available)

---

## Architecture Diagram

```
src/main.rs  (composition root)
    │
    ├── Config::from_toml()          remote-core/src/config.rs
    │
    ├── whitelist middleware          remote-core/src/bot/dispatch.rs
    │        │
    │        └── drops unauthorized users silently
    │
    ├── handler pipeline             remote-core/src/bot/handler.rs
    │        │
    │        └── generic over T: MediaController
    │
    └── T: MediaController
             ├── MockMediaController  remote-core/src/mock.rs       (tests)
             └── PlayerctlController  crates/remote-os/src/playerctl.rs  (Linux runtime)
```

---

## Security Notes

- The whitelist check happens **before** any handler or media command is invoked.
- An empty whitelist is a startup error — the bot refuses to run.
- Unauthorized messages produce no response and no outbound API call.
- `TELOXIDE_TOKEN` is read from environment, never committed.
- `config.toml` (with `allowed_users`) is excluded from version control.

---

## Milestones

| Step | Description | Status |
|------|-------------|--------|
| 1 | Config parser + whitelist validation | Complete |
| 2 | `MockMediaController` spy | Complete |
| 3 | Handler pipeline + whitelist dispatch | Pending |
| 4 | `/player` UI + inline keyboard + callbacks | Pending |
| 5 | `remote-os` playerctl integration | Pending |
