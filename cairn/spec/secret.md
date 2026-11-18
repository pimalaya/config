---
cairn: spec
capability: secret
status: current
---

# Secrets

The `secret` module resolves a `Secret`: a literal read from the configuration, or a command whose standard output holds the value.

### Requirement: The secret command reads a terminal, never piped input
`Secret::get` SHALL hand the secret command the caller's standard input when it is a terminal, so a command prompting for a password can ask for it, and a null one otherwise, so input piped to the caller never reaches the command. The command's standard output and standard error SHALL be captured: the first output line is the value, and a failure reports them as the next requirement says.

### Requirement: A failing secret command never reports its standard output
A secret command exiting with a non-zero status SHALL fail with its trimmed standard error, or with its program and exit status when it wrote none. Its standard output, where the secret comes out, SHALL NOT reach the error, its `Display` nor its `Debug`, even when the command printed the secret before failing. The error SHALL end with ", run `{command}` yourself to see why", the command as configured: a shell line as written, a program and its arguments joined by spaces, each quoted for a POSIX shell when it needs to be. It SHALL name nothing else of the run, no output and no environment.

This is how a helper of the family reports a failure to an app resolving it (finding COH-1, ruled 2026-11-18): a family program prints its failure on standard output, which the error never reads, so the user runs the command to read it. Standard error stays for logs, moving failures there having broken callers before (vim among them); standard output holds the result or the failure, family-wide; and no environment variable or configuration option tells a helper it runs as a secret command.

#### Scenario: A hook failing after the lookup
- **GIVEN** the secret command `pass show mail && notify-send ok`, whose lookup prints the password and whose hook then fails, writing nothing on standard error
- **WHEN** it is resolved
- **THEN** the error names `/bin/sh` and its exit status, and holds no part of the password, then ends with ", run `pass show mail && notify-send ok` yourself to see why"

#### Scenario: A token helper failing on its standard output
- **GIVEN** the secret command `ortie token show`, no token stored, ortie printing its failure on standard output and nothing on standard error
- **WHEN** it is resolved
- **THEN** the error reads "Secret command error: /bin/sh failed with exit status: 1, run `ortie token show` yourself to see why"

#### Scenario: A prompting command under a terminal
- **GIVEN** a secret command asking for a password on its terminal, such as `systemd-ask-password`
- **WHEN** it is resolved by a caller whose standard input is a terminal
- **THEN** the prompt reads from that terminal, and the typed value is the secret

#### Scenario: Piped input
- **GIVEN** a caller reading a message or a JSON-RPC stream on its standard input from a pipe
- **WHEN** a secret command is resolved
- **THEN** the command reads an empty standard input, and the piped bytes stay with the caller

### Requirement: A refused secret never echoes its value
`Secret` SHALL deserialize from a table holding one key: `raw`, a string, the literal value, or `command`, `cmd` its alias, a command. A value of another type where the table belongs, a bare string, a number or a boolean among them, SHALL be refused with "a secret is a table: `raw` for a literal value, `command` for a command printing it", an empty table likewise; a `raw` holding no string with "a secret's `raw` is a string"; another key with "unknown variant `{key}`, expected `raw` or `command`"; both keys with "a secret holds one of `raw` or `command`, not both". No refusal SHALL hold the value written.

#### Scenario: A literal written without `raw`
- **GIVEN** `password = "hunter2"`
- **WHEN** it is read as a `Secret`
- **THEN** reading fails with "a secret is a table: `raw` for a literal value, `command` for a command printing it", which holds no part of `hunter2`
