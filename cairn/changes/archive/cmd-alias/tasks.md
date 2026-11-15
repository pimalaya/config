---
cairn: tasks
change: cmd-alias
---

- [x] Every place the crate reads a command checked: `Secret` (alias held), `CommandConfig` and its adapter (a value, no key), `notify` (no command), `sample::variants` (gap)
- [x] `sample::variants` counts a secret's `cmd` with its `raw` and `command`, with a test failing on the old rule
- [x] A test reading a secret under `cmd` and `command`, written back as `command`
- [x] Docs: the `command` module, `CommandConfig`, the lib.rs header, the README
- [x] Spec (command.md added, sample.md modified), log, CHANGELOG
