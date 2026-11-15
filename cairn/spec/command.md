---
cairn: spec
capability: command
status: current
---

# Commands

The `command` module reads a command a configuration holds, a shell line string or a program-plus-arguments list, as a `CommandConfig` written back in the shape it was read.

### Requirement: `cmd` is the alias of `command`
Wherever pimalaya-config reads a command under a key, the key SHALL be `command` and `cmd` SHALL be accepted as its alias, read the same and written back as `command`: the command of a `Secret`, and the field a line sets for `sample::variants`. A consumer keying a `CommandConfig` of its own SHALL name the key `command` and accept `cmd` the same way (`#[serde(alias = "cmd")]`).

#### Scenario: A secret written with `cmd`
- **GIVEN** `password.cmd = "pass show mail"`
- **WHEN** it is read as a `Secret`
- **THEN** it is the shell command `pass show mail`, written back as `command = "pass show mail"`
