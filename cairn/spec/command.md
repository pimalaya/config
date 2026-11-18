---
cairn: spec
capability: command
status: current
---

# Commands

The `command` module reads a command a configuration holds, a shell line string or a program-plus-arguments list, as a `CommandConfig` written back in the shape it was read, and the `Timeout` a hook command may run under.

### Requirement: `cmd` is the alias of `command`
Wherever pimalaya-config reads a command under a key, the key SHALL be `command` and `cmd` SHALL be accepted as its alias, read the same and written back as `command`: the command of a `Secret`, and the field a line sets for `sample::variants`. A consumer keying a `CommandConfig` of its own SHALL name the key `command` and accept `cmd` the same way (`#[serde(alias = "cmd")]`).

#### Scenario: A secret written with `cmd`
- **GIVEN** `password.cmd = "pass show mail"`
- **WHEN** it is read as a `Secret`
- **THEN** it is the shell command `pass show mail`, written back as `command = "pass show mail"`

### Requirement: A hook command may carry a timeout
`command::Timeout` SHALL read as `duration::deserialize` reads a duration (an integer of seconds, or `"90s"`, `"5m"` and the other units) and SHALL be written back as `duration::serialize` writes one. A duration of zero SHALL be refused with "Invalid timeout of zero, omit `timeout` for a command running until it ends". There is no default: a consumer SHALL read it as an optional field `timeout: Option<Timeout>` of the table holding the hook's `command`, an absent key meaning the command runs until it ends, and that table keeps refusing unknown keys.

#### Scenario: A first sync allowed half an hour
- **GIVEN** a hook table holding `command = "neverest sync"` and `timeout = "30m"`
- **WHEN** it is read
- **THEN** its timeout is 1800 seconds, written back as `"30m"`

#### Scenario: A zero timeout
- **GIVEN** a hook table holding `timeout = 0`
- **WHEN** it is read
- **THEN** reading fails with "Invalid timeout of zero, omit `timeout` for a command running until it ends"

### Requirement: A caller stops a command that outlives its timeout
A consumer running a hook command under a `Timeout` SHALL, once the duration has passed since the spawn and the command still runs, send it `SIGTERM`, then kill it (`Child::kill`, `SIGKILL`) if it still runs `Timeout::GRACE` (5 seconds) later, and SHALL reap it either way. On Unix it SHALL spawn the command in a process group of its own and signal that group, so the children of a shell line stop with it. Where there is no `SIGTERM`, on Windows, it SHALL kill the command at once. A command stopped this way SHALL be reported as a failed hook naming its timeout, never as a success.
