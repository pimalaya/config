use std::{
    io,
    process::{Command, Stdio},
};

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

/// A secret value sourced either from a literal in the TOML config
/// or from a shell command's stdout.
///
/// [`Secret::get`] resolves the value at the moment it's needed —
/// nothing is cached. The `Command` variant deserializes through
/// [`crate::command`]: see that module for the accepted TOML
/// shapes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    Raw(#[serde(serialize_with = "de")] SecretString),
    #[serde(alias = "cmd", with = "crate::command")]
    Command(Command),
}

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("Spawn secret command error")]
    Spawn(#[source] io::Error),
    #[error("Wait secret command error")]
    Wait(#[source] io::Error),
    #[error("Secret command error: {0}")]
    Output(String),
}

pub fn de<S: Serializer>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error> {
    secret.expose_secret().serialize(serializer)
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

impl Secret {
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
