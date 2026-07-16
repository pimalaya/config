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
//! lazily at the moment it is needed rather than cached. [`command`]
//! (same feature) is the serde adapter its `Command` variant relies on,
//! reading a [`std::process::Command`] from a shell line string or a
//! program-plus-arguments list and writing either shape back unchanged.
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
//! [`Secret`]: secret::Secret

#[cfg(feature = "secret")]
pub mod command;
#[cfg(feature = "secret")]
pub mod secret;
#[cfg(feature = "toml")]
pub mod toml;
