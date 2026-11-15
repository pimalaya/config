---
cairn: log
change: duration-adapter
landed: 2026-11-15
---

# One duration for configuration

Finding F40 of the CLI alignment round (task L8): neverest reads durations as strings with a unit (`HumanDuration`, `"90d"`), carillon and himalaya as integer seconds. Added `pimalaya_config::duration` behind the `toml` feature, the serde adapter for `std::time::Duration`, and `duration::option` for `Option<Duration>`:

- An integer is seconds; a string is ASCII digits and one of `s`, `m`, `h`, `d`, `w`, trimmed; `"0"` alone is zero. Everything neverest's `HumanDuration` and carillon's `u64` fields read today reads the same.
- Refused naming the value: negative, another type, no number, a sign, inner spaces, no unit, another unit (`mo`, `y`), overflow.
- Written as a string in the largest whole unit, `"0"` for zero, as `HumanDuration` writes; a sub-second duration fails to be written.

No dependency added; no schemars support, the crate having none. Tests in src/duration.rs and src/duration/option.rs cover each shape, each refusal with its exact message, the written form read back, and the optional field absent and present.

Capabilities moved: duration (cairn/spec/duration.md added).
