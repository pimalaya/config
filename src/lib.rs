#![cfg_attr(docsrs, feature(doc_cfg))]

//! # pimalaya-config
//!
//! Config utils shared by the Pimalaya command-line tools. Published
//! for internal Pimalaya usage: the API follows the needs of its
//! consumers (himalaya, neverest, cardamum, calendula and friends) and
//! may change without notice.
//!
//! ## Layout
//!
//! The crate is deliberately std: it reads files, spawns commands and
//! expands environment variables, so it exposes no I/O-free coroutines
//! and the no_std conventions of the io-* family do not apply here.
//! Each piece sits behind its own cargo feature, so a binary pulls in
//! only what it uses.
//!
//! [`toml`] (`toml` feature) holds the [`TomlConfig`] loader trait:
//! it reads and deep-merges a project's TOML configuration from
//! explicit paths, falls back to the platform default locations, and
//! tells a missing file (mapped to `Ok(None)` to drive a wizard) apart
//! from an unreadable one. The module also carries the
//! [`shell_expanded_string`] and [`shell_expanded_path`] deserializers
//! expanding environment variables in string and path config fields.
//!
//! [`secret`] (`secret` feature) holds the [`Secret`] enum resolving a
//! secret from a literal value or a shell command's standard output,
//! lazily at the moment it is needed rather than cached, and the
//! [`SecretResolver`] memoizing those spawns so a command named by
//! several fields is run once. [`command`] (same feature) holds what a
//! configured command is: a [`CommandConfig`], read from a shell line
//! string or a program-plus-arguments list and written back as the
//! shape it came from, which becomes a [`std::process::Command`] only
//! when something runs it. The module doubles as the serde adapter for
//! a field holding that runnable form directly.
//!
//! [`notify`] is its counterpart for desktop notifications, reading a
//! [`Notification`] from a summary and a body. It is always compiled,
//! so a consumer names that type and shows one with no `#[cfg]` of its
//! own: what the `notify` feature decides is whether anything is behind
//! it. With it, the type is notify-rust's own; without it, it holds
//! nothing, and reading or showing one fails naming the cargo feature
//! to rebuild with.
//!
//! ## Conventions
//!
//! The conventions every Pimalaya repository shares are described in
//! the org
//! [ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md)
//! and
//! [GUIDELINES](https://github.com/pimalaya/.github/blob/master/GUIDELINES.md).
//! As a shared std toolkit crate, pimalaya-config is exempt from the
//! strict item-name prefix: the crate name and module path already
//! namespace its public items.
//!
//! [`TomlConfig`]: toml::TomlConfig
//! [`shell_expanded_string`]: toml::shell_expanded_string
//! [`shell_expanded_path`]: toml::shell_expanded_path
//! [`CommandConfig`]: command::CommandConfig
//! [`Secret`]: secret::Secret
//! [`SecretResolver`]: secret::SecretResolver
//! [`Notification`]: notify::Notification

#[cfg(feature = "secret")]
pub mod command;
pub mod notify;
#[cfg(feature = "secret")]
pub mod secret;
#[cfg(feature = "toml")]
pub mod toml;
