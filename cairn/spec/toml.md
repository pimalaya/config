---
cairn: spec
capability: toml
status: current
---

# TOML

The `toml` feature reads a product's TOML configuration: its default paths, the layering of several files, a missing configuration, the account a command runs against, and the deserializers expanding the shell syntax in a string or a path.

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

### Requirement: A missing configuration names where it was looked for
`TomlConfig::load` SHALL fail with a `ConfigMissingError` when the first explicit path does not exist, or when no path is given and no default path exists. The error SHALL carry the path `target_path` answers, the explicit one when given, else the platform default, and the app's `sample_url`, and SHALL read "No configuration found at {path}, start from the sample: {sample_url}". Any other I/O error, a file that exists but cannot be read included, SHALL fail as an I/O error, never as a missing configuration. A later explicit path that cannot be read SHALL be skipped.

#### Scenario: A mistyped explicit path
- **GIVEN** `-c /tmp/typo.toml`, the file absent
- **WHEN** the configuration is loaded
- **THEN** loading fails with "No configuration found at /tmp/typo.toml, start from the sample: <sample URL>"

#### Scenario: A first run
- **GIVEN** no explicit path and no default file
- **WHEN** the configuration is loaded
- **THEN** loading fails with a `ConfigMissingError` naming the platform default path

### Requirement: An account is taken by name or as the default
`TomlConfig::take_account` SHALL take the account named, an absent, empty or `default` name meaning the one account marked default. An unknown name SHALL fail with `AccountError::Missing`, reading "Account `{name}` not found, the configuration holds: {names}", the names sorted and comma-separated, or "the configuration holds no account" when it holds none. No account marked default SHALL fail with `AccountError::NoDefault`, reading "No default account found, name one with `-a <NAME>` or mark one with `default = true`". Several accounts marked default SHALL fail with `AccountError::SeveralDefaults` rather than pick one, reading "Several accounts are marked default ({names}), keep `default = true` on one, or name one with `-a <NAME>`", the names sorted.

#### Scenario: An unknown name
- **GIVEN** accounts `b` and `a`
- **WHEN** account `c` is taken
- **THEN** it fails with "Account `c` not found, the configuration holds: a, b"

#### Scenario: Two defaults
- **GIVEN** accounts `b` and `a`, both marked default
- **WHEN** the default account is taken
- **THEN** it fails naming `a, b`, and takes neither
