---
cairn: tasks
change: one-dirs-version
---

- [x] src/toml.rs: `shellexpand::full_with_context` with dirs 7's home directory and the environment
- [x] Cargo.toml: shellexpand without `tilde`
- [x] Tests: tilde and variable expansion, an undefined variable left raw
- [x] fmt, clippy `-D warnings` (all features, `toml` alone), `cargo test --all-features`, `cargo deny check`, `cargo tree -i dirs@6.0.0` empty
- [x] Delta folded, log written, change archived
