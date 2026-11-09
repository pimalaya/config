---
cairn: log
change: one-dirs-version
landed: 2026-11-09
---

# One dirs version

`shell_expanded_string` (src/toml.rs) calls `shellexpand::full_with_context` with dirs 7's home directory and the environment, in place of `shellexpand::full`, and shellexpand is built with `base-0` only. Its `tilde` feature pinned dirs below 7, so every binary carried dirs 6 beside the dirs 7 this crate uses; `cargo tree -i dirs@6.0.0` now finds nothing. The behaviour is unchanged: a home directory that is not valid Unicode leaves the tilde as written, as the `tilde` feature did, and an undefined variable leaves the value raw. `a_leading_tilde_expands_to_the_home_directory` and `variables_expand_and_an_undefined_one_leaves_the_value_raw` guard it.

Capabilities moved: toml (cairn/spec/toml.md, new: "Shell-expanded strings" and "One dirs version" added).

Verified: `cargo fmt`, `cargo clippy --all-targets -D warnings` with all features and with `toml` alone, `cargo test --all-features` (36 unit tests and the doc test), `cargo deny check`, cairn/verify.sh.
