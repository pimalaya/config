use std::{fmt, fs, io, path::Path, path::PathBuf};

use anyhow::{bail, Context, Result};
use dirs::{config_dir, home_dir};
use log::debug;
use serde::{de::Visitor, Deserialize, Deserializer};
use serde_toml_merge::merge;
use toml::Value;

pub trait TomlConfig: for<'de> Deserialize<'de> {
    type Account;

    fn project_name() -> &'static str;

    fn take_default_account(&mut self) -> Option<(String, Self::Account)>;
    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)>;

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
                let ref content =
                    fs::read_to_string(path).context("Read TOML config file error")?;
                toml::from_str(content).context("Parse TOML config error")
            }
            _ => {
                let path = &paths[0];

                let mut merged_content = fs::read_to_string(path)
                    .context("Read TOML config file error")?
                    .parse::<Value>()
                    .context("Parse TOML config error")?;

                for path in &paths[1..] {
                    let content = fs::read_to_string(path);

                    let content = match content {
                        Ok(content) => content.parse().context("Parse TOML config error")?,
                        Err(err) => {
                            debug!("skip invalid subconfig at {}: {err}", path.display());
                            continue;
                        }
                    };

                    match merge(merged_content, content) {
                        Ok(content) => merged_content = content,
                        Err(err) => bail!("Merge TOML subconfigs error: {err}"),
                    }
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
    ///   `PermissionDenied` — those should not be silently treated as
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
        formatter.write_str("an string containing environment variable(s)")
    }

    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        match shellexpand::full(&v) {
            Ok(v) => Ok(v.to_string()),
            Err(_) => Ok(v),
        }
    }
}

pub fn shell_expanded_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    deserializer.deserialize_string(ShellExpandedStringVisitor)
}

pub fn shell_expanded_path<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<PathBuf, D::Error> {
    shell_expanded_string(deserializer).map(Into::into)
}
