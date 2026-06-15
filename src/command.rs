//! Serde adapter for [`std::process::Command`].
//!
//! Use via `#[serde(with = "pimalaya_config::command")]` on a field
//! (or enum-variant payload) of type [`std::process::Command`].
//!
//! Two TOML shapes are accepted:
//!
//! - **String** — `cmd = "pass show foo"`. The whole string is
//!   handed to the platform shell: `/bin/sh -c "<string>"` on Unix,
//!   `cmd /C "<string>"` on Windows. Use this when you want pipes,
//!   glob expansion, env substitution, etc.
//! - **Sequence of strings** — `cmd = ["pass", "show", "foo"]`.
//!   First element is the program, the rest are its arguments. No
//!   shell is involved, so quoting/whitespace rules are the kernel
//!   exec rules.
//!
//! Empty inputs (empty string, blank string, empty array) are
//! deserialization errors.
//!
//! Serialization mirrors the two shapes: a command built through the
//! platform shell (see [`shell`]) is written back as the bare command
//! line string it came from, and any other command is written as the
//! program + args sequence. A string-form command therefore round-trips
//! as the same string rather than the verbose `["/bin/sh", "-c", ...]`.

use std::{fmt, process::Command};

use serde::{
    Deserializer, Serializer,
    de::{Error, SeqAccess, Visitor},
    ser::SerializeSeq,
};

/// Builds a [`Command`] that runs `line` through the platform shell:
/// `/bin/sh -c <line>` on Unix, `cmd /C <line>` on Windows. Used by
/// the string-form deserializer; exposed so callers writing the shell
/// command line themselves (e.g. interactive wizards) can match the
/// same semantics.
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

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Command, D::Error> {
    deserializer.deserialize_any(CommandVisitor)
}

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
    type Value = Command;

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

        Ok(shell(line))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let Some(program) = seq.next_element::<String>()? else {
            return Err(<A::Error as Error>::custom("command cannot be empty"));
        };

        let mut cmd = Command::new(program);

        while let Some(arg) = seq.next_element::<String>()? {
            cmd.arg(arg);
        }

        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde::de::value::{Error, SeqDeserializer, StringDeserializer};

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
}
