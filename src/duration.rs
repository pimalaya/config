//! A duration as a configuration writes it, and the serde adapter for
//! [`std::time::Duration`].
//!
//! Two TOML shapes are accepted:
//!
//! - **Integer**: `timeout = 90`, a number of seconds.
//! - **String**: `purge-after = "90d"`, a number and its unit: `s`
//!   seconds, `m` minutes, `h` hours, `d` days of 86400 seconds, `w` weeks
//!   of 7 days. A bare `"0"` is zero. Months and years, having no fixed
//!   length, are refused.
//!
//! Use `#[serde(with = "pimalaya_config::duration")]` on a field of type
//! [`Duration`], and [`option`] on an `Option<Duration>` one. A duration
//! is written back as a string in the largest unit dividing it evenly.

pub mod option;

use std::{fmt, time::Duration};

use serde::{
    Deserializer, Serializer,
    de::{self, Visitor},
    ser,
};

/// The units a string may end with, and the seconds each stands for,
/// largest first.
const UNITS: [(&str, u64); 5] = [
    ("w", 7 * 86400),
    ("d", 86400),
    ("h", 3600),
    ("m", 60),
    ("s", 1),
];

/// Deserializes a [`Duration`] from an integer of seconds or a string
/// holding a number and its unit.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    deserializer.deserialize_any(DurationVisitor)
}

/// Serializes a [`Duration`] as a string in the largest unit dividing it
/// evenly (`"90d"`), failing on a fraction of a second.
pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
    if duration.subsec_nanos() > 0 {
        let err = format!("Invalid duration {duration:?}, not a whole number of seconds");
        return Err(ser::Error::custom(err));
    }

    let secs = duration.as_secs();

    if secs == 0 {
        return serializer.serialize_str("0");
    }

    let unit = UNITS
        .into_iter()
        .find(|(_, size)| secs.is_multiple_of(*size));
    let (unit, size) = unit.unwrap_or(("s", 1));

    serializer.serialize_str(&format!("{}{unit}", secs / size))
}

struct DurationVisitor;

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer of seconds, or a number and its unit such as \"90d\"")
    }

    fn visit_u64<E: de::Error>(self, secs: u64) -> Result<Self::Value, E> {
        Ok(Duration::from_secs(secs))
    }

    fn visit_i64<E: de::Error>(self, secs: i64) -> Result<Self::Value, E> {
        match u64::try_from(secs) {
            Ok(secs) => Ok(Duration::from_secs(secs)),
            Err(_) => Err(E::custom(format!(
                "Invalid duration {secs}, a duration cannot be negative"
            ))),
        }
    }

    fn visit_str<E: de::Error>(self, raw: &str) -> Result<Self::Value, E> {
        parse(raw).map_err(E::custom)
    }
}

/// Parses `<number><unit>`, or a bare `0`.
fn parse(raw: &str) -> Result<Duration, String> {
    let trimmed = raw.trim();
    let number = trimmed.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let unit = &trimmed[number.len()..];

    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "Invalid duration {raw:?}, write an integer of seconds or a number \
             and its unit such as \"90d\""
        ));
    }

    let overflow = || format!("Invalid duration {raw:?}, too long to hold");
    let count: u64 = number.parse().map_err(|_| overflow())?;

    let size = match UNITS.into_iter().find(|(name, _)| *name == unit) {
        Some((_, size)) => size,
        None if unit.is_empty() && count == 0 => 0,
        None if unit.is_empty() => {
            return Err(format!(
                "Invalid duration {raw:?}, its unit is missing: `s`, `m`, `h`, \
                 `d` or `w`"
            ));
        }
        None => {
            return Err(format!(
                "Invalid duration {raw:?}, its unit `{unit}` is none of `s`, \
                 `m`, `h`, `d` or `w`"
            ));
        }
    };

    let secs = count.checked_mul(size).ok_or_else(overflow)?;
    Ok(Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Configured {
        #[serde(with = "crate::duration")]
        after: Duration,
    }

    fn read(value: &str) -> Result<Duration, String> {
        let config = toml::from_str::<Configured>(&format!("after = {value}"));
        config
            .map(|config| config.after)
            .map_err(|err| err.message().to_owned())
    }

    fn write(after: Duration) -> Result<String, String> {
        let written = toml::to_string(&Configured { after });
        written
            .map(|text| text.trim().to_owned())
            .map_err(|err| err.to_string())
    }

    #[test]
    fn an_integer_is_seconds() {
        assert_eq!(read("0"), Ok(Duration::ZERO));
        assert_eq!(read("90"), Ok(Duration::from_secs(90)));
    }

    #[test]
    fn a_string_is_a_number_and_its_unit() {
        for (raw, secs) in [
            ("\"0\"", 0),
            ("\"0d\"", 0),
            ("\"60s\"", 60),
            ("\"5m\"", 300),
            ("\"2h\"", 7200),
            ("\"90d\"", 7776000),
            ("\"1w\"", 604800),
            ("\" 12h \"", 43200),
        ] {
            assert_eq!(read(raw), Ok(Duration::from_secs(secs)), "{raw}");
        }
    }

    #[test]
    fn a_bad_value_is_refused_naming_it() {
        for (raw, err) in [
            ("-5", "Invalid duration -5, a duration cannot be negative"),
            (
                "\"\"",
                "Invalid duration \"\", write an integer of seconds or a \
                 number and its unit such as \"90d\"",
            ),
            (
                "\"90 days\"",
                "Invalid duration \"90 days\", write an integer of seconds or a \
                 number and its unit such as \"90d\"",
            ),
            (
                "\"-5s\"",
                "Invalid duration \"-5s\", write an integer of seconds or a \
                 number and its unit such as \"90d\"",
            ),
            (
                "\"90\"",
                "Invalid duration \"90\", its unit is missing: `s`, `m`, `h`, \
                 `d` or `w`",
            ),
            (
                "\"6mo\"",
                "Invalid duration \"6mo\", its unit `mo` is none of `s`, `m`, \
                 `h`, `d` or `w`",
            ),
            (
                "\"99999999999999999999s\"",
                "Invalid duration \"99999999999999999999s\", too long to hold",
            ),
            (
                "\"3000000000000000w\"",
                "Invalid duration \"3000000000000000w\", too long to hold",
            ),
        ] {
            assert_eq!(read(raw), Err(String::from(err)), "{raw}");
        }

        let err = read("1.5").unwrap_err();
        assert!(err.contains("expected an integer of seconds"), "{err}");
    }

    #[test]
    fn a_duration_is_written_in_its_largest_whole_unit() {
        for (secs, written) in [
            (0, "after = \"0\""),
            (45, "after = \"45s\""),
            (1800, "after = \"30m\""),
            (43200, "after = \"12h\""),
            (7776000, "after = \"90d\""),
            (1209600, "after = \"2w\""),
            (90061, "after = \"90061s\""),
        ] {
            let text = write(Duration::from_secs(secs)).unwrap();
            assert_eq!(text, written, "{secs}");

            let read_back = toml::from_str::<Configured>(&text).unwrap();
            assert_eq!(read_back.after, Duration::from_secs(secs));
        }

        let err = write(Duration::from_millis(1500)).unwrap_err();
        assert!(err.contains("not a whole number of seconds"), "{err}");
    }
}
