---
cairn: tasks
change: notification-template
---

- [x] Proposal and delta reviewed (user)
- [x] `NotificationTemplate` with its serde shape, `render`, `check`, `summary`, `body`, in every build
- [x] Expansion: a parser of the four forms (shellexpand left under `toml`); tests for each scenario
- [x] No shellexpand under `notify`: not needed by the parser
- [x] Crate docs: the template, the camelCase rule (guideline cli-008); README feature list
- [x] `cargo fmt`, `cargo test --all-features`, `cargo clippy --all-features`, `cargo build --no-default-features`, and `notify` alone (the module doctest needs `toml`, as before)
- [x] Fold into spec/notify.md, log entry, CHANGELOG under Unreleased (Added)
- [ ] Release (user)
- [ ] docs/README.md, empty, superseded by cairn/ (the user decides on removing it)
