---
cairn: change
id: account-resolution
status: landed
created: 2026-11-11
---

# Account resolution

## Why

Every Pimalaya CLI resolves the account a command runs against the same way, and Ortie 3.0 and Himalaya each hand-roll it on top of `TomlConfig`: guideline cli-002 asks a missing configuration to name the path it looked for, a missing named account to list the accounts the configuration holds, and a missing default account to name both ways of picking one. The trait only answers `Ok(None)` for a missing configuration, meant to launch a wizard, `Get account `{name}` error` for an unknown name, and picks one default at random among several, the accounts map being a `HashMap` (Ortie does, Himalaya refuses it by hand).

The family drops its wizards, so `Ok(None)` no longer has a wizard to drive, and `toml::to_string`, the compact serializer a wizard printed its account with, loses its callers: every one of them (himalaya, cardamum, calendula, neverest, unfog, carillon, sirup, himalaya-tui) sits in a wizard.

## What

- `TomlConfig::sample_url`, the documented sample configuration the app ships, and `TomlConfig::accounts`, every account name with whether it is marked default, both provided by the app. `take_default_account` goes: the trait finds the default itself.
- `TomlConfig::load`: `from_paths_or_default`, a missing configuration failing with a `ConfigMissingError` naming the `target_path` and the sample URL, worded as Ortie words it.
- `TomlConfig::take_account` answers the account or an `AccountError`: `Missing` lists the accounts the configuration holds, sorted, `NoDefault` names both ways of picking one, `SeveralDefaults` names the defaults, sorted. The wording is Ortie's and Himalaya's.
- `toml::to_string` and its helpers go, and toml's `display` feature with them (kept for the tests as a dev-dependency, which also lets the `secret` feature's tests build alone).
- Docs saying `Ok(None)` drives a wizard are rewritten.
