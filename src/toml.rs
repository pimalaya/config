use std::{fmt, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use dirs::{config_dir, home_dir};
use log::debug;
use serde::{de::Visitor, Deserialize, Deserializer};
use serde_toml_merge::merge;
use toml::Value;

pub trait TomlConfig: for<'de> Deserialize<'de> {
    type Account;

    fn project_name() -> &'static str;

    fn find_default_account(&self) -> Option<(String, Self::Account)>;
    fn find_account(&self, name: &str) -> Option<(String, Self::Account)>;

    fn get_account(&self, account_name: Option<&str>) -> Result<(String, Self::Account)> {
        match account_name {
            Some("default") | Some("") | None => match self.find_default_account() {
                Some(account) => Ok(account),
                None => bail!("Get default account error"),
            },
            Some(name) => match self.find_account(name) {
                Some(account) => Ok(account),
                None => bail!("Get account `{name}` error"),
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

    fn from_paths_or_default(paths: &[PathBuf]) -> Result<Self> {
        match paths.len() {
            0 => Self::from_default_paths(),
            _ if paths[0].exists() => Self::from_paths(paths),
            _ => bail!("Invalid TOML config file paths"),
        }
    }

    fn from_default_paths() -> Result<Self> {
        match Self::first_valid_default_path() {
            Some(path) => Self::from_paths(&[path]),
            None => bail!("Invalid TOML config file paths"),
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
