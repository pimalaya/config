---
cairn: log
change: account-resolution
landed: 2026-11-11
---

# Account resolution

`TomlConfig` (src/toml.rs) resolves the account a command runs against as guideline cli-002 asks, in place of the copies in Ortie 3.0 (src/cli.rs `take_account`) and Himalaya (src/cli.rs `resolve_account`, src/config.rs `take_account`), whose wording it keeps. The app provides `sample_url` and `accounts` (every name with its `default` flag) beside `project_name` and `take_named_account`; `take_default_account` is gone.

- `load` answers the configuration or a `ConfigMissingError { path, sample_url }`, the path being `target_path`'s: the first `-c` path, else the base layer, else the platform default. `from_paths_or_default` keeps its `Option` answer, now documented without a wizard, and a later `-c` path that cannot be read is still skipped with a debug log.
- `take_account` answers `Result<(String, Account), AccountError>`: `Missing { name, names }` with the names sorted, `NoDefault`, and `SeveralDefaults(names)` sorted. Ortie used to pick one default at random among several, its accounts sitting in a `HashMap`; Himalaya refused by hand, and now every app does.
- `toml::to_string`, the compact serializer the wizards printed an account with, is removed with its helpers: its callers (himalaya, cardamum, calendula, neverest, unfog, carillon, sirup, himalaya-tui) all sit in wizards, which the family drops. toml's `display` feature went with it; toml is a dev-dependency with it for the tests, which also makes the `secret` feature's tests build alone (they failed to find toml before).
- thiserror joins the `toml` feature for the two error types.

Capabilities moved: toml (cairn/spec/toml.md: "A missing configuration names where it was looked for" and "An account is taken by name or as the default" added).

Verified: `cargo fmt`, `cargo clippy --all-targets -D warnings` with all features, none, the defaults and each feature alone, `cargo test --all-features` (49 unit tests and the doc test), `cargo test --no-default-features --features secret`, `cargo doc --all-features --no-deps`, `-D missing_docs`, cairn/verify.sh.
