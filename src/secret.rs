//! Secret resolution from a config literal or a shell command.
//!
//! A [`Secret`] is either an inline value or a command whose standard
//! output holds the value; [`Secret::get`] resolves it lazily, spawning
//! the command only when the value is actually needed. The command side
//! is a [`CommandConfig`], the shape the configuration wrote, which is
//! what makes a secret comparable and cheap to clone.
//!
//! [`SecretResolver`] is that same resolution, memoized. A consumer
//! assembling a runtime account resolves every secret through one of
//! them, so a command named by several fields is spawned once for the
//! whole account.

use std::{collections::HashMap, io, process::Stdio, time::Instant};

use log::{debug, trace};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

use crate::command::CommandConfig;

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
/// [`Secret::get`] resolves the value at the moment it's needed and
/// caches nothing: a caller resolving the same command twice spawns it
/// twice, which is what [`SecretResolver`] exists to avoid. See
/// [`crate::command`] for the TOML shapes the `Command` variant accepts.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    /// A literal secret value read straight from the config.
    Raw(#[serde(serialize_with = "de")] SecretString),
    /// A command whose standard output is the secret value.
    #[serde(alias = "cmd")]
    Command(CommandConfig),
}

impl Secret {
    /// Resolves the secret, spawning the command and reading its first
    /// output line for the [`Command`](Self::Command) variant.
    ///
    /// Returns a [`SecretError`] when the command fails to spawn or
    /// exits with a non-zero status.
    ///
    /// The command can take as long as the credential store behind it
    /// does, a locked `gpg-agent` being seconds rather than
    /// milliseconds, so the spawn is logged with the time it took. The
    /// value is never logged, and neither are the arguments, a command
    /// line being free to carry the secret itself.
    pub fn get(self) -> Result<SecretString, SecretError> {
        match self {
            Self::Raw(secret) => Ok(secret),
            Self::Command(config) => {
                let mut cmd = config.to_command();

                debug!("resolving secret command");
                trace!("program: {}", cmd.get_program().to_string_lossy());

                let start = Instant::now();
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

                debug!("secret command resolved in {:?}", start.elapsed());

                Ok(secret)
            }
        }
    }
}

/// Resolves secrets while memoizing the commands it spawns.
///
/// A configuration usually points several fields at one credential: an
/// account's IMAP and SMTP tables name the same password entry, and so
/// do its CardDAV and CalDAV ones. Resolved field by field, that entry
/// is read as many times as it appears, and for a `pass` or `gpg` entry
/// each read is a key unlock. A resolver spawns each distinct command
/// once and hands its value to every field naming it.
///
/// Distinct is [`CommandConfig`]'s own equality, which never crosses the
/// two shapes: what a configuration wrote is what is compared.
///
/// The resolver holds the plaintext for as long as it lives, so it
/// belongs where a runtime account is assembled and should be dropped
/// with it rather than kept for the life of the process.
#[derive(Debug, Default)]
pub struct SecretResolver {
    resolved: HashMap<CommandConfig, SecretString>,
}

impl SecretResolver {
    /// Builds a resolver with an empty memo.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves `secret`, reusing the value of an identical command
    /// this resolver already spawned.
    ///
    /// A literal is returned as is: there is nothing to spawn and
    /// nothing to memoize.
    pub fn resolve(&mut self, secret: Secret) -> Result<SecretString, SecretError> {
        let Secret::Command(config) = &secret else {
            return secret.get();
        };

        if let Some(resolved) = self.resolved.get(config) {
            debug!("reusing an already resolved secret command");
            return Ok(resolved.clone());
        }

        let config = config.clone();
        let resolved = secret.get()?;
        self.resolved.insert(config, resolved.clone());

        Ok(resolved)
    }
}

/// Serializes a [`SecretString`] by exposing its inner value.
///
/// Wired as the `serialize_with` of [`Secret::Raw`] so a config round
/// trips back to the literal it came from.
pub fn de<S: Serializer>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error> {
    secret.expose_secret().serialize(serializer)
}

#[cfg(all(test, unix))]
mod tests {
    use std::{env::temp_dir, fs, path::PathBuf, process};

    use secrecy::ExposeSecret;

    use super::{Secret, SecretResolver};
    use crate::command::CommandConfig;

    /// A path in the platform temporary directory, unique to the
    /// running test binary so a concurrent run cannot share it.
    fn temp_path(label: &str) -> PathBuf {
        temp_dir().join(format!("pimalaya-config-{label}-{}", process::id()))
    }

    fn shell(line: &str) -> Secret {
        Secret::Command(CommandConfig::Shell(line.to_owned()))
    }

    #[test]
    fn a_command_backed_secret_is_spawned_once_per_resolver() {
        let path = temp_path("resolve-once");
        let _ = fs::remove_file(&path);

        // Counts its own runs: one more byte, and the running total, per
        // spawn.
        let line = format!(
            "printf x >> {path}; wc -c < {path} | tr -d ' '",
            path = path.display()
        );

        let mut resolver = SecretResolver::new();
        let first = resolver.resolve(shell(&line)).unwrap();
        let second = resolver.resolve(shell(&line)).unwrap();

        assert_eq!(first.expose_secret(), "1");
        assert_eq!(second.expose_secret(), "1");

        let unmemoized = shell(&line).get().unwrap();
        assert_eq!(unmemoized.expose_secret(), "2");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn distinct_commands_resolve_to_their_own_value() {
        let mut resolver = SecretResolver::new();

        let first = resolver.resolve(shell("printf first")).unwrap();
        let second = resolver.resolve(shell("printf second")).unwrap();

        assert_eq!(first.expose_secret(), "first");
        assert_eq!(second.expose_secret(), "second");
    }

    /// The two shapes are two commands, whatever they end up running:
    /// the memo compares what the configuration wrote, never what it
    /// would resolve to.
    #[test]
    fn a_shell_line_and_its_argv_spelling_are_resolved_on_their_own() {
        let path = temp_path("shape-is-the-key");
        let _ = fs::remove_file(&path);

        let line = format!(
            "printf x >> {path}; wc -c < {path} | tr -d ' '",
            path = path.display()
        );
        let argv = Secret::Command(CommandConfig::Argv {
            program: String::from("/bin/sh"),
            args: vec![String::from("-c"), line.clone()],
        });

        let mut resolver = SecretResolver::new();

        assert_eq!(resolver.resolve(shell(&line)).unwrap().expose_secret(), "1");
        assert_eq!(resolver.resolve(argv).unwrap().expose_secret(), "2");

        fs::remove_file(&path).unwrap();
    }
}
