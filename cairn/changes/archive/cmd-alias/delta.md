---
cairn: delta
change: cmd-alias
---

## ADDED Requirements

### Requirement: `cmd` is the alias of `command`
Wherever pimalaya-config reads a command under a key, the key SHALL be `command` and `cmd` SHALL be accepted as its alias, read the same and written back as `command`: the command of a `Secret`, and the field a line sets for `sample::variants`. A consumer keying a `CommandConfig` of its own SHALL name the key `command` and accept `cmd` the same way (`#[serde(alias = "cmd")]`).

#### Scenario: A secret written with `cmd`
- **GIVEN** `password.cmd = "pass show mail"`
- **WHEN** it is read as a `Secret`
- **THEN** it is the shell command `pass show mail`, written back as `command = "pass show mail"`

## MODIFIED Requirements

### Requirement: Each section is a base variant
`sample::variants(sample, sections)` SHALL answer, for each heading of `sections` in order, the sample with the paragraph that heading opens (up to the next blank line) uncommented: one `#` off every line holding code, a closing bracket or a further `#`, the prose lines, the heading among them, staying comments. Each line of code it uncomments SHALL comment out the active lines setting the same field, an active array through its closing bracket, a secret's `raw`, `command` and `cmd` counting as one field. The variant SHALL be named after the heading. A heading opening no paragraph SHALL panic, naming it. An empty `sections` SHALL make the sample as written the one base, named `as written`.

#### Scenario: A secret offered as a literal and as a command
- **GIVEN** a sample holding `a.raw = "x"`, then `#a.command = "y"` and `#a.cmd = "z"`
- **WHEN** its variants are built
- **THEN** the variants uncommenting `a.command` and `a.cmd` each comment `a.raw` out, and leave the other commented

## REMOVED Requirements
