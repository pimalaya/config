---
cairn: change
id: cmd-alias
status: landed
created: 2026-11-15
---

# `cmd` is the alias of `command`

## Why

The CLI alignment round of 2026-11-15 found `cmd` accepted as an alias of `command` in every secret (src/secret.rs) and in ortie's storage, and refused by name in carillon's and comodoro's hooks. The owner ruled (decision D6): "accept it everywhere instead, cmd is a cmon alias, no point to refuse it". This crate is where the family reads a command, so the rule is stated here once.

Checking every place this crate reads a command found one gap: `sample::variants` counted a secret's `raw` and `command` as one field but not `cmd`, so a sample offering `#password.cmd = ...` beside an active `password.raw` built a variant holding both, which no secret reads.

## What

- The spec states the rule: wherever the crate reads a command under a key, the key is `command` and `cmd` its alias, read the same and written back as `command`; a consumer keying a `CommandConfig` of its own does the same through `#[serde(alias = "cmd")]`.
- `Secret` keeps its alias, now covered by a test reading both keys and writing `command` back.
- `sample::variants` counts a secret's `raw`, `command` and `cmd` as one field.
- The `command` module and `CommandConfig` docs show the key as `command` and name the alias.

`CommandConfig` itself reads a value, never a key, and `notify` reads no command: nothing else to change. The apps with hooks of their own (carillon, comodoro) add the alias on their side.
