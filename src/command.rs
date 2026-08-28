//! A command as a configuration writes it, and the serde adapter for
//! [`std::process::Command`].
//!
//! [`CommandConfig`] is the configured shape, kept as written: it is
//! what a field holding a command should store, being comparable,
//! hashable and cheap to clone, none of which a built
//! [`std::process::Command`] is. It becomes one through
//! [`CommandConfig::to_command`], at the moment something runs it.
//!
//! The adapter (`#[serde(with = "pimalaya_config::command")]` on a field
//! of type [`std::process::Command`]) serves a caller that wants the
//! runnable form directly, having nothing to compare or clone.
//!
//! Two TOML shapes are accepted:
//!
//! - **String**: `cmd = "pass show foo"`. The whole string is
//!   handed to the platform shell: `/bin/sh -c "<string>"` on Unix,
//!   `cmd /C "<string>"` on Windows. Use this when you want pipes,
//!   glob expansion, env substitution, etc.
//! - **Sequence of strings**: `cmd = ["pass", "show", "foo"]`.
//!   First element is the program, the rest are its arguments. No
//!   shell is involved, so quoting/whitespace rules are the kernel
//!   exec rules.
//!
//! Empty inputs (empty string, blank string, empty array) are
//! deserialization errors.
//!
//! Serialization mirrors the two shapes. A [`CommandConfig`] writes back
//! the shape it was read as, having kept it. The [`std::process::Command`]
//! adapter has no such memory and recovers it: a command built through
//! [`shell`] is written as its bare command line, and any other as the
//! program + args sequence.

use std::{fmt, process::Command};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, SeqAccess, Visitor},
    ser::SerializeSeq,
};

/// A command as a configuration writes it: a shell line, or a program
/// and its arguments.
///
/// The two are distinct values, and comparing them never crosses the
/// variants: a `Shell` line and the `Argv` spelling that runs it through
/// the platform shell do the same thing and are still not equal here.
/// Treating one as the other means guessing what a configuration meant,
/// and a caller keyed on this, resolving a credential by command, is the
/// last place to guess.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CommandConfig {
    /// A shell command line, run through the platform shell (see
    /// [`shell`]).
    Shell(String),
    /// A program and its arguments, run with no shell in between.
    Argv {
        /// The program to execute.
        program: String,
        /// Its arguments, empty for a program taking none. The program
        /// is a field of its own so an argv cannot be empty, which the
        /// shape a sequence deserializes from would otherwise allow.
        args: Vec<String>,
    },
}

impl CommandConfig {
    /// Builds the runnable command, which is where the platform shell
    /// enters for a [`Shell`](Self::Shell) line.
    pub fn to_command(&self) -> Command {
        match self {
            Self::Shell(line) => shell(line),
            Self::Argv { program, args } => {
                let mut cmd = Command::new(program);
                cmd.args(args);
                cmd
            }
        }
    }
}

impl<'de> Deserialize<'de> for CommandConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(CommandVisitor)
    }
}

impl Serialize for CommandConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Shell(line) => serializer.serialize_str(line),
            Self::Argv { program, args } => {
                let mut seq = serializer.serialize_seq(Some(args.len() + 1))?;
                seq.serialize_element(program)?;

                for arg in args {
                    seq.serialize_element(arg)?;
                }

                seq.end()
            }
        }
    }
}

/// Builds a [`Command`] that runs `line` through the platform shell:
/// `/bin/sh -c <line>` on Unix, `cmd /C <line>` on Windows. Used by
/// [`CommandConfig::to_command`]; exposed so callers writing the shell
/// command line themselves can match the same semantics.
pub fn shell(line: &str) -> Command {
    let (program, flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("/bin/sh", "-c")
    };

    let mut cmd = Command::new(program);
    cmd.arg(flag).arg(line);
    cmd
}

/// Deserializes a [`Command`] from a shell line string or a
/// program-plus-arguments list. See the module docs for the accepted
/// TOML shapes.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Command, D::Error> {
    CommandConfig::deserialize(deserializer).map(|config| config.to_command())
}

/// Serializes a [`Command`] back to the shape it came from: a bare
/// command line for shell-form commands (see [`shell`]), a
/// program-plus-arguments sequence otherwise.
pub fn serialize<S: Serializer>(cmd: &Command, serializer: S) -> Result<S::Ok, S::Error> {
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<String> = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    // A command produced by `shell` round-trips as its bare command
    // line: the deserializer wraps it back through the platform shell on
    // load, so emit the string form rather than the verbose sequence.
    if let Some(line) = shell_line(&program, &args) {
        return serializer.serialize_str(line);
    }

    let mut seq = serializer.serialize_seq(Some(args.len() + 1))?;
    seq.serialize_element(&program)?;

    for arg in &args {
        seq.serialize_element(arg)?;
    }

    seq.end()
}

/// Returns the bare command line when `program`/`args` are exactly what
/// [`shell`] produces on the current platform (`/bin/sh -c <line>` on
/// Unix, `cmd /C <line>` on Windows).
fn shell_line<'a>(program: &str, args: &'a [String]) -> Option<&'a str> {
    let (shell_program, shell_flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("/bin/sh", "-c")
    };

    match args {
        [flag, line] if program == shell_program && flag.as_str() == shell_flag => Some(line),
        _ => None,
    }
}

struct CommandVisitor;

impl<'de> Visitor<'de> for CommandVisitor {
    type Value = CommandConfig;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a shell command line (string, wrapped through the platform shell) \
             or a non-empty list of strings (program + args)",
        )
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        let line = v.trim();

        if line.is_empty() {
            return Err(E::custom("command cannot be empty"));
        }

        Ok(CommandConfig::Shell(line.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let Some(program) = seq.next_element::<String>()? else {
            return Err(<A::Error as Error>::custom("command cannot be empty"));
        };

        let mut args = Vec::new();

        while let Some(arg) = seq.next_element::<String>()? {
            args.push(arg);
        }

        Ok(CommandConfig::Argv { program, args })
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde::de::value::{Error, SeqDeserializer, StringDeserializer};

    use super::CommandConfig;

    fn de_str(s: &str) -> Result<Command, Error> {
        let d = StringDeserializer::<Error>::new(s.to_owned());
        super::deserialize(d)
    }

    fn de_seq<'a, I: IntoIterator<Item = &'a str>>(items: I) -> Result<Command, Error> {
        let owned: Vec<String> = items.into_iter().map(String::from).collect();
        let d = SeqDeserializer::<_, Error>::new(owned.into_iter());
        super::deserialize(d)
    }

    fn parts(cmd: &Command) -> (String, Vec<String>) {
        (
            cmd.get_program().to_string_lossy().into_owned(),
            cmd.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect(),
        )
    }

    #[test]
    fn deserialize_string_wraps_in_platform_shell() {
        let cmd = de_str("pass show foo | tail -1").unwrap();
        let expected = super::shell("pass show foo | tail -1");
        assert_eq!(parts(&cmd), parts(&expected));
    }

    #[test]
    fn deserialize_empty_or_blank_string() {
        assert_eq!(
            de_str("").unwrap_err().to_string(),
            "command cannot be empty"
        );
        assert_eq!(
            de_str("   \n\t").unwrap_err().to_string(),
            "command cannot be empty"
        );
    }

    #[test]
    fn deserialize_seq() {
        let cmd = de_seq(["pass", "show", "foo"]).unwrap();
        let (prog, args) = parts(&cmd);
        assert_eq!(prog, "pass");
        assert_eq!(args, vec!["show", "foo"]);
    }

    #[test]
    fn deserialize_empty_seq() {
        let v: [&str; 0] = [];
        assert_eq!(
            de_seq(v).unwrap_err().to_string(),
            "command cannot be empty"
        );
    }

    #[derive(serde::Serialize)]
    struct Wrap {
        #[serde(serialize_with = "super::serialize")]
        cmd: Command,
    }

    #[test]
    fn serialize_shell_command_as_string() {
        let out = toml::to_string(&Wrap {
            cmd: super::shell("pass show foo"),
        })
        .unwrap();
        assert_eq!(out.trim(), r#"cmd = "pass show foo""#);
    }

    #[test]
    fn serialize_seq_command_as_sequence() {
        let out = toml::to_string(&Wrap {
            cmd: de_seq(["pass", "show", "foo"]).unwrap(),
        })
        .unwrap();
        assert_eq!(out.trim(), r#"cmd = ["pass", "show", "foo"]"#);
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct WrapConfig {
        cmd: CommandConfig,
    }

    #[test]
    fn a_configured_command_round_trips_as_the_shape_it_was_written_in() {
        for written in [
            r#"cmd = "pass show foo""#,
            r#"cmd = ["pass", "show", "foo"]"#,
        ] {
            let parsed: WrapConfig = toml::from_str(written).unwrap();
            assert_eq!(toml::to_string(&parsed).unwrap().trim(), written);
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_shell_line_and_its_argv_spelling_are_two_values() {
        let shell = CommandConfig::Shell(String::from("pass show foo"));
        let argv = CommandConfig::Argv {
            program: String::from("/bin/sh"),
            args: vec![String::from("-c"), String::from("pass show foo")],
        };

        assert_ne!(shell, argv);
        assert_eq!(parts(&shell.to_command()), parts(&argv.to_command()));
    }
}
