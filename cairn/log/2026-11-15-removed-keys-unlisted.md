---
cairn: log
change: removed-keys-unlisted
landed: 2026-11-15
---

# An unknown key lists no removed one

Task L13 of the CLI alignment round, from live testing: serde's "unknown field `x`, expected one of ..." listed the removed keys among the valid ones, each being a field whose type refuses it. carillon at 77ee2ee listed `folder`, `mailbox`, `collection`, `watch`, `hooks` and `dav` on its account table.

A flat set of removed names would have been wrong: carillon refuses `mailbox`, `watch` and `hooks` on the account and reads them under each backend, neverest refuses `collection` on the account and reads it under each source. `TomlConfig::from_paths` now asks the configuration type itself, on an unknown-field error only:

- serde's message is read back (the fixed text of `de::Error::unknown_field`, checked by writing it again), any other message left alone;
- the tables of the document holding the unknown key are kept when the type, given a document holding only that key there, fails with the same message;
- a listed key leaves the list when, alone at each of those tables, the type fails with the `toml::Removed` sentence; the list is written again as serde writes it (one key, two, one of several, "there are no fields");
- the error displays as the TOML one, location kept, the message replaced (a private error type, no source, so a chain prints it once).

Nothing to register, no global state; an app gets it by moving to the release. One file and several layers both, the merged document cloned once before it is read. A table reached through a tagged enum, or a list that does not read back, leaves serde's message as it was.

Tests in src/toml.rs: `an_unknown_key_lists_no_removed_one` (the account, a valid `host` under `imap` beside the unknown one, exact rendering with the location), `a_key_removed_elsewhere_stays_listed` (`mailbox` refused on the account, listed under `imap`, `hook` not), `an_unknown_key_in_a_layer_lists_no_removed_one` (two layers), the three failing on 0.5.1; `the_keys_left_are_listed_as_serde_lists_them` (one, several, none). Checked offline on scratch copies of carillon (c68fea4) and neverest (ac5c0e3) patched onto this crate, with temp configurations: carillon's account, imap and hooks tables, neverest's account, source and collection tables, each listing only the keys it reads.

No public API change.

Capabilities moved: toml ("An unknown key lists no removed one" added).
