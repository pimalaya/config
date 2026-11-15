---
cairn: change
id: duration-adapter
status: landed
created: 2026-11-15
---

# One duration for configuration

## Why

The CLI alignment round of 2026-11-15 (finding F40) found durations written two ways: strings with a unit in neverest (`store.purge-after = "90d"`, `store.index-timeout = "60s"`, its own `HumanDuration`), integer seconds in carillon (`watch.poll.interval = 120`, `watch.idle.timeout`, `watch.push.ping`) and himalaya (the mbox lock `timeout`). A user moving between the apps meets both, and each app parses its own. Task N10 of the round has neverest accept integer seconds too; that wants one parser, here.

## What

`pimalaya_config::duration`, behind the `toml` feature, the serde adapter for a `std::time::Duration` (`#[serde(with = "pimalaya_config::duration")]`), and `duration::option` for an `Option<Duration>`:

- An integer is that many seconds, what carillon and himalaya read today.
- A string is a number and its unit, `s`, `m`, `h`, `d` (86400 seconds) or `w` (7 days), trimmed, and a bare `"0"` is zero: what neverest's `HumanDuration` reads today, so no file written for either app changes meaning.
- Anything else is refused naming the value: a negative integer, a float, a string with no number, a sign or inner spaces, no unit, another unit (months and years have no fixed length, as neverest refuses them), an overflow.
- Written back as a string in the largest unit dividing it evenly (`"90d"`, `"0"` for zero), as neverest writes `HumanDuration`; a fraction of a second fails to be written rather than being rounded.

No dependency added. No schemars support: the crate offers none elsewhere.
