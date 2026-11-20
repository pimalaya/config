---
cairn: change
id: the-tls-table-is-spelled-once
status: landed
created: 2026-11-20
---

# The TLS table is spelled once

## Why

The CLI contract audit of 2026-11-20 found the family's binaries spelling their TLS configuration apart. The owner ruled (A1, two rounds, 2026-11-20): TLS should be the same everywhere, on himalaya's table (`tls.provider`, `tls.rustls.crypto`, `tls.cert`, `tls.client.cert`, `tls.client.key`, an account-level `tls` inherited field by field by each `<backend>.tls`), but repeated per binary rather than shared from this crate, since serde's `flatten` cannot mix with `deny_unknown_fields` and an app extending one field would otherwise rewrite and map the whole type. Written once as a spec line, the copies cannot drift unnoticed: the configuration dumps are compared against it.

## What

Documentation only, no code: one requirement in cairn/spec/toml.md naming the keys, their values, the inheritance and the serde-only refusal of an unknown key.
