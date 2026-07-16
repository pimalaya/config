//! Secret resolution from a config literal or a shell command.
//!
//! A [`Secret`] is either an inline value or a command whose standard
//! output holds the value; [`Secret::get`] resolves it lazily, spawning
//! the command only when the value is actually needed. The command side
//! deserializes through the sibling [`crate::command`] adapter.

use std::{
    io,
    process::{Command, Stdio},
};

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

/// The errors raised while resolving a command-backed [`Secret`].
#[derive(Debug, Error)]
pub enum SecretError {
    /// The secret command could not be spawned.
    #[error("Spawn secret command error")]
    Spawn(#[source] io::Error),
    /// Waiting for the secret command to exit failed.
    #[error("Wait secret command error")]
    Wait(#[source] io::Error),
    /// The secret command exited with a non-zero status; the payload
    /// carries its trimmed standard error or output.
    #[error("Secret command error: {0}")]
    Output(String),
}

/// A secret value sourced either from a literal in the TOML config
/// or from a shell command's stdout.
///
/// [`Secret::get`] resolves the value at the moment it's needed;
/// nothing is cached. The `Command` variant deserializes through
/// [`crate::command`]: see that module for the accepted TOML
/// shapes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    /// A literal secret value read straight from the config.
    Raw(#[serde(serialize_with = "de")] SecretString),
    /// A shell command whose standard output is the secret value.
    #[serde(alias = "cmd", with = "crate::command")]
    Command(Command),
}

impl Secret {
    /// Resolves the secret, spawning the command and reading its first
    /// output line for the [`Command`](Self::Command) variant.
    ///
    /// Returns a [`SecretError`] when the command fails to spawn or
    /// exits with a non-zero status.
    pub fn get(self) -> Result<SecretString, SecretError> {
        match self {
            Self::Raw(secret) => Ok(secret),
            Self::Command(mut cmd) => {
                let output = cmd
                    .stdin(Stdio::null())
                    .output()
                    .map_err(SecretError::Spawn)?;

                if !output.status.success() {
                    let bytes = if output.stdout.is_empty() {
                        output.stderr
                    } else {
                        output.stdout
                    };
                    let err = String::from_utf8_lossy(&bytes).trim().to_string();
                    return Err(SecretError::Output(err));
                }

                let secret = String::from_utf8_lossy(&output.stdout);
                let secret = secret.lines().next().unwrap_or(secret.as_ref());
                let secret = secret.trim_matches(['\r', '\n']).into();

                Ok(secret)
            }
        }
    }
}

impl Clone for Secret {
    fn clone(&self) -> Self {
        match self {
            Self::Raw(secret) => Self::Raw(secret.clone()),
            Self::Command(cmd) => {
                let mut new = Command::new(cmd.get_program());
                new.args(cmd.get_args());
                Self::Command(new)
            }
        }
    }
}

/// Serializes a [`SecretString`] by exposing its inner value.
///
/// Wired as the `serialize_with` of [`Secret::Raw`] so a config round
/// trips back to the literal it came from.
pub fn de<S: Serializer>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error> {
    secret.expose_secret().serialize(serializer)
}
