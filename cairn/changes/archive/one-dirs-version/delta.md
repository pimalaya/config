---
cairn: delta
change: one-dirs-version
---

## ADDED Requirements

### Requirement: Shell-expanded strings
`shell_expanded_string`, `shell_expanded_path` and `opt_shell_expanded_path` SHALL expand environment variables (`$name`, `${name}`) and a leading tilde, the tilde to the home directory dirs reports. A home directory that is not valid Unicode SHALL leave the tilde as written, and an undefined variable SHALL leave the whole value as written rather than fail.

#### Scenario: A path under the home directory
- **GIVEN** a field read through `shell_expanded_string` holding `~/x`
- **WHEN** it is deserialized
- **THEN** it holds the home directory followed by `/x`

#### Scenario: An undefined variable
- **GIVEN** a field holding `$UNDEFINED/x`, the variable unset
- **WHEN** it is deserialized
- **THEN** it holds `$UNDEFINED/x`

### Requirement: One dirs version
shellexpand SHALL be built without its `tilde` feature, the home directory coming from the crate's own dirs dependency, so a binary carries one dirs version.

## MODIFIED Requirements

## REMOVED Requirements
