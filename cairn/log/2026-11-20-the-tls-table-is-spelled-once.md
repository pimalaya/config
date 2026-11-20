---
cairn: log
change: the-tls-table-is-spelled-once
landed: 2026-11-20
---

# The TLS table is spelled once

Owner ruling A1 of the CLI contract audit of 2026-11-20 (plans ledger cli-contract-audit-2026-11-20, proposal.md, both rounds): every binary aligns its TLS table on himalaya's, and the table stays a type repeated per binary.

- cairn/spec/toml.md: *Every binary spells its TLS table alike* added. Keys `provider` (`rustls`, `native-tls`), `rustls.crypto` (`aws`, `ring`), `cert`, `client.cert`, `client.key`, paths shell-expanded; the account's `tls` taken field by field by each `<backend>.tls`, `client` as one; unknown keys refused by `deny_unknown_fields` alone, never by name (the 2026-11-14 ruling, restated as A5).
- No code: the crate reads no TLS table. himalaya's src/config/tls.rs is the reference implementation the other binaries copy. The kit's check row prints the resolved table as `tls: {provider, crypto, cert, clientCert}` (pimalaya-cli, the-contract-audit-rulings).

Capabilities moved: toml (Every binary spells its TLS table alike, added).
