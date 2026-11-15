---
cairn: log
change: default-is-a-name
landed: 2026-11-15
---

# `default` is a name like any other

`TomlConfig::take_account` no longer reads the name `default` as the account marked default: an absent or empty name takes that account, any other name is looked up as written. `AccountError::Missing` for the name `default` adds "omit `-a` to take the account marked default". Tests: `default_is_a_name_like_any_other` (an account named `default` is taken by that name while another is marked default, and the name missing shows the hint; both fail on the old code) and `no_name_and_an_empty_one_take_the_default_account`.

Capabilities moved: toml (cairn/spec/toml.md, "An account is taken by name or as the default" modified).
