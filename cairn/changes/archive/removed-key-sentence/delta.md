---
cairn: delta
change: removed-key-sentence
---

## ADDED Requirements

### Requirement: A removed key is refused in one sentence
`toml::Removed` SHALL read "The `{name}` key is removed, {why}: {instead}, see MIGRATION.md" for a `Key`, and the same with `table` in place of `key` for a `Table`. `Removed::error` SHALL answer that sentence as the format's deserialization error. A type declared through `removed_key!` SHALL deserialize from no value: a field `Option<T>` of it SHALL read `None` when its key is absent, SHALL fail the load with the sentence when the key is present, whatever it holds, and SHALL be written as nothing.

#### Scenario: A removed table
- **GIVEN** `removed_key!` declaring `TableRemoved: Table { name: "card.list.table", why: "the output being plain text", instead: "delete it" }`, the type of a field `table`
- **WHEN** a configuration holding `[table]` with any key is read
- **THEN** reading fails with "The `card.list.table` table is removed, the output being plain text: delete it, see MIGRATION.md"

#### Scenario: No removed key
- **GIVEN** the same field
- **WHEN** a configuration without `table` is read and written back
- **THEN** the field reads `None` and nothing is written for it

## MODIFIED Requirements

## REMOVED Requirements
