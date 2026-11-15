//! The serde adapter for an `Option<Duration>`, reading and writing the
//! shapes [`crate::duration`] does.
//!
//! Use `#[serde(default, with = "pimalaya_config::duration::option")]`:
//! serde only calls it for a present key, an absent one defaulting to
//! `None`.

use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deserializes an optional [`Duration`], as [`crate::duration`] does a
/// mandatory one.
pub fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error> {
    let duration = Option::<DurationField>::deserialize(deserializer)?;
    Ok(duration.map(|DurationField(duration)| duration))
}

/// Serializes an optional [`Duration`], as [`crate::duration`] does a
/// mandatory one.
pub fn serialize<S: Serializer>(
    duration: &Option<Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    duration.map(DurationField).serialize(serializer)
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
struct DurationField(#[serde(with = "crate::duration")] Duration);

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Configured {
        #[serde(
            default,
            with = "crate::duration::option",
            skip_serializing_if = "Option::is_none"
        )]
        after: Option<Duration>,
    }

    #[test]
    fn an_absent_key_is_none_and_a_present_one_reads_both_shapes() {
        let config: Configured = toml::from_str("").unwrap();
        assert_eq!(config.after, None);
        assert_eq!(toml::to_string(&config).unwrap(), "");

        for (written, secs) in [("after = 90", 90), ("after = \"90d\"", 7776000)] {
            let config: Configured = toml::from_str(written).unwrap();
            assert_eq!(config.after, Some(Duration::from_secs(secs)), "{written}");
        }

        let config = Configured {
            after: Some(Duration::from_secs(7776000)),
        };
        assert_eq!(toml::to_string(&config).unwrap().trim(), "after = \"90d\"");
    }

    #[test]
    fn a_bad_value_is_refused_naming_it() {
        let err = toml::from_str::<Configured>("after = \"90y\"").unwrap_err();
        let expected = "Invalid duration \"90y\", its unit `y` is none of \
                        `s`, `m`, `h`, `d` or `w`";

        assert_eq!(err.message(), expected);
    }
}
