use std::{io, process::Output};

use io_process::{
    command::Command,
    coroutines::spawn_then_wait_with_output::{
        SpawnThenWaitWithOutput, SpawnThenWaitWithOutputError, SpawnThenWaitWithOutputResult,
    },
    runtimes::std::handle as handle_process,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    Raw(#[serde(serialize_with = "de")] SecretString),
    #[serde(alias = "cmd")]
    Command(Command),
}

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("Spawn secret command error: command invalid or program found")]
    Spawn(#[source] io::Error),
    #[error("Spawn secret command error: coroutine failure")]
    Coroutine(#[source] SpawnThenWaitWithOutputError),
    #[error("Secret command error: {0}")]
    Output(String),
}

pub fn de<S: Serializer>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error> {
    secret.expose_secret().serialize(serializer)
}

impl Secret {
    pub fn get(self) -> Result<SecretString, SecretError> {
        match self {
            Self::Raw(secret) => Ok(secret.clone()),
            Self::Command(mut cmd) => {
                cmd.expand = true;

                let mut coroutine = SpawnThenWaitWithOutput::new(cmd.clone());
                let mut arg = None;

                let Output {
                    status,
                    stdout,
                    stderr,
                } = loop {
                    match coroutine.resume(arg.take()) {
                        SpawnThenWaitWithOutputResult::Io(io) => {
                            arg = Some(handle_process(io).map_err(SecretError::Spawn)?);
                        }
                        SpawnThenWaitWithOutputResult::Ok(output) => {
                            break output;
                        }
                        SpawnThenWaitWithOutputResult::Err(err) => {
                            return Err(SecretError::Coroutine(err))
                        }
                    }
                };

                if !status.success() {
                    let bytes = if stdout.is_empty() { stderr } else { stdout };
                    let err = String::from_utf8_lossy(&bytes).to_string();
                    return Err(SecretError::Output(err));
                };

                let secret = String::from_utf8_lossy(&stdout);
                let secret = secret.lines().next().unwrap_or(secret.as_ref());
                let secret = secret.trim_matches(['\r', '\n']).into();

                Ok(secret)
            }
        }
    }
}
