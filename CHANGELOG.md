# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Relicensed under `MIT OR Apache-2.0`, adding the Apache-2.0 option next to the existing MIT license.
- Gated the `anyhow` and `log` dependencies behind the `toml` feature, the only layer using them, so a `secret`-only build no longer pulls them in.

### Fixed

- Fixed `shell_expanded_string` and `shell_expanded_path` not expanding environment variables when the deserializer handed over a borrowed string: the visitor implemented only `visit_string`, so borrowed inputs fell through to the erroring default. It now expands on `visit_str`, covering both cases.

## [0.0.2] - 2026-07-13

### Changed

- Reworked the `Command` serde adapter so a shell-form command round-trips as its bare command line string instead of the verbose `["/bin/sh", "-c", ...]` sequence; a program-plus-arguments command still round-trips as a string sequence.

  Added the `shell` helper building a platform-shell command, so callers writing the command line themselves match the deserializer semantics.

## [0.0.1] - 2026-06-06

### Added

- Added the `TomlConfig` loader trait: reads and deep-merges a project's TOML configuration from explicit paths, falls back to the platform default locations, and distinguishes a missing file (mapped to `Ok(None)` to drive a wizard) from an unreadable one.
- Added the `Secret` enum resolving a secret from a literal value or a shell command's standard output, lazily at use time.
- Added the `command` serde adapter reading a `std::process::Command` from a shell line string or a program-plus-arguments list.
- Added the `shell_expanded_string` and `shell_expanded_path` deserializers expanding environment variables in string and path config fields.

[unreleased]: https://github.com/pimalaya/config/compare/v0.0.2..HEAD
[0.0.2]: https://github.com/pimalaya/config/compare/v0.0.1..v0.0.2
[0.0.1]: https://github.com/pimalaya/config/compare/root...v0.0.1
