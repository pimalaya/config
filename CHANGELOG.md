# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `command::CommandConfig`, a command as a configuration writes it: a shell line, or a program and its arguments. It is the shape a field holding a command should store, being comparable, hashable and cheap to clone, none of which a built `std::process::Command` is, and it becomes one through `to_command` at the moment something runs it. It writes back the shape it was read as, having kept it, where the `std::process::Command` adapter has to recover that from the built command.

- Added `secret::SecretResolver`, resolving secrets while memoizing the commands it spawns, so a command named by several configuration fields is run once instead of once per field. A `pass` or `gpg` entry shared by an account's IMAP and SMTP tables now costs one key unlock per resolver rather than one per field. Distinct is `CommandConfig`'s own equality, which never crosses the two shapes: a shell line and the argv spelling that runs it through the platform shell are two commands, whatever they end up running. The resolver holds plaintext for as long as it lives and is meant to be dropped with the runtime account it builds.

### Changed

- **BREAKING**: `Secret::Command` carries a `command::CommandConfig` instead of a `std::process::Command`.

  A built command is neither comparable, hashable nor clonable, and has forgotten the shape it came from, so a secret holding one could not be compared without rebuilding a shape out of it, nor cloned without copying it field by field. Keeping what the configuration wrote makes both trivial and removes a class of bug with them: the environment overrides and working directory a built command could carry were dropped by the clone and silently discarded by the serializer, and are now simply not representable. A caller constructs `CommandConfig::Shell(line)` or `CommandConfig::Argv { program, args }` where it built a `Command` before.

  The `command` module keeps serving `std::process::Command` for a field that wants the runnable form directly (`#[serde(with = "pimalaya_config::command")]`), and `command::shell` is unchanged. Both TOML shapes parse and serialize exactly as before, so no configuration file changes.

- Logged secret command resolution at `debug`, with the time the spawn took, since a locked `gpg-agent` answers in seconds and the wait was previously silent at every log level. Neither the value nor the command arguments are logged, a command line being free to carry the secret itself.

### Fixed

- **`TomlConfig::target_path` named a file that shadows the one being read.** It answered with the platform path (`$XDG_CONFIG_HOME/<project>/config.toml`) whenever no explicit path was given, while `from_paths_or_default` reads the *first* default path that exists and merges nothing. For a project configured through `$HOME/.<project>rc`, the two disagree: a wizard handed the platform path writes a second configuration that takes precedence and hides every account in the first, and the caller's clobber guard cannot fire, the path it was given being exactly the one that does not exist yet. It now answers with the explicit path when one was given, else the configuration already in effect, else the platform path, so a first run still lands where it always did.

## [0.1.4] - 2026-08-14

### Changed

- Raised the minimum supported Rust version from 1.87 to 1.89.

  The `notify` feature reaches `time` through notify-rust's macOS backend, and the release fixing RUSTSEC-2026-0009, a denial of service parsing hostile RFC 2822 input, needs 1.88. The crate settles on 1.89, the version the newest Pimalaya binaries already require.

## [0.1.3] - 2026-08-14

### Changed

- Compiled the `notify` module unconditionally, so a consumer names `notify::Notification` and shows one with no `#[cfg]` of its own.

  With the `notify` feature, that type is notify-rust's own and its `show` reaches the daemon. Without it, the type holds nothing, and reading one from a configuration, writing one back and showing one all fail with the same message, naming the `notify` cargo feature to rebuild with. A configuration naming a notification a build cannot send is refused as it loads, pointing at the offending table.

  serde became a non-optional dependency along the way, since the module needs it in every build.

## [0.1.2] - 2026-08-14

### Added

- Added the `notify` serde adapter, reading a `notify_rust::Notification` from a summary and a body.

  It is the counterpart of the `command` adapter, and works the same way: `#[serde(with = "pimalaya_config::notify")]` on a field of that type reads `{ summary = "…", body = "…" }` and writes the same shape back, dropping an empty body. Both hand back a ready-to-use value of a foreign type and perform no I/O, so this crate builds the process and the notification while the caller runs and shows them. Expanding a template into a summary stays with the caller too, since which variables exist is its business.

  It sits behind the `notify` feature, which pulls notify-rust with its D-Bus backend, the one every Pimalaya binary sending notifications already uses. That backend links against the system D-Bus, so the `vendored` feature forwards to notify-rust for the platforms lacking one.

## [0.1.1] - 2026-07-25

### Added

- Added `toml::to_string`, a TOML serializer tuned for wizard output.

  It keeps the per-account tables (`[accounts.<name>]`) as the only table headers, flattens every other table into dotted keys (`imap.sasl.plain.username = …`), and drops empty tables.

## [0.1.0] - 2026-07-16

### Changed

- Migrated from toml 0.8 to toml 1.1. The `toml` feature now enables toml's own `serde` feature explicitly, which 1.x split out of its default set; parsing, `Value` handling and serialization are otherwise unchanged.
- Dropped the `serde-toml-merge` dependency, inlining an equivalent deep-merge (scalars override, arrays concatenate, tables merge recursively, incompatible types error): the crate does not follow toml past 0.9, so it could not move to toml 1.x.
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

[unreleased]: https://github.com/pimalaya/config/compare/v0.1.4..HEAD
[0.1.4]: https://github.com/pimalaya/config/compare/v0.1.3..v0.1.4
[0.1.3]: https://github.com/pimalaya/config/compare/v0.1.2..v0.1.3
[0.1.2]: https://github.com/pimalaya/config/compare/v0.1.1..v0.1.2
[0.1.1]: https://github.com/pimalaya/config/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/pimalaya/config/compare/v0.0.2..v0.1.0
[0.0.2]: https://github.com/pimalaya/config/compare/v0.0.1..v0.0.2
[0.0.1]: https://github.com/pimalaya/config/compare/root...v0.0.1
