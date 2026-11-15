---
cairn: change
id: removed-keys-unlisted
status: landed
created: 2026-11-15
---

# An unknown key lists no removed one

## Why

Live testing of 2026-11-15 (task L13 of the CLI alignment round): an unknown key fails the load with serde's "unknown field `x`, expected one of ...", and that list holds the removed keys too, since each is a field whose type refuses it (`removed_key!`, `toml::Removed`). carillon at 77ee2ee listed `folder`, `mailbox`, `collection`, `watch`, `hooks` and `dav` as valid account keys. A user picking one from the list gets refused on the next run.

A flat set of removed names, registered by the macro and stripped from every list, would be wrong: carillon refuses `mailbox`, `watch` and `hooks` at the account level and reads them under each backend; neverest refuses `collection` at the account level and reads it under each source. Which keys are removed depends on the table, so the stripping has to know the type that raised the error, which serde's message does not say.

## What

`TomlConfig::from_paths` (so `load`, `from_paths_or_default` and `from_default_paths`) rewrites an unknown-field error of the configuration type before returning it:

- It reads the unknown key and the expected list back from serde's message, the fixed text of `de::Error::unknown_field`, and gives up on any other message.
- It finds the tables of the document holding the unknown key, and keeps those where the configuration type, given a document holding only that key at that place, fails with the same message: the table that raised the error, or one of the same type.
- A listed key is removed when, put alone at each of those places, the type fails with the sentence of `toml::Removed`. It leaves the list, which is written again as serde writes it: one key, two, one of several, or "there are no fields" when every one is removed.
- The error keeps its location: it displays as the TOML error did, the message replaced.

Nothing to adopt: an app loading through `TomlConfig` gets it by moving to the release. No global state, no registration: the configuration type itself answers, through the refusal its removed keys already carry. Anything the probe cannot settle (no document, a table reached through a tagged enum, a list it cannot read back) leaves the error as serde wrote it.

## Not done

A removed key refused by an app's own sentence, not the family one, stays listed. An app parsing with `toml::from_str` directly, not through the loader, keeps serde's message.
