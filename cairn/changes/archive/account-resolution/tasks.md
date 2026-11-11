---
cairn: tasks
change: account-resolution
---

- [x] src/toml.rs: `sample_url`, `accounts`, `load`, `take_account` answering an `AccountError`, `ConfigMissingError`; `take_default_account` removed
- [x] src/toml.rs: `to_string`, `emit` and `bare_or_quoted` removed
- [x] Cargo.toml: `toml` feature pulls thiserror; toml without `display`, a dev-dependency with it
- [x] Docs: src/toml.rs header, `from_paths_or_default`, `target_path`, src/lib.rs header, README.md
- [x] Tests: missing configuration (explicit path, defaults), skipped later path, named, unknown, default, no default, several defaults
- [x] fmt, clippy `-D warnings` (all features, none, each alone), `cargo test --all-features`, `cargo doc --all-features --no-deps`
- [x] CHANGELOG.md
- [x] Delta folded, log written, change archived
