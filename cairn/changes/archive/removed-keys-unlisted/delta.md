---
cairn: delta
change: removed-keys-unlisted
---

## ADDED Requirements

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

## MODIFIED Requirements

## REMOVED Requirements
