# Contributing to tg-media-remote

Thank you for your interest in contributing.

## Development setup

```bash
git clone <repo>
cd tg-media-remote
cargo test        # all tests must pass before you start
```

No system dependencies are required to run the tests — no Telegram token, no `playerctl`, no D-Bus.

## Development protocol (SDD)

This project follows Spec-Driven Development. Changes must follow this order — do not skip ahead:

1. Write or update the spec / test first
2. Implement the minimum code to make it pass
3. Verify with `cargo test`

See [ROADMAP.md](ROADMAP.md) for the full implementation plan and step definitions.

## Rules

- **All tests must pass:** `cargo test` must exit zero before opening a PR.
- **No new dependencies without discussion:** the core crate (`remote-core`) must remain free of system dependencies and compile on any platform.
- **Bot logic stays in `remote-core`:** never import `remote-os` types into `remote-core`. The `MediaController` trait is the only interface between the layers.
- **No `config.toml` in commits:** this file contains personal Telegram user IDs. It is listed in `.gitignore`.
- **No `TELOXIDE_TOKEN` in commits:** store the token in the environment only.

## Code style

- Format with `cargo fmt` before committing.
- No warnings: code must compile cleanly under `cargo clippy -- -D warnings`.
- Doc comments (`///`) on all public types and functions.
- Tests live in `#[cfg(test)]` modules inside the same file as the code they test.

## Pull requests

- Keep PRs focused on a single step from the roadmap.
- Include a short description of what changed and why.
- Reference the relevant roadmap step (e.g. "Step 4 — inline keyboard").

## Reporting issues

Please include:
- Your Linux distribution and `playerctl` version (`playerctl --version`)
- Rust toolchain version (`rustc --version`)
- The full error output
