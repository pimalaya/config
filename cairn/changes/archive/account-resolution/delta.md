---
cairn: delta
change: account-resolution
---

## ADDED Requirements

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
`TomlConfig::take_account` SHALL take the account named, an absent, empty or `default` name meaning the one account marked default. An unknown name SHALL fail with `AccountError::Missing`, reading "Account `{name}` not found, the configuration holds: {names}", the names sorted and comma-separated. No account marked default SHALL fail with `AccountError::NoDefault`, reading "No default account found, name one with `-a <NAME>` or mark one with `default = true`". Several accounts marked default SHALL fail with `AccountError::SeveralDefaults` rather than pick one, reading "Several accounts are marked default ({names}), keep `default = true` on one, or name one with `-a <NAME>`", the names sorted.

#### Scenario: An unknown name
- **GIVEN** accounts `b` and `a`
- **WHEN** account `c` is taken
- **THEN** it fails with "Account `c` not found, the configuration holds: a, b"

#### Scenario: Two defaults
- **GIVEN** accounts `b` and `a`, both marked default
- **WHEN** the default account is taken
- **THEN** it fails naming `a, b`, and takes neither

## MODIFIED Requirements

## REMOVED Requirements
