---
cairn: delta
change: the-tls-table-is-spelled-once
---

## ADDED Requirements

### Requirement: Every binary spells its TLS table alike
A binary reaching a server over TLS SHALL read a `tls` table of its own type, repeated in each binary rather than shared from this crate (serde's `flatten` cannot mix with `deny_unknown_fields`), and SHALL spell it with these keys, each optional: `provider`, `rustls` or `native-tls`, the build's first when unset; `rustls.crypto`, `aws` or `ring`, the build's first when unset; `cert`, the path of a PEM certificate trusted beside the platform's; `client.cert` and `client.key`, the paths of a PEM client certificate and of its PEM private key (PKCS#8 under native-tls), both required once `client` is given. Paths SHALL be shell-expanded as `shell_expanded_path` expands them. An account SHALL take the table at `tls` and each of its network backends at `<backend>.tls`, the backend's table taking from the account's, field by field, each of `provider`, `rustls.crypto`, `cert` and `client` it leaves unset, `client` as one so a certificate is never paired with another table's key. A key the table does not read SHALL be refused by serde's `deny_unknown_fields` alone, at every level of the table, never by name. A binary MAY read more keys in its table, but SHALL NOT rename, move or retype these. pimalaya-config reads no TLS table: this requirement is where the keys are written once, and the binaries' configuration dumps are checked against it.

#### Scenario: A backend table over the account's
- **GIVEN** an account holding `tls.provider = "native-tls"` and `tls.cert = "~/ca.pem"`, and its `imap.tls.cert = "~/imap.pem"`
- **WHEN** the IMAP connection is configured
- **THEN** it runs native-tls trusting the home directory's `imap.pem`, and every other network backend of the account native-tls trusting `ca.pem`

#### Scenario: A misspelled key
- **GIVEN** an account holding `tls.rustls.cypto = "ring"`
- **WHEN** the configuration is loaded
- **THEN** loading fails with serde's "unknown field `cypto`, expected `crypto`"

## MODIFIED Requirements

## REMOVED Requirements
