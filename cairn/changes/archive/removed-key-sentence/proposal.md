---
cairn: change
id: removed-key-sentence
status: landed
created: 2026-11-15
---

# One sentence for a removed key

## Why

The CLI alignment round of 2026-11-15 (finding F41) found the apps refusing a removed configuration key in six shapes: himalaya's "The `x` block is removed since v3: delete it, see MIGRATION.md", cardamum's "The `x` table is removed, <why>: delete it, see MIGRATION.md", calendula's "The `x` table is gone, ...; remove it from the calendula configuration" with no pointer to MIGRATION.md, neverest's free sentences through its `removed_key!` macro, carillon's "The account-level `x` moved under each backend as ..." and "The hook key `x` is now `y`", ortie's "The `hooks` table was removed in 3.0.0: ...". Each app declares an uninhabited enum, or a `deserialize_with` function, per removed key, with its own Deserialize impl. The rules of the round ask for one sentence family-wide, naming MIGRATION.md.

## What

- `toml::Removed`, a `Key` or a `Table` with its `name`, `why` and `instead`, whose `Display` is the family sentence: "The `<name>` key is removed, <why>: <instead>, see MIGRATION.md", `table` for a table. `Removed::error` answers it as any `serde::de::Error`, for an app keeping its own refuser: one line.
- `removed_key!`, exported at the crate root, declaring the uninhabited enum an app holds as `Option<T>`: absent, the field reads `None`; present, whatever its value, the load fails with the sentence. The type derives the usual traits and serializes (it has no value), so a configuration struct deriving `Serialize` needs no `skip_serializing`. A removed key is one invocation and one field.

Both sit behind the `toml` feature, beside the field deserializers. The macro's expansion names `::serde`, which every app depends on.

## Adoption

Each app keeps its own `why` and `instead`. The six shapes map without loss: cardamum's already is the sentence; himalaya's "since v3" goes in `why`; calendula's "moved to `unfog`" becomes a `why` and gains MIGRATION.md; neverest's longer replacements go in `instead`, one type per name (`left` and `right` become two); carillon's renamed hooks read "The `on-message-added` key is removed, renamed for 1.0: write `on-message-add` instead, see MIGRATION.md"; ortie's `hooks` table maps directly.
