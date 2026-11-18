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

use std::{
    collections::HashMap,
    fmt,
    io::{self, IsTerminal},
    process::Stdio,
    time::Instant,
};

use log::{debug, trace};
use secrecy::{ExposeSecret, SecretString};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Error as _, MapAccess, Visitor},
};
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
    /// carries its trimmed standard error, or its program and exit status
    /// when it wrote none, never its standard output, which holds the
    /// secret.
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
///
/// It reads from a table holding `raw` or `command`, `cmd` its alias.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    /// A literal secret value read straight from the config.
    Raw(#[serde(serialize_with = "de")] SecretString),
    /// A command whose standard output is the secret value.
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

                // NOTE: piped input belongs to the caller, never the command
                let stdin = if io::stdin().is_terminal() {
                    Stdio::inherit()
                } else {
                    Stdio::null()
                };

                let start = Instant::now();
                let output = cmd.stdin(stdin).output().map_err(SecretError::Spawn)?;

                if !output.status.success() {
                    // NOTE: a command failing after printing the secret
                    // would leak it, and a lookup missing its entry
                    // (`secret-tool`, `security`) writes no error at all.
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let err = match stderr.trim() {
                        "" => {
                            let program = cmd.get_program().to_string_lossy();
                            format!("{program} failed with {}", output.status)
                        }
                        err => err.to_owned(),
                    };

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

/// Reads a table holding `raw` or `command`, `cmd` its alias.
///
/// A refusal never echoes the value: a bare string, the natural mistake
/// for a literal, would be the secret itself.
impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(SecretVisitor)
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

/// The refusal of a secret written as a bare value.
const BARE: &str =
    "a secret is a table: `raw` for a literal value, `command` for a command printing it";

/// The refusal of a `raw` secret holding no string.
const RAW_NOT_STRING: &str = "a secret's `raw` is a string";

/// The keys a secret table reads, as an unknown one is told.
const KEYS: &[&str] = &["raw", "command"];

/// Visits a [`Secret`], refusing anything but a table with [`BARE`].
struct SecretVisitor;

impl<'de> Visitor<'de> for SecretVisitor {
    type Value = Secret;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a table holding `raw` or `command`")
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Secret, E> {
        Err(E::custom(BARE))
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Secret, E> {
        Err(E::custom(BARE))
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Secret, E> {
        Err(E::custom(BARE))
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Secret, E> {
        Err(E::custom(BARE))
    }

    fn visit_str<E: de::Error>(self, _: &str) -> Result<Secret, E> {
        Err(E::custom(BARE))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Secret, A::Error> {
        let Some(key) = map.next_key::<String>()? else {
            return Err(A::Error::custom(BARE));
        };

        let secret = match key.as_str() {
            "raw" => Secret::Raw(map.next_value::<RawSecret>()?.0.into()),
            "command" | "cmd" => Secret::Command(map.next_value()?),
            key => return Err(A::Error::unknown_variant(key, KEYS)),
        };

        if map.next_key::<String>()?.is_some() {
            return Err(A::Error::custom(
                "a secret holds one of `raw` or `command`, not both",
            ));
        }

        Ok(secret)
    }
}

/// The value of a `raw` secret, refused with [`RAW_NOT_STRING`] when it
/// is no string.
struct RawSecret(String);

impl<'de> Deserialize<'de> for RawSecret {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_string(RawSecretVisitor)
    }
}

/// Visits a [`RawSecret`].
struct RawSecretVisitor;

impl Visitor<'_> for RawSecretVisitor {
    type Value = RawSecret;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<RawSecret, E> {
        Err(E::custom(RAW_NOT_STRING))
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<RawSecret, E> {
        Err(E::custom(RAW_NOT_STRING))
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<RawSecret, E> {
        Err(E::custom(RAW_NOT_STRING))
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<RawSecret, E> {
        Err(E::custom(RAW_NOT_STRING))
    }

    fn visit_str<E: de::Error>(self, raw: &str) -> Result<RawSecret, E> {
        Ok(RawSecret(raw.to_owned()))
    }

    fn visit_string<E: de::Error>(self, raw: String) -> Result<RawSecret, E> {
        Ok(RawSecret(raw))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{env::temp_dir, fs, path::PathBuf, process};

    use secrecy::ExposeSecret;
    use serde::{Deserialize, Serialize};

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
    fn a_failed_helper_never_exposes_its_stdout() {
        for (line, expected) in [
            (
                "printf SECRET_MARKER; printf locked >&2; exit 7",
                "Secret command error: locked",
            ),
            (
                "printf SECRET_MARKER; exit 7",
                "Secret command error: /bin/sh failed with exit status: 7",
            ),
        ] {
            let err = shell(line).get().unwrap_err();
            assert_eq!(err.to_string(), expected);
            assert!(!format!("{err:?}").contains("SECRET_MARKER"));
        }
    }

    #[test]
    fn a_successful_helper_still_returns_the_first_line() {
        let secret = shell("printf 'dummy-password\\nsecond-line\\n'")
            .get()
            .unwrap();
        assert_eq!(secret.expose_secret(), "dummy-password");
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

    #[test]
    fn a_silent_failing_command_names_its_program_and_status() {
        let err = Secret::Command(CommandConfig::Argv {
            program: String::from("false"),
            args: Vec::new(),
        })
        .get()
        .unwrap_err();

        assert!(
            err.to_string()
                .starts_with("Secret command error: false failed with ")
        );
    }

    #[test]
    fn a_failing_command_reports_its_stderr() {
        let err = shell("echo locked >&2; exit 1").get().unwrap_err();
        assert_eq!(err.to_string(), "Secret command error: locked");
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

    #[derive(Debug, Deserialize, Serialize)]
    struct Account {
        password: Secret,
    }

    /// Either key reads the command, which is written back as `command`.
    #[test]
    fn cmd_is_the_alias_of_command() {
        for key in ["cmd", "command"] {
            let written = format!("password.{key} = \"pass show mail\"");
            let account: Account = toml::from_str(&written).unwrap();

            let Secret::Command(CommandConfig::Shell(line)) = &account.password else {
                panic!("`{key}` read no shell command");
            };
            assert_eq!(line, "pass show mail");

            let rewritten = toml::to_string(&account).unwrap();
            assert_eq!(rewritten.trim(), "[password]\ncommand = \"pass show mail\"");
        }
    }

    #[test]
    fn a_refused_secret_never_echoes_its_value() {
        let bare =
            "a secret is a table: `raw` for a literal value, `command` for a command printing it";

        for (written, expected) in [
            ("password = \"FAKE4242\"", bare),
            ("password = 4242", bare),
            ("password = true", bare),
            ("password = {}", bare),
            ("password.raw = 4242", "a secret's `raw` is a string"),
            (
                "password.lookup = \"FAKE4242\"",
                "unknown variant `lookup`, expected `raw` or `command`",
            ),
            (
                "password = { raw = \"FAKE4242\", command = \"x\" }",
                "a secret holds one of `raw` or `command`, not both",
            ),
        ] {
            let err = toml::from_str::<Account>(written).unwrap_err();
            assert_eq!(err.message(), expected, "{written}");
        }

        let account: Account = toml::from_str("password = { raw = \"FAKE4242\" }").unwrap();
        let Secret::Raw(raw) = account.password else {
            panic!("no literal read");
        };
        assert_eq!(raw.expose_secret(), "FAKE4242");
    }
}
