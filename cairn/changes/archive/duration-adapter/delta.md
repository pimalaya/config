---
cairn: delta
change: duration-adapter
---

## ADDED Requirements

### Requirement: A duration is an integer of seconds or a number and its unit
`duration::deserialize` SHALL read a non-negative integer as that many seconds, and a string, trimmed of surrounding whitespace, as ASCII digits followed by a unit: `s` (1 second), `m` (60), `h` (3600), `d` (86400) or `w` (604800). The string `"0"` without a unit SHALL read zero. Any other value SHALL be refused with an error naming it: a negative integer, a value of another type, a string with no number, a sign or inner spaces, no unit, another unit, months and years among them, or a count overflowing 64-bit seconds.

#### Scenario: Both shapes
- **GIVEN** `after = 90`, then `after = "90d"`
- **WHEN** each is read
- **THEN** they read 90 seconds, then 7776000 seconds

#### Scenario: A unit with no fixed length
- **GIVEN** `after = "6mo"`
- **WHEN** it is read
- **THEN** reading fails with "Invalid duration "6mo", its unit `mo` is none of `s`, `m`, `h`, `d` or `w`"

#### Scenario: A string with no unit
- **GIVEN** `after = "90"`
- **WHEN** it is read
- **THEN** reading fails with "Invalid duration "90", its unit is missing: `s`, `m`, `h`, `d` or `w`"

### Requirement: A duration is written in its largest whole unit
`duration::serialize` SHALL write a string, `"0"` for zero, else the count in the largest of `w`, `d`, `h`, `m` and `s` dividing the duration evenly, which reads back as the same duration. A duration holding a fraction of a second SHALL fail to be written.

#### Scenario: Written back
- **GIVEN** 7776000 seconds, then 90061 seconds
- **WHEN** each is written
- **THEN** they read `"90d"`, then `"90061s"`

### Requirement: An optional duration
`duration::option` SHALL read and write an `Option<Duration>` as `duration` does a `Duration`, a present key reading `Some`. Paired with `#[serde(default)]`, an absent key SHALL read `None`.

## MODIFIED Requirements

## REMOVED Requirements
