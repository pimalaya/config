---
cairn: log
change: removed-key-sentence
landed: 2026-11-15
---

# One sentence for a removed key

Finding F41 of the CLI alignment round (task L7): six shapes of the removed-key refusal across the apps. Added, behind the `toml` feature:

- `toml::Removed`, `Key` or `Table` with `name`, `why`, `instead`, displaying "The `<name>` key is removed, <why>: <instead>, see MIGRATION.md" (`table` for a table), and `Removed::error::<E: serde::de::Error>`.
- `removed_key!` (crate root), declaring an uninhabited enum deriving `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `PartialEq`, whose `Deserialize` fails with the sentence and whose `Serialize` is the empty match. Its expansion names `::serde`.

Tests: `a_removed_key_is_refused_by_name_and_an_absent_one_reads_none` (a key and a table, exact sentence through `toml::de::Error::message`; absent, `None` and written as nothing) and the macro's doctest. Additive: cargo-semver-checks 0.47.0 against v0.5.0 as a patch release, no finding.

Capabilities moved: toml ("A removed key is refused in one sentence" added).
