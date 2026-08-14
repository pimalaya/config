//! Serde adapter for [`notify_rust::Notification`].
//!
//! Use via `#[serde(with = "pimalaya_config::notify")]` on a
//! field (or enum-variant payload) of type
//! [`notify_rust::Notification`].
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
//! What comes out is a ready-to-show notification, exactly as
//! [`command`](crate::command) yields a ready-to-run process: this
//! crate builds it and never shows it, leaving the daemon call, and the
//! decision of when to make it, to the caller.
//!
//! ```rust,no_run
//! use notify_rust::Notification;
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
//! A template belongs to the caller too: a summary carrying variables
//! is expanded before the notification is shown, since which variables
//! exist is the caller's business, not this crate's.

use notify_rust::Notification as NativeNotification;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// Deserializes a [`NativeNotification`] from a summary and a body. See
/// the module docs for the accepted TOML shape.
pub fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<NativeNotification, D::Error> {
    let notification = Notification::deserialize(deserializer)?;

    if notification.summary.trim().is_empty() {
        return Err(Error::custom("notification summary is empty"));
    }

    // NOTE: the native type is non-exhaustive, so it is built through
    // its setters rather than from a literal.
    let mut native = NativeNotification::new();
    native
        .summary(&notification.summary)
        .body(&notification.body);

    Ok(native)
}

/// Serializes a [`NativeNotification`] back to the shape it came from,
/// dropping an empty body.
pub fn serialize<S: Serializer>(
    native: &NativeNotification,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    Notification {
        summary: native.summary.clone(),
        body: native.body.clone(),
    }
    .serialize(serializer)
}

/// A notification as a configuration writes it, which is the portable
/// subset of what [`NativeNotification`] carries: everything else in it
/// is either platform-specific (the hints) or better left to the caller
/// (the appname, taken from the running binary).
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Notification {
    summary: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    body: String,
}

#[cfg(test)]
mod tests {
    use notify_rust::Notification as NativeNotification;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct Hook {
        #[serde(with = "crate::notify")]
        notify: NativeNotification,
    }

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

    #[test]
    fn a_body_is_optional_and_never_written_empty() {
        let hook: Hook = toml::from_str(r#"notify = { summary = "Comodoro" }"#)
            .expect("deserialize the notification");

        assert_eq!(hook.notify.summary, "Comodoro");
        assert!(hook.notify.body.is_empty());

        let toml = toml::to_string(&hook).expect("serialize the notification");

        assert!(!toml.contains("body"));
    }

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
}
