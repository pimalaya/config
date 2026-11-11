---
cairn: change
id: sample-variants
status: landed
created: 2026-11-11
---

# Sample variants

## Why

A Pimalaya CLI documents its configuration in a config.sample.toml holding one account whose provider settings sit in commented sections, a user uncommenting one, then the options they need. Ortie's test parses every such variant through its loader, so a sample line that stopped parsing fails a test rather than a user; the helper building the variants lives in Ortie's tests (src/config.rs), and every other CLI moving to the same sample shape would copy it.

## What

- A `sample` cargo feature, off by default and meant for dev-dependencies, holding `sample::variants(sample, sections)`: given the sample text and the headings opening its sections, the variants a user may write, each a `sample::Variant` with a name and the TOML text. The app parses each with its own configuration type.
- Ortie's rules, lifted: each section is uncommented whole (one `#` off every line but the prose), then on top of it each commented element inside it and each line of commented code outside the sections is uncommented in place of the active lines of its field, an active array commented through its closing bracket.
- Generalized past Ortie's sample: the sections are named by the app rather than guessed from the paragraphs, a commented multi-line array outside the sections is uncommented whole, prose holding ` = ` is not taken for code, and a sample without sections is its own base.
- Ortie's test logic as unit tests over a small inline sample.
