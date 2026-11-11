# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **BREAKING**: Added the required `toml::TomlConfig::sample_url`, the documented sample configuration a missing one points to.
- **BREAKING**: Added the required `toml::TomlConfig::accounts`, every account name with whether it is marked default.
- Added `toml::TomlConfig::load`, failing with a `toml::ConfigMissingError` when no configuration exists.

  The error names the path looked for, the first `-c` one or the default location, and the sample URL.

- Added `toml::AccountError`, the unknown, missing default and several default accounts `toml::TomlConfig::take_account` refuses.
- Added the `sample` cargo feature and `sample::variants`, the variants a user may write from a documented sample configuration, for its tests.

### Changed

- **BREAKING**: Changed `toml::TomlConfig::take_account` to answer the account or a `toml::AccountError`, never `Ok(None)`.

  An unknown name lists the accounts the configuration holds, sorted, and several accounts marked default are refused rather than one picked at random.

### Removed

- **BREAKING**: Removed the required `toml::TomlConfig::take_default_account`: `take_account` finds the default through `accounts`.
- **BREAKING**: Removed `toml::to_string`, the compact serializer of the wizards.

## [0.3.1] - 2026-11-11

### Added

- Added `notify::NotificationTemplate`, a notification whose summary and body hold variables in the shell's syntax (`$name`, `${name}`, `${name:-default}`, `$$`).

  It reads the table `notify::Notification` reads. `render` fills the variables through a lookup the caller gives, `check` refuses a variable the caller does not declare, and a malformed `${...}` is refused when the template is read.

### Changed

- Changed the shell expansion of `toml::shell_expanded_string` and the path deserializers to take the home directory from dirs 7 rather than shellexpand's `tilde` feature, which pinned dirs 6: a binary carries one dirs version.

## [0.3.0] - 2026-11-03

### Added

- Added `toml::TomlConfig::default_paths`, every default config path candidate.
- Added `toml::TomlConfig::valid_default_paths`, the existing default paths, deduplicated.

### Changed

- **BREAKING**: Changed `toml::TomlConfig` to load every existing default path as a layer, not only the first.

  `$HOME/.<project>rc` merges last and overrides the platform files. A file reached by two paths is read once.

- Changed TOML read and parse errors to name the failing config file.

### Removed

- **BREAKING**: Removed `toml::TomlConfig::first_valid_default_path`, replaced by `valid_default_paths`.

### Fixed

- Fixed the empty `Secret command error` of a command failing silently ([pimalaya/himalaya#745](https://github.com/pimalaya/himalaya/issues/745)).

  A `secret-tool lookup` missing its entry prints nothing. The error now names the program and its exit status.

## [0.2.1] - 2026-10-02

### Added

- Added `toml::opt_shell_expanded_path`, the optional variant of `toml::shell_expanded_path`.

## [0.2.0] - 2026-08-29

### Added

- Added `command::CommandConfig`, a shell line or a program and its arguments, as the configuration wrote it.

  Unlike `std::process::Command`, it is comparable, hashable and clonable. `to_command` builds the runnable command.

- Added `secret::SecretResolver`, running each distinct secret command once.

  A `pass` entry shared by an account's IMAP and SMTP tables now costs one key unlock.

### Changed

- **BREAKING**: Changed `secret::Secret::Command` to carry a `command::CommandConfig` instead of a `std::process::Command`.

  Configuration files parse and serialize unchanged.

- Changed secret command resolution to log its spawn duration at `debug`.

### Fixed

- Fixed `toml::TomlConfig::target_path` naming the platform path while another default file is in effect.

  A wizard writing there shadowed every account of an existing `$HOME/.<project>rc`. It now names the file in effect.

## [0.1.4] - 2026-08-14

### Changed

- Raised the minimum supported Rust version from 1.87 to 1.89.

  The `notify` feature reaches `time`, whose RUSTSEC-2026-0009 fix needs 1.88.

## [0.1.3] - 2026-08-14

### Changed

- Changed the `notify` module to compile without the `notify` feature.

  `notify::Notification` then holds nothing, and reading, writing or showing one fails with an error naming the feature.

- Changed serde to a non-optional dependency.

## [0.1.2] - 2026-08-14

### Added

- Added the `notify` serde adapter, reading a `notify_rust::Notification` from a summary and a body.

  It sits behind the `notify` feature. The `vendored` feature forwards to notify-rust for platforms without a system D-Bus.

## [0.1.1] - 2026-07-25

### Added

- Added `toml::to_string`, a TOML serializer tuned for wizard output.

  Per-account tables are the only headers, every other table is flattened into dotted keys, and empty tables are dropped.

## [0.1.0] - 2026-07-16

### Changed

- Changed the toml dependency from 0.8 to 1.1.
- Replaced the `serde-toml-merge` dependency with an inlined deep-merge.
- Relicensed under `MIT OR Apache-2.0`.
- Gated the `anyhow` and `log` dependencies behind the `toml` feature.

### Fixed

- Fixed `toml::shell_expanded_string` and `toml::shell_expanded_path` failing on borrowed strings.

## [0.0.2] - 2026-07-13

### Added

- Added `command::shell`, building a platform-shell command.

### Changed

- Changed the `command` serde adapter to write a shell command back as its bare command line.

## [0.0.1] - 2026-06-06

### Added

- Added the `toml::TomlConfig` loader trait.

  It deep-merges TOML files from explicit or default paths, and maps a missing file to `Ok(None)` to drive a wizard.

- Added `secret::Secret`, a literal value or a shell command output, resolved lazily.
- Added the `command` serde adapter, reading a `std::process::Command` from a shell line or a program and its arguments.
- Added `toml::shell_expanded_string` and `toml::shell_expanded_path`, expanding environment variables in config fields.

[unreleased]: https://github.com/pimalaya/config/compare/v0.3.1..HEAD
[0.3.1]: https://github.com/pimalaya/config/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/pimalaya/config/compare/v0.2.1..v0.3.0
[0.2.1]: https://github.com/pimalaya/config/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/config/compare/v0.1.4..v0.2.0
[0.1.4]: https://github.com/pimalaya/config/compare/v0.1.3..v0.1.4
[0.1.3]: https://github.com/pimalaya/config/compare/v0.1.2..v0.1.3
[0.1.2]: https://github.com/pimalaya/config/compare/v0.1.1..v0.1.2
[0.1.1]: https://github.com/pimalaya/config/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/pimalaya/config/compare/v0.0.2..v0.1.0
[0.0.2]: https://github.com/pimalaya/config/compare/v0.0.1..v0.0.2
[0.0.1]: https://github.com/pimalaya/config/compare/root...v0.0.1
