---
cairn: change
id: one-dirs-version
status: landed
created: 2026-11-09
---

# One dirs version

## Why

`shell_expanded_string` called `shellexpand::full`, which needs shellexpand 3.1.2's `tilde` feature, and that feature pins dirs below 7. The crate already depends on dirs 7 for its config paths, so every binary carried dirs 6 and 7, and `cargo deny check` warned about the duplicate (seen in ortie on 2026-11-09).

## What

- `shell_expanded_string`, and the path deserializers built on it, call `shellexpand::full_with_context` with the home directory from dirs 7 and the environment as context. A home directory that is not valid Unicode leaves the tilde as written, as the `tilde` feature does, and an undefined variable still leaves the value raw.
- shellexpand is built with `base-0` only.
- Two tests guard the expansion: a leading tilde, a tilde elsewhere, a variable and an undefined one.
