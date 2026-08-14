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
//! decision of when to make it, to the caller. A template belongs to
//! the caller too, since which variables exist is its business.
//!
//! ## What the `notify` feature decides
//!
//! The module is always compiled, so a consumer names [`Notification`]
//! and shows one whatever the build, with no `#[cfg]` of its own. What
//! the feature decides is whether anything is behind them: without it,
//! [`Notification`] holds nothing, reading one from a configuration
//! fails, and showing one fails the same way, all of them naming the
//! cargo feature to rebuild with.

#[cfg(not(feature = "notify"))]
use serde::ser::Error as _;
#[cfg(feature = "notify")]
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer, de::Error as _};

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

// NOTE: the shapes under test are TOML ones, so the tests need the
// crate the `toml` feature pulls to write them.
#[cfg(all(test, feature = "toml"))]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::notify::Notification;

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
        const MESSAGE: &str =
            "Missing `notify` cargo feature, rebuild with it to send notifications";

        let err = toml::from_str::<Hook>(r#"notify = { summary = "Comodoro" }"#)
            .expect_err("a notification cannot be read without a backend");

        assert!(err.to_string().contains(MESSAGE), "{err}");
        assert_eq!(Notification.show().unwrap_err(), MESSAGE);
    }
}
