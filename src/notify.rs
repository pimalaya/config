//! Serde adapter for [`notify_rust::Notification`].
//!
//! Use via `#[serde(with = "pimalaya_config::notify")]` on a field (or
//! enum-variant payload) of type [`Notification`].
//!
//! One TOML shape is accepted, a table carrying the two fields every
//! notification daemon renders:
//!
//! ```toml
//! notify = { summary = "Comodoro", body = "Work started!" }
//! ```
//!
//! The summary is required and cannot be blank, since a notification
//! without one has nothing to show. The body is optional and defaults
//! to empty. Serialization mirrors the shape, and omits an empty body.
//!
//! ```rust,no_run
//! use pimalaya_config::notify::Notification;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Hook {
//!     #[serde(with = "pimalaya_config::notify")]
//!     notify: Notification,
//! }
//!
//! let hook: Hook = toml::from_str(r#"notify = { summary = "Hi", body = "There" }"#).unwrap();
//! hook.notify.show().unwrap();
//! ```
//!
//! What comes out is a ready-to-show notification, exactly as
//! [`command`](crate::command) yields a ready-to-run process: this
//! crate builds it and never shows it, leaving the daemon call, and the
//! decision of when to make it, to the caller.
//!
//! A notification holding variables is a [`NotificationTemplate`], read
//! from the same table as a plain struct field. It expands the shell's
//! syntax (`$name`, `${name}`, `${name:-default}`, `$$`), so a hook's
//! notification and its command spell a variable alike; which variables
//! exist stays the caller's business, answered by the lookup it renders
//! with. Pimalaya products name them in camelCase, as their `--json`
//! keys (guideline cli-008).
//!
//! ## What the `notify` feature decides
//!
//! The module is always compiled, so a consumer names [`Notification`]
//! and shows one whatever the build, with no `#[cfg]` of its own. What
//! the feature decides is whether anything is behind them: without it,
//! [`Notification`] holds nothing, reading one from a configuration
//! fails, and showing one fails the same way, all of them naming the
//! cargo feature to rebuild with.

use std::{error, fmt};

#[cfg(not(feature = "notify"))]
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

/// A desktop notification, as notify-rust holds it.
#[cfg(feature = "notify")]
pub type Notification = notify_rust::Notification;

/// The notification a build without the `notify` cargo feature cannot
/// have.
///
/// It carries nothing, since nothing can be done with it: reading one
/// from a configuration fails, and so does showing one.
#[cfg(not(feature = "notify"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Notification;

#[cfg(not(feature = "notify"))]
impl Notification {
    /// Reports that this build has no notification backend, where
    /// notify-rust's own `show` would reach the daemon.
    pub fn show(&self) -> Result<(), &'static str> {
        Err(MISSING_FEATURE)
    }
}

/// What every path says when the `notify` cargo feature is missing.
#[cfg(not(feature = "notify"))]
const MISSING_FEATURE: &str = "Missing `notify` cargo feature";

/// Deserializes a [`Notification`] from a summary and a body. See the
/// module docs for the accepted TOML shape.
#[cfg(feature = "notify")]
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Notification, D::Error> {
    let fields = NotificationFields::deserialize(deserializer)?;

    if fields.summary.trim().is_empty() {
        return Err(D::Error::custom("notification summary is empty"));
    }

    // NOTE: the type is non-exhaustive, so it is built through its
    // setters rather than from a literal.
    let mut notification = Notification::new();
    notification.summary(&fields.summary).body(&fields.body);

    Ok(notification)
}

/// Refuses the notification, this build having no backend to show it
/// with.
#[cfg(not(feature = "notify"))]
pub fn deserialize<'de, D: Deserializer<'de>>(_: D) -> Result<Notification, D::Error> {
    Err(D::Error::custom(MISSING_FEATURE))
}

/// Serializes a [`Notification`] back to the shape it came from,
/// dropping an empty body.
#[cfg(feature = "notify")]
pub fn serialize<S: Serializer>(
    notification: &Notification,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    NotificationFields {
        summary: notification.summary.clone(),
        body: notification.body.clone(),
    }
    .serialize(serializer)
}

/// Refuses to write a notification this build could never have read.
#[cfg(not(feature = "notify"))]
pub fn serialize<S: Serializer>(_: &Notification, _: S) -> Result<S::Ok, S::Error> {
    Err(S::Error::custom(MISSING_FEATURE))
}

/// The fields a configuration writes, which is the portable subset of
/// what a notification carries: everything else is either
/// platform-specific (the hints) or better left to the caller (the
/// appname, taken from the running binary).
#[cfg(feature = "notify")]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct NotificationFields {
    summary: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    body: String,
}

/// A notification whose summary and body hold variables, read from the
/// table a [`Notification`] reads and rendered when it is shown.
///
/// Variables take the shell's syntax: `$name`, `${name}`, and
/// `${name:-default}`, the default replacing an empty value; `$$` writes
/// a `$`, and a `$` before anything else stays as it is. Which names
/// exist is the caller's business: [`render`](Self::render) asks its
/// lookup for each, and [`check`](Self::check) tests them against the
/// names a caller declares, so a configuration naming an unknown one is
/// refused when it is read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationTemplate {
    summary: String,
    body: String,
}

impl NotificationTemplate {
    /// Builds a template, refusing a blank summary and a malformed
    /// variable.
    pub fn new(
        summary: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<Self, NotificationTemplateError> {
        let summary = summary.into();
        let body = body.into();

        if summary.trim().is_empty() {
            return Err(NotificationTemplateError::EmptySummary);
        }

        parse(&summary)?;
        parse(&body)?;

        Ok(Self { summary, body })
    }

    /// The summary, as written.
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// The body, as written.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Fails naming the first variable `known` refuses, without asking
    /// for any value.
    pub fn check(&self, known: impl Fn(&str) -> bool) -> Result<(), NotificationTemplateError> {
        for text in [&self.summary, &self.body] {
            for segment in parse(text)? {
                if let Segment::Var { name, .. } = segment
                    && !known(name)
                {
                    return Err(NotificationTemplateError::UnknownVariable(name.to_owned()));
                }
            }
        }

        Ok(())
    }

    /// Expands the variables through `lookup` into a ready notification.
    ///
    /// `lookup` answers a name's value, an empty one for a name it knows
    /// but cannot fill, and `None` for a name it does not know, which
    /// fails the render.
    #[cfg(feature = "notify")]
    pub fn render(
        &self,
        mut lookup: impl FnMut(&str) -> Option<String>,
    ) -> Result<Notification, NotificationTemplateError> {
        let summary = expand(&self.summary, &mut lookup)?;
        let body = expand(&self.body, &mut lookup)?;

        let mut notification = Notification::new();
        notification.summary(&summary).body(&body);

        Ok(notification)
    }

    /// Refuses to render, this build having no backend to show the
    /// notification with.
    #[cfg(not(feature = "notify"))]
    pub fn render(
        &self,
        _: impl FnMut(&str) -> Option<String>,
    ) -> Result<Notification, NotificationTemplateError> {
        Err(NotificationTemplateError::MissingFeature)
    }
}

impl<'de> Deserialize<'de> for NotificationTemplate {
    #[cfg(feature = "notify")]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = NotificationFields::deserialize(deserializer)?;
        Self::new(fields.summary, fields.body).map_err(D::Error::custom)
    }

    #[cfg(not(feature = "notify"))]
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(D::Error::custom(MISSING_FEATURE))
    }
}

impl Serialize for NotificationTemplate {
    #[cfg(feature = "notify")]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        NotificationFields {
            summary: self.summary.clone(),
            body: self.body.clone(),
        }
        .serialize(serializer)
    }

    #[cfg(not(feature = "notify"))]
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(S::Error::custom(MISSING_FEATURE))
    }
}

/// Why a template cannot be built, checked or rendered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationTemplateError {
    /// The summary is blank, leaving the notification nothing to show.
    EmptySummary,
    /// A variable the caller does not know.
    UnknownVariable(String),
    /// A `${...}` form other than `${name}` and `${name:-default}`.
    InvalidVariable(String),
    /// This build has no `notify` cargo feature.
    MissingFeature,
}

impl fmt::Display for NotificationTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySummary => write!(f, "Notification summary is empty"),
            Self::UnknownVariable(name) => write!(f, "Unknown notification variable `{name}`"),
            Self::InvalidVariable(form) => write!(
                f,
                "Invalid notification variable `{form}`, expected `$name`, `${{name}}` or `${{name:-default}}`"
            ),
            Self::MissingFeature => write!(f, "Missing `notify` cargo feature"),
        }
    }
}

impl error::Error for NotificationTemplateError {}

/// A piece of a template: text kept as it is, or a variable.
// NOTE: only rendering reads the text and the default, and a build
// without the `notify` feature renders nothing.
#[cfg_attr(not(feature = "notify"), allow(dead_code))]
enum Segment<'a> {
    Text(&'a str),
    Var {
        name: &'a str,
        default: Option<&'a str>,
    },
}

/// Splits a template into its text and its variables.
fn parse(text: &str) -> Result<Vec<Segment<'_>>, NotificationTemplateError> {
    let mut segments = Vec::new();
    let mut rest = text;

    while let Some(at) = rest.find('$') {
        if at > 0 {
            segments.push(Segment::Text(&rest[..at]));
        }

        let after = &rest[at + 1..];

        if let Some(tail) = after.strip_prefix('$') {
            segments.push(Segment::Text("$"));
            rest = tail;
            continue;
        }

        if let Some(braced) = after.strip_prefix('{') {
            let Some(end) = braced.find('}') else {
                return Err(NotificationTemplateError::InvalidVariable(
                    rest[at..].to_owned(),
                ));
            };

            let inner = &braced[..end];

            let (name, default) = match inner.split_once(":-") {
                Some((name, default)) => (name, Some(default)),
                None => (inner, None),
            };

            if name_len(name) != name.len() || name.is_empty() {
                return Err(NotificationTemplateError::InvalidVariable(format!(
                    "${{{inner}}}"
                )));
            }

            segments.push(Segment::Var { name, default });
            rest = &braced[end + 1..];
            continue;
        }

        let len = name_len(after);

        if len == 0 {
            segments.push(Segment::Text("$"));
        } else {
            segments.push(Segment::Var {
                name: &after[..len],
                default: None,
            });
        }

        rest = &after[len..];
    }

    if !rest.is_empty() {
        segments.push(Segment::Text(rest));
    }

    Ok(segments)
}

/// The length of the shell variable name `text` starts with, zero when
/// it starts with none.
fn name_len(text: &str) -> usize {
    let bytes = text.as_bytes();

    match bytes.first() {
        Some(b) if b.is_ascii_alphabetic() || *b == b'_' => bytes
            .iter()
            .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
            .count(),
        _ => 0,
    }
}

/// Expands the variables of `text` through `lookup`.
#[cfg(feature = "notify")]
fn expand(
    text: &str,
    lookup: &mut impl FnMut(&str) -> Option<String>,
) -> Result<String, NotificationTemplateError> {
    let mut out = String::with_capacity(text.len());

    for segment in parse(text)? {
        match segment {
            Segment::Text(text) => out.push_str(text),
            Segment::Var { name, default } => {
                let Some(value) = lookup(name) else {
                    return Err(NotificationTemplateError::UnknownVariable(name.to_owned()));
                };

                match default {
                    Some(default) if value.is_empty() => out.push_str(default),
                    _ => out.push_str(&value),
                }
            }
        }
    }

    Ok(out)
}

// NOTE: the shapes under test are TOML ones, so the tests need the
// crate the `toml` feature pulls to write them.
#[cfg(all(test, feature = "toml"))]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::notify::{Notification, NotificationTemplate, NotificationTemplateError};

    #[derive(Debug, Deserialize, Serialize)]
    struct TemplateHook {
        notify: NotificationTemplate,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct Hook {
        #[serde(with = "crate::notify")]
        notify: Notification,
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_summary_and_a_body_round_trip() {
        let hook: Hook =
            toml::from_str(r#"notify = { summary = "Comodoro", body = "Work started!" }"#)
                .expect("deserialize the notification");

        assert_eq!(hook.notify.summary, "Comodoro");
        assert_eq!(hook.notify.body, "Work started!");

        let toml = toml::to_string(&hook).expect("serialize the notification");
        let hook: Hook = toml::from_str(&toml).expect("deserialize it back");

        assert_eq!(hook.notify.summary, "Comodoro");
        assert_eq!(hook.notify.body, "Work started!");
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_body_is_optional_and_never_written_empty() {
        let hook: Hook = toml::from_str(r#"notify = { summary = "Comodoro" }"#)
            .expect("deserialize the notification");

        assert_eq!(hook.notify.summary, "Comodoro");
        assert!(hook.notify.body.is_empty());

        let toml = toml::to_string(&hook).expect("serialize the notification");

        assert!(!toml.contains("body"));
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_notification_with_nothing_to_show_is_refused() {
        // A blank summary renders as an empty notification bubble, which
        // is a configuration mistake rather than an intent.
        assert!(toml::from_str::<Hook>(r#"notify = { summary = "" }"#).is_err());
        assert!(toml::from_str::<Hook>(r#"notify = { summary = "  " }"#).is_err());
        assert!(toml::from_str::<Hook>(r#"notify = { body = "orphan" }"#).is_err());

        // A typo is caught rather than silently dropped.
        assert!(toml::from_str::<Hook>(r#"notify = { summary = "s", tittle = "t" }"#).is_err());
    }

    #[cfg(not(feature = "notify"))]
    #[test]
    fn a_build_without_the_feature_names_the_missing_one() {
        const MESSAGE: &str = "Missing `notify` cargo feature";

        let err = toml::from_str::<Hook>(r#"notify = { summary = "Comodoro" }"#)
            .expect_err("a notification cannot be read without a backend");

        assert!(err.to_string().contains(MESSAGE), "{err}");
        assert_eq!(Notification.show().unwrap_err(), MESSAGE);
    }

    #[cfg(not(feature = "notify"))]
    #[test]
    fn a_template_without_the_feature_names_the_missing_one() {
        let err = toml::from_str::<TemplateHook>(r#"notify = { summary = "$id" }"#)
            .expect_err("a template cannot be read without a backend");

        assert!(
            err.to_string().contains("Missing `notify` cargo feature"),
            "{err}"
        );

        let template = NotificationTemplate::new("$id", "").expect("build the template");

        assert_eq!(
            template.render(|_| Some(String::new())).unwrap_err(),
            NotificationTemplateError::MissingFeature
        );
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_template_renders_values_and_defaults() {
        let hook: TemplateHook = toml::from_str(
            r#"notify = { summary = "New mail from $senderName", body = "${subject:-no subject} (${id})" }"#,
        )
        .expect("deserialize the template");

        let lookup = |subject: &'static str| {
            move |name: &str| match name {
                "senderName" => Some(String::from("Alice")),
                "subject" => Some(String::from(subject)),
                "id" => Some(String::from("42")),
                _ => None,
            }
        };

        let notification = hook.notify.render(lookup("")).expect("render");
        assert_eq!(notification.summary, "New mail from Alice");
        assert_eq!(notification.body, "no subject (42)");

        let notification = hook.notify.render(lookup("Hi")).expect("render");
        assert_eq!(notification.body, "Hi (42)");
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_template_refuses_a_name_its_lookup_does_not_know() {
        let template = NotificationTemplate::new("$sendr", "").expect("build the template");

        assert_eq!(
            template.render(|_| None).unwrap_err(),
            NotificationTemplateError::UnknownVariable(String::from("sendr"))
        );
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_dollar_is_written_by_two_or_before_no_name() {
        let template =
            NotificationTemplate::new("Pay $$5, $5 or $", "$$id").expect("build the template");
        let notification = template.render(|_| None).expect("render");

        assert_eq!(notification.summary, "Pay $5, $5 or $");
        assert_eq!(notification.body, "$id");
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_malformed_variable_is_refused_when_read() {
        for (form, named) in [
            ("${subject:none}", "${subject:none}"),
            ("${subject-none}", "${subject-none}"),
            ("${}", "${}"),
            ("${1st}", "${1st}"),
            ("x ${id", "${id"),
        ] {
            let toml = format!("notify = {{ summary = \"s\", body = \"{form}\" }}");
            let err = toml::from_str::<TemplateHook>(&toml).expect_err(form);

            assert!(
                err.to_string().contains(&format!("`{named}`")),
                "{form}: {err}"
            );
        }
    }

    #[cfg(feature = "notify")]
    #[test]
    fn a_template_round_trips_and_refuses_a_blank_summary() {
        let hook: TemplateHook =
            toml::from_str(r#"notify = { summary = "${id:-none}" }"#).expect("deserialize");
        let toml = toml::to_string(&hook).expect("serialize");
        let back: TemplateHook = toml::from_str(&toml).expect("deserialize it back");

        assert_eq!(back.notify, hook.notify);
        assert!(!toml.contains("body"));
        assert!(toml::from_str::<TemplateHook>(r#"notify = { summary = " " }"#).is_err());
    }

    #[test]
    fn a_check_names_the_first_undeclared_variable() {
        let template = NotificationTemplate::new("$id in ${collection:-?}", "${mailbox}")
            .expect("build the template");

        assert_eq!(
            template.check(|name| ["id", "collection"].contains(&name)),
            Err(NotificationTemplateError::UnknownVariable(String::from(
                "mailbox"
            )))
        );
        assert_eq!(
            template.check(|name| ["id", "collection", "mailbox"].contains(&name)),
            Ok(())
        );
    }
}
