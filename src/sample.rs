//! Variants of a sample configuration, for its tests.
//!
//! A documented sample holds its account's provider settings in
//! commented sections, a user uncommenting one, then the options they
//! need. [`variants`] answers what a user may write from it, for a test
//! to parse each with the app's own configuration type.

use std::ops::Range;

/// A sample configuration with part of its commented code uncommented.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variant {
    /// What was uncommented: the section heading, then the code line.
    pub name: String,
    /// The TOML text.
    pub text: String,
}

/// The variants of `sample`, whose account sits in the sections the
/// `sections` headings open.
///
/// A section is the paragraph a heading line opens, up to the next blank
/// line. Each is uncommented whole, one `#` off every line but the prose,
/// then on top of it each commented element inside it and each line of
/// commented code outside the sections, in place of the active lines
/// setting its field. A field is `key = value`, spaced around `=`, an
/// element a line opening on `"`; a commented field opening an array
/// comes with its lines through the closing bracket. With no section,
/// the sample as written is the one base.
///
/// # Panics
///
/// Panics when a heading opens no paragraph of `sample`.
pub fn variants(sample: &str, sections: &[&str]) -> Vec<Variant> {
    let lines: Vec<&str> = sample.lines().collect();
    let mut bases: Vec<(&str, Range<usize>)> = Vec::with_capacity(sections.len());

    for heading in sections {
        let start = lines.iter().enumerate().position(|(i, line)| {
            let opens = i == 0 || lines[i - 1].trim().is_empty();
            opens && line.trim_end() == *heading
        });

        let Some(start) = start else {
            panic!("Section `{heading}` not found in the sample");
        };

        let len = lines[start..]
            .iter()
            .position(|line| line.trim().is_empty());
        bases.push((heading, start..len.map_or(lines.len(), |len| start + len)));
    }

    if bases.is_empty() {
        bases.push(("as written", 0..0));
    }

    let mut variants = Vec::new();

    for (heading, section) in &bases {
        let uncommented: Vec<_> = section
            .clone()
            .filter_map(|i| {
                let line = lines[i].strip_prefix('#')?;
                let rest = line.trim_start();
                let code = rest.starts_with(['#', ']']) || code(lines[i]).is_some();
                code.then(|| (i, line.to_owned()))
            })
            .collect();

        let base = swap(&lines, &uncommented);
        let base: Vec<_> = base.iter().map(String::as_str).collect();

        variants.push(Variant {
            name: heading.to_string(),
            text: base.join("\n"),
        });

        let mut next = 0;

        for (i, line) in base.iter().enumerate() {
            let elsewhere = bases
                .iter()
                .any(|(_, other)| other != section && other.contains(&i));

            let Some(code) = code(line).filter(|_| i >= next && !elsewhere) else {
                continue;
            };

            let mut uncommented = vec![(i, code.to_owned())];

            if code.trim_end().ends_with('[') {
                for (j, line) in base.iter().enumerate().skip(i + 1) {
                    let Some(line) = line.trim_start().strip_prefix('#') else {
                        break;
                    };

                    uncommented.push((j, line.to_owned()));

                    if line.trim_start().starts_with(']') {
                        break;
                    }
                }
            }

            next = i + uncommented.len();

            variants.push(Variant {
                name: format!("{heading}, {code}"),
                text: swap(&base, &uncommented).join("\n"),
            });
        }
    }

    variants
}

/// The code a commented line holds, one `#` off: a field or an array
/// element.
fn code(line: &str) -> Option<&str> {
    let line = line.trim_start().strip_prefix('#')?.trim_start();

    let field = key(line).is_some_and(|key| {
        key.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.\"".contains(c))
    });

    (field || line.starts_with('"')).then_some(line)
}

/// The field a line sets, a secret's `raw` and `command` being one.
fn key(line: &str) -> Option<&str> {
    let key = line.trim_start().split_once(" = ")?.0;
    let secret = key.strip_suffix(".raw").or(key.strip_suffix(".command"));
    Some(secret.unwrap_or(key))
}

/// `lines` with the `uncommented` ones swapped in, and the active lines
/// setting one of their fields commented out, an array through its
/// closing bracket.
fn swap(lines: &[&str], uncommented: &[(usize, String)]) -> Vec<String> {
    let keys: Vec<_> = uncommented.iter().filter_map(|(_, l)| key(l)).collect();
    let mut swapped = Vec::with_capacity(lines.len());
    let mut in_array = false;

    for (i, line) in lines.iter().enumerate() {
        if let Some((_, code)) = uncommented.iter().find(|(j, _)| *j == i) {
            swapped.push(code.clone());
            continue;
        }

        let active = !line.trim_start().starts_with('#');
        let replaced = active && key(line).is_some_and(|key| keys.contains(&key));

        if in_array {
            in_array = !line.trim_start().starts_with(']');
            swapped.push(format!("#{line}"));
        } else if replaced {
            in_array = line.trim_end().ends_with('[');
            swapped.push(format!("#{line}"));
        } else {
            swapped.push(line.to_string());
        }
    }

    swapped
}

#[cfg(test)]
mod tests {
    use toml::Table;

    use crate::sample::{Variant, variants};

    const SAMPLE: &str = r#"[accounts.example]
default = true
#auto-refresh = false
#alpn = ["http/1.1"]

grant = "code"  # sign in through a browser
#grant = "device"  # sign in elsewhere

#resources = [
#  "https://example.org/a",
#]

scopes = [
  "base",
]

# Alpha
#
#client-id = "alpha"
##client-secret.raw = "secret"
#scopes = [
#  "alpha",
##  "beta",
#]

# Beta
#
# `client-id = "x"` stays prose.
#client-id = "beta"
#client-secret.command = "pass show beta"
"#;

    const SECTIONS: [&str; 2] = ["# Alpha", "# Beta"];

    fn variant(name: &str) -> Variant {
        let variants = variants(SAMPLE, &SECTIONS);
        let variant = variants.into_iter().find(|variant| variant.name == name);
        variant.unwrap_or_else(|| panic!("no variant {name}"))
    }

    #[test]
    fn every_variant_parses_and_holds_the_account() {
        for Variant { name, text } in variants(SAMPLE, &SECTIONS) {
            let config: Table = text
                .parse()
                .unwrap_or_else(|err| panic!("sample with {name} uncommented: {err}"));

            assert!(config["accounts"].get("example").is_some(), "{name}");
        }
    }

    #[test]
    fn each_section_comes_whole_then_each_line_on_top_of_it() {
        let names: Vec<_> = variants(SAMPLE, &SECTIONS)
            .into_iter()
            .map(|variant| variant.name)
            .collect();

        assert_eq!(
            names,
            [
                "# Alpha",
                "# Alpha, auto-refresh = false",
                "# Alpha, alpn = [\"http/1.1\"]",
                "# Alpha, grant = \"device\"  # sign in elsewhere",
                "# Alpha, resources = [",
                "# Alpha, scopes = [",
                "# Alpha, client-secret.raw = \"secret\"",
                "# Alpha, \"beta\",",
                "# Beta",
                "# Beta, auto-refresh = false",
                "# Beta, alpn = [\"http/1.1\"]",
                "# Beta, grant = \"device\"  # sign in elsewhere",
                "# Beta, resources = [",
            ],
        );
    }

    /// The prose stays commented, the heading included, and so does the
    /// other section.
    #[test]
    fn a_section_replaces_the_active_fields_it_sets() {
        let text = variant("# Alpha").text;

        let alpha = "\
#scopes = [
#  \"base\",
#]

# Alpha
#
client-id = \"alpha\"
#client-secret.raw = \"secret\"
scopes = [
  \"alpha\",
#  \"beta\",
]

# Beta
#
# `client-id = \"x\"` stays prose.
#client-id = \"beta\"
#client-secret.command = \"pass show beta\"";

        assert!(text.ends_with(alpha), "{text}");
    }

    #[test]
    fn a_line_replaces_the_active_field_it_sets() {
        let text = variant("# Beta, grant = \"device\"  # sign in elsewhere").text;

        assert!(text.contains("\n#grant = \"code\"  # sign in through a browser\n"));
        assert!(text.contains("\ngrant = \"device\"  # sign in elsewhere\n"));
    }

    #[test]
    fn a_commented_array_comes_whole() {
        let text = variant("# Alpha, scopes = [").text;

        assert!(text.contains("\nscopes = [\n  \"base\",\n]\n"), "{text}");
        assert!(text.contains("\n#scopes = [\n#  \"alpha\",\n##  \"beta\",\n#]\n"));

        let text = variant("# Beta, resources = [").text;
        assert!(text.contains("\nresources = [\n  \"https://example.org/a\",\n]\n"));
    }

    #[test]
    fn without_sections_the_sample_as_written_is_the_base() {
        let variants = variants("a = 1\n#a = 2\n#b = 3\n", &[]);

        let expected = [
            ("as written", "a = 1\n#a = 2\n#b = 3"),
            ("as written, a = 2", "#a = 1\na = 2\n#b = 3"),
            ("as written, b = 3", "a = 1\n#a = 2\nb = 3"),
        ];

        let expected: Vec<_> = expected
            .into_iter()
            .map(|(name, text)| Variant {
                name: name.to_owned(),
                text: text.to_owned(),
            })
            .collect();

        assert_eq!(variants, expected);
    }

    #[test]
    #[should_panic(expected = "Section `# Gamma` not found in the sample")]
    fn a_heading_opening_no_section_panics() {
        variants(SAMPLE, &["# Alpha", "# Gamma"]);
    }
}
