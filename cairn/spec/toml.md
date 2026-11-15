---
cairn: spec
capability: toml
status: current
---

# TOML

The `toml` feature reads a product's TOML configuration: its default paths, the layering of several files, a missing configuration, the account a command runs against, the deserializers expanding the shell syntax in a string or a path, and the sentence refusing a key a release removed, which an unknown key's error leaves out of the keys it lists.

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
`TomlConfig::take_account` SHALL take the account named, an absent or empty name meaning the one account marked default; `default` is a name like any other. An unknown name SHALL fail with `AccountError::Missing`, reading "Account `{name}` not found, the configuration holds: {names}", the names sorted and comma-separated, or "the configuration holds no account" when it holds none, followed by ", omit `-a` to take the account marked default" when the name is `default`. No account marked default SHALL fail with `AccountError::NoDefault`, reading "No default account found, name one with `-a <NAME>` or mark one with `default = true`". Several accounts marked default SHALL fail with `AccountError::SeveralDefaults` rather than pick one, reading "Several accounts are marked default ({names}), keep `default = true` on one, or name one with `-a <NAME>`", the names sorted.

#### Scenario: An unknown name
- **GIVEN** accounts `b` and `a`
- **WHEN** account `c` is taken
- **THEN** it fails with "Account `c` not found, the configuration holds: a, b"

#### Scenario: Two defaults
- **GIVEN** accounts `b` and `a`, both marked default
- **WHEN** the default account is taken
- **THEN** it fails naming `a, b`, and takes neither

#### Scenario: The name `default`
- **GIVEN** accounts `default` and `b`, `b` marked default
- **WHEN** account `default` is taken
- **THEN** it takes `default`

### Requirement: A removed key is refused in one sentence
`toml::Removed` SHALL read "The `{name}` key is removed, {why}: {instead}, see MIGRATION.md" for a `Key`, and the same with `table` in place of `key` for a `Table`. `Removed::error` SHALL answer that sentence as the format's deserialization error. A type declared through `removed_key!` SHALL deserialize from no value: a field `Option<T>` of it SHALL read `None` when its key is absent, SHALL fail the load with the sentence when the key is present, whatever it holds, and SHALL be written as nothing.

#### Scenario: A removed table
- **GIVEN** `removed_key!` declaring `TableRemoved: Table { name: "card.list.table", why: "the output being plain text", instead: "delete it" }`, the type of a field `table`
- **WHEN** a configuration holding `[table]` with any key is read
- **THEN** reading fails with "The `card.list.table` table is removed, the output being plain text: delete it, see MIGRATION.md"

#### Scenario: No removed key
- **GIVEN** the same field
- **WHEN** a configuration without `table` is read and written back
- **THEN** the field reads `None` and nothing is written for it

### Requirement: An unknown key lists no removed one
When loading through `TomlConfig` fails on a key no field of a table takes, the error SHALL read as serde's "unknown field `{key}`, expected ..." without the keys that table refuses as removed: a listed key SHALL leave the list when the configuration type, given a document holding only that key at that table, fails with the sentence of `toml::Removed`. The list SHALL be written as serde writes it, "expected `{a}`", "expected `{a}` or `{b}`", "expected one of `{a}`, `{b}`, ...", or "there are no fields" when every key is removed. A key the same type reads elsewhere SHALL stay listed there. The error SHALL keep the location the TOML error shows. When the table cannot be told, the error SHALL stay as serde wrote it.

#### Scenario: A key next to a removed one
- **GIVEN** an account table reading `default` and `imap`, and refusing `mailbox` and `watch` as removed
- **WHEN** a configuration holding `host` under the account is loaded
- **THEN** loading fails with "unknown field `host`, expected `default` or `imap`", at the line of `host`

#### Scenario: A key the account refuses, read under a backend
- **GIVEN** the same account, its `imap` table reading `host` and `mailbox`, and refusing `hook` as removed
- **WHEN** a configuration holding `port` under `imap` is loaded
- **THEN** loading fails with "unknown field `port`, expected `host` or `mailbox`"
