---
cairn: delta
change: default-is-a-name
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: An account is taken by name or as the default
`TomlConfig::take_account` SHALL take the account named, an absent or empty name meaning the one account marked default; `default` is a name like any other. An unknown name SHALL fail with `AccountError::Missing`, reading "Account `{name}` not found, the configuration holds: {names}", the names sorted and comma-separated, or "the configuration holds no account" when it holds none, followed by ", omit `-a` to take the account marked default" when the name is `default`. No account marked default SHALL fail with `AccountError::NoDefault`, reading "No default account found, name one with `-a <NAME>` or mark one with `default = true`". Several accounts marked default SHALL fail with `AccountError::SeveralDefaults` rather than pick one, reading "Several accounts are marked default ({names}), keep `default = true` on one, or name one with `-a <NAME>`", the names sorted.

#### Scenario: The name `default`
- **GIVEN** accounts `default` and `b`, `b` marked default
- **WHEN** account `default` is taken
- **THEN** it takes `default`

## REMOVED Requirements
