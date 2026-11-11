---
cairn: spec
capability: sample
status: current
---

# Sample variants

The `sample` feature, meant for dev-dependencies, builds the variants a user may write from a documented sample configuration, for a test to parse each with the app's own configuration type.

### Requirement: Each section is a base variant
`sample::variants(sample, sections)` SHALL answer, for each heading of `sections` in order, the sample with the paragraph that heading opens (up to the next blank line) uncommented: one `#` off every line holding code, a closing bracket or a further `#`, the prose lines, the heading among them, staying comments. Each line of code it uncomments SHALL comment out the active lines setting the same field, an active array through its closing bracket, a secret's `raw` and `command` counting as one field. The variant SHALL be named after the heading. A heading opening no paragraph SHALL panic, naming it. An empty `sections` SHALL make the sample as written the one base, named `as written`.

#### Scenario: Two provider sections
- **GIVEN** a sample whose sections are `# Alpha` and `# Beta`
- **WHEN** its variants are built
- **THEN** the `# Alpha` variant holds the code of Alpha uncommented and the code of Beta still commented

### Requirement: Each commented line of code is a variant on top of each base
On top of each base, every line still commented that holds code, inside the base's section or outside every section, SHALL make one more variant, uncommented in place of the active lines setting its field and named `{heading}, {code}`. A line holding code SHALL be a field (`key = value`, the key a bare or dotted TOML key) or an array element (a line opening on `"`); prose holding ` = ` SHALL stay prose. A commented field opening a multi-line array SHALL be uncommented through its closing bracket, its elements making no variant of their own, while a commented element inside an active array SHALL make its own. Lines inside another section SHALL make no variant.

#### Scenario: An alternative value outside the sections
- **GIVEN** `grant = "code"` active and `#grant = "device"` below it, outside the sections
- **WHEN** the variants of the `# Alpha` base are built
- **THEN** one, named `# Alpha, grant = "device"`, holds `#grant = "code"` and `grant = "device"`
