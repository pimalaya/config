---
cairn: log
change: hook-timeout
landed: 2026-11-18
---

# A hook command may carry a timeout

Finding CRL-2 of the CLI family analysis of 2026-11-15, whose direction asks for an optional `timeout` in pimalaya-config's command table, family-wide by cli-008: a carillon hook runs inline with no timeout, so one that hangs holds the watch loop and a shutdown for good. No default, since the hook the carillon sample recommends (`neverest sync`) runs for minutes on a first sync.

- src/command.rs: `Timeout(pub Duration)`, read and written through `crate::duration`, a zero refused naming the key to omit instead, and `Timeout::GRACE` (5 seconds), the time a command has to end after `SIGTERM`. It is a field type, not a table: comodoro's `TimerHook` and carillon's `ItemHook` and `FlagHook` hold `command: Option<CommandConfig>` under `deny_unknown_fields`, which `#[serde(flatten)]` would defeat, so each adds `timeout: Option<Timeout>` beside `command` when it enforces it. Until then their tables refuse `timeout` as an unknown key, as today: nothing breaks.
- src/lib.rs: `duration` builds under `secret` as well as `toml`, the command module needing it; it depends on serde alone.
- The spec states what a caller does, since the crate runs no hook: `SIGTERM` to the command's process group, `SIGKILL` after the grace, reaped either way, reported as a failed hook naming the timeout; on Windows a kill at once.
- Tests: `a_timeout_reads_as_a_duration_and_is_optional` (integer, `"90s"`, `"5m"`, absent, written back as `"5m"`) and `a_timeout_of_zero_or_no_duration_is_refused`, in a table under `deny_unknown_fields`.

Left to the consumers: carillon and comodoro add the field and the stopping logic, carillon with the rest of CRL-2 (null stdin, the hook off the watch loop).

Capabilities moved: command (A hook command may carry a timeout, A caller stops a command that outlives its timeout, added).
