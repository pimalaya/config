---
cairn: tasks
change: removed-keys-unlisted
---

- [x] `TomlConfig::from_paths` rewrites an unknown-field error, one file or several layers
- [x] Read serde's message back, find the tables failing as the document did, try each listed key alone there, write the list again as serde does
- [x] The rewritten error displays as the TOML one, location kept
- [x] Regression tests failing on 0.5.1: an unknown key next to a removed one, at the account level and under a backend reading a key the account refuses, one file and two layers; the lists left with one key, several, none
- [x] Docs: the `toml` module header and `from_paths`
- [x] Spec (toml.md), log, CHANGELOG
