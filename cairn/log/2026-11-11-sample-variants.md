---
cairn: log
change: sample-variants
landed: 2026-11-11
---

# Sample variants

The `sample` cargo feature (no dependency) holds src/sample.rs: `sample::variants(sample, sections) -> Vec<sample::Variant { name, text }>`, lifted from the test helper of Ortie's src/config.rs (`code`, `key`, `swap`, `variants`), which every CLI moving to Ortie's sample shape would otherwise copy.

Kept from Ortie: each section uncommented whole, one `#` off every line but the prose; on top of it, each commented element inside it and each commented line of code outside the sections, swapped in for the active lines of its field, an active array commented through its closing bracket, a secret's `raw` and `command` being one field; variants named `{heading}` and `{heading}, {code}`.

Generalized past Ortie's sample, each locked by a test that fails on Ortie's rules:

- The app names its sections by heading, a heading opening a paragraph (first line or after a blank one), rather than every all-commented headed paragraph counting: a documented option group such as `# How the columns are fitted` followed by commented alternatives would have been taken for a section. A heading opening no paragraph panics, naming it.
- A commented field opening a multi-line array is uncommented through its closing bracket, its elements making no variant of their own; Ortie uncommented the opening line alone, which cannot parse.
- A field's key must be a bare or dotted TOML key, so prose such as `` `composer = "x"` runs `` stays prose; Ortie only refused keys holding a space.
- A closing bracket followed by a comment still closes an array, and a sample with no section is its own base, named `as written`.

Ortie's test logic is ported as unit tests over an inline sample, toml parsing every variant, plus the exact variant list, a section base, a field swap, the array blocks, the no-section case and the panic.

Capabilities moved: sample (cairn/spec/sample.md, new: "Each section is a base variant" and "Each commented line of code is a variant on top of each base").

Verified: as for account-resolution, the same day; `cargo clippy --all-targets --no-default-features --features sample -D warnings` included.
