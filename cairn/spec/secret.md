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
A secret command exiting with a non-zero status SHALL fail with its trimmed standard error, or with its program and exit status when it wrote none. Its standard output, where the secret comes out, SHALL NOT reach the error, its `Display` nor its `Debug`, even when the command printed the secret before failing.

#### Scenario: A hook failing after the lookup
- **GIVEN** the secret command `pass show mail && notify-send ok`, whose lookup prints the password and whose hook then fails, writing nothing on standard error
- **WHEN** it is resolved
- **THEN** the error names `/bin/sh` and its exit status, and holds no part of the password

#### Scenario: A prompting command under a terminal
- **GIVEN** a secret command asking for a password on its terminal, such as `systemd-ask-password`
- **WHEN** it is resolved by a caller whose standard input is a terminal
- **THEN** the prompt reads from that terminal, and the typed value is the secret

#### Scenario: Piped input
- **GIVEN** a caller reading a message or a JSON-RPC stream on its standard input from a pipe
- **WHEN** a secret command is resolved
- **THEN** the command reads an empty standard input, and the piped bytes stay with the caller
