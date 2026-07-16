//! TOML configuration loading, merging and shell expansion.
//!
//! [`TomlConfig`] is the loader trait: it reads a project's TOML config
//! from explicit paths or the platform default locations, deep-merges
//! several files into one, and maps a missing file to `Ok(None)` so
//! callers can launch a wizard. The [`shell_expanded_string`] and
//! [`shell_expanded_path`] deserializers expand environment variables
//! in string and path config fields.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use dirs::{config_dir, home_dir};
use log::debug;
use serde::{Deserialize, Deserializer, de::Visitor};
use toml::Value;

/// A deserializable TOML configuration exposing named accounts.
///
/// Consumers implement it on their own config type to inherit the
/// loading, deep-merging and default-path resolution provided by the
/// default methods; only the project name and the account accessors are
/// left to the implementor.
pub trait TomlConfig: for<'de> Deserialize<'de> {
    /// The per-account configuration this config holds.
    type Account;

    /// The project name, used to build the default config paths
    /// (`<project>/config.toml`, `.<project>rc`).
    fn project_name() -> &'static str;

    /// Takes the account marked as default, if any.
    fn take_default_account(&mut self) -> Option<(String, Self::Account)>;

    /// Takes the account registered under `name`, if any.
    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)>;

    /// Takes the default account when `name` is empty, `"default"` or
    /// absent, the named account otherwise.
    ///
    /// Returns an error when a name is given but no matching account
    /// exists.
    fn take_account(&mut self, name: Option<&str>) -> Result<Option<(String, Self::Account)>> {
        match name {
            Some("default") | Some("") | None => Ok(self.take_default_account()),
            Some(name) => match self.take_named_account(name) {
                None => bail!("Get account `{name}` error"),
                account => Ok(account),
            },
        }
    }

    /// Read and parse the TOML configuration at the given paths
    ///
    /// Returns an error if a configuration file cannot be read or if
    /// a content cannot be parsed.
    fn from_paths(paths: &[PathBuf]) -> Result<Self> {
        match paths.len() {
            0 => {
                bail!("Read TOML config from empty paths error");
            }
            1 => {
                let path = &paths[0];
                let content = fs::read_to_string(path).context("Read TOML config file error")?;
                toml::from_str(&content).context("Parse TOML config error")
            }
            _ => {
                let base = fs::read_to_string(&paths[0]).context("Read TOML config file error")?;
                let mut merged_content: Value =
                    toml::from_str(&base).context("Parse TOML config error")?;

                for path in &paths[1..] {
                    let content = match fs::read_to_string(path) {
                        Ok(content) => content,
                        Err(err) => {
                            debug!("skip invalid subconfig at {}: {err}", path.display());
                            continue;
                        }
                    };

                    let content: Value =
                        toml::from_str(&content).context("Parse TOML config error")?;
                    merged_content = merge_toml(merged_content, content, "$")?;
                }

                merged_content.try_into().context("Parse TOML config error")
            }
        }
    }

    /// Loads the configuration from `paths` if any are given, falling
    /// back to the platform default path otherwise.
    ///
    /// Returns:
    ///
    /// - `Err` on parse errors or non-`NotFound` I/O errors (notably
    ///   `PermissionDenied`: those should not be silently treated as
    ///   "no config" because a brand-new wizard run could overwrite a
    ///   pre-existing but unreadable file).
    /// - `Ok(None)` when no config file is found (paths empty + no
    ///   default exists, or an explicit path that does not exist on
    ///   disk). Callers can use this signal to launch a wizard or
    ///   bail with their own message.
    /// - `Ok(Some(config))` on success.
    fn from_paths_or_default(paths: &[PathBuf]) -> Result<Option<Self>> {
        match paths.first() {
            None => Self::from_default_paths(),
            Some(path) => match path_status(path)? {
                PathStatus::Missing => Ok(None),
                PathStatus::Present => Self::from_paths(paths).map(Some),
            },
        }
    }

    /// Loads the configuration from the first valid default path.
    /// Returns `Ok(None)` if no default path exists.
    fn from_default_paths() -> Result<Option<Self>> {
        match Self::first_valid_default_path() {
            Some(path) => Self::from_paths(&[path]).map(Some),
            None => Ok(None),
        }
    }

    /// Returns the path a brand-new configuration should be written
    /// to: the first explicit path if one was given, otherwise the
    /// platform default. Used by callers to drive wizard targeting
    /// after [`from_paths_or_default`] returns `Ok(None)`.
    ///
    /// [`from_paths_or_default`]: Self::from_paths_or_default
    fn target_path(paths: &[PathBuf]) -> Result<PathBuf> {
        match paths.first() {
            Some(path) => Ok(path.clone()),
            None => Self::default_path(),
        }
    }

    /// Get the default configuration path
    ///
    /// Returns an error if the XDG configuration directory cannot be
    /// found.
    fn default_path() -> Result<PathBuf> {
        let Some(dir) = config_dir() else {
            bail!("Get XDG config directory error");
        };

        Ok(dir.join(Self::project_name()).join("config.toml"))
    }

    /// Get the first default configuration path that points to a
    /// valid file
    ///
    /// Tries paths in this order:
    ///
    /// - `$XDG_CONFIG_DIR/<project>/config.toml`
    /// - `$HOME/.config/<project>/config.toml`
    /// - `$HOME/.<project>rc`
    fn first_valid_default_path() -> Option<PathBuf> {
        let project = Self::project_name();

        Self::default_path()
            .ok()
            .filter(|p| p.exists())
            .or_else(|| home_dir().map(|p| p.join(".config").join(project).join("config.toml")))
            .filter(|p| p.exists())
            .or_else(|| home_dir().map(|p| p.join(format!(".{project}rc"))))
            .filter(|p| p.exists())
    }
}

/// Deep-merges the `other` TOML value into `base`, at the given dotted
/// `path` for error reporting.
///
/// `other` wins on scalars, tables are merged recursively and arrays
/// are concatenated; incompatible types on the two sides are an error.
/// Replaces the serde-toml-merge dependency, which does not follow toml
/// past 0.9.
fn merge_toml(base: Value, other: Value, path: &str) -> Result<Value> {
    match (base, other) {
        (Value::Table(mut base), Value::Table(other)) => {
            for (key, other) in other {
                let merged = match base.remove(&key) {
                    Some(base) => merge_toml(base, other, &format!("{path}.{key}"))?,
                    None => other,
                };
                base.insert(key, merged);
            }
            Ok(Value::Table(base))
        }
        (Value::Array(mut base), Value::Array(other)) => {
            base.extend(other);
            Ok(Value::Array(base))
        }
        (base, other) if base.same_type(&other) => Ok(other),
        (base, other) => bail!(
            "Merge TOML subconfigs error: incompatible types at `{path}`, \
             base is {}, override is {}",
            base.type_str(),
            other.type_str(),
        ),
    }
}

/// Distinguishes "the file is not there" from "the file exists" so
/// `from_paths_or_default` can map only the former to `Ok(None)` and
/// surface every other I/O issue as `Err`.
enum PathStatus {
    Missing,
    Present,
}

fn path_status(path: &Path) -> Result<PathStatus> {
    match fs::metadata(path) {
        Ok(_) => Ok(PathStatus::Present),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(PathStatus::Missing),
        Err(err) => {
            Err(err).with_context(|| format!("Stat TOML config file `{}` error", path.display()))
        }
    }
}

struct ShellExpandedStringVisitor;

impl<'de> Visitor<'de> for ShellExpandedStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string containing environment variable(s)")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match shellexpand::full(v) {
            Ok(v) => Ok(v.into_owned()),
            Err(_) => Ok(v.to_owned()),
        }
    }
}

/// Deserializes a string field, expanding environment variables and
/// the leading tilde in it.
///
/// Use via `#[serde(deserialize_with = "...")]`. Expansion failures
/// (an undefined variable) leave the raw value untouched rather than
/// erroring.
pub fn shell_expanded_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    deserializer.deserialize_string(ShellExpandedStringVisitor)
}

/// Deserializes a path field, expanding environment variables and the
/// leading tilde in it, as [`shell_expanded_string`] does for strings.
pub fn shell_expanded_path<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<PathBuf, D::Error> {
    shell_expanded_string(deserializer).map(Into::into)
}

#[cfg(test)]
mod tests {
    use toml::Value;

    fn doc(s: &str) -> Value {
        toml::from_str(s).unwrap()
    }

    fn merge(base: &str, other: &str) -> Result<Value, String> {
        super::merge_toml(doc(base), doc(other), "$").map_err(|err| err.to_string())
    }

    #[test]
    fn scalars_override_and_missing_keys_are_added() {
        let got = merge("a = 1\nkeep = true\n", "a = 2\nadded = \"x\"\n").unwrap();
        assert_eq!(got, doc("a = 2\nkeep = true\nadded = \"x\"\n"));
    }

    #[test]
    fn arrays_are_concatenated() {
        let got = merge("xs = [1, 2]\n", "xs = [3]\n").unwrap();
        assert_eq!(got, doc("xs = [1, 2, 3]\n"));
    }

    #[test]
    fn tables_are_merged_recursively() {
        let got = merge("[t]\na = 1\nb = 2\n", "[t]\nb = 3\nc = 4\n").unwrap();
        assert_eq!(got, doc("[t]\na = 1\nb = 3\nc = 4\n"));
    }

    #[test]
    fn incompatible_types_are_an_error() {
        let err = merge("a = 1\n", "a = \"x\"\n").unwrap_err();
        assert!(err.contains("incompatible types at `$.a`"), "{err}");
    }
}
