---
cairn: log
change: notification-template
landed: 2026-11-05
---

# Notification templates

Added `notify::NotificationTemplate` and `NotificationTemplateError` in src/notify.rs: the `{ summary, body }` table, held as text, rendered through a caller's lookup with the shell's syntax (`$name`, `${name}`, `${name:-default}`, `$$`), checked against declared names, every other `${...}` form refused when read. Spec opened: `notify` (ADDED: the four requirements of the change).

The expansion is a small parser of its own rather than shellexpand: the four forms exactly, `${name:default}` refused rather than looked up, and no dependency added to a `notify`-only build. shellexpand stays under `toml` alone.

The test of a build without `notify` expected "Missing `notify` cargo feature, rebuild with it to send notifications" while the code says "Missing `notify` cargo feature"; both came from 7e97d379, comodoro reads the code's, so the test was aligned to it. The module doctest still needs the `toml` feature (it calls `toml::from_str`), so `cargo test --no-default-features` fails on it as it did before this change.
