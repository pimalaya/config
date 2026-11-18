---
cairn: log
change: secret-never-echoed
landed: 2026-11-18
---

# A TOML error never echoes a secret

Finding KIT-1 of the CLI family analysis of 2026-11-15: `imap.sasl.plain.password = "<value>"`, the natural mistake for `password.raw`, failed the load with the value twice, in the source line the TOML error quoted and in serde's "unknown variant `<value>`", `Secret` being a derived externally tagged enum reading a bare string as a variant name. The merged-files path quoted no line and leaked the same through serde's message. Every app printed it, on stdout under `--json` and on an MCP server's standard error, and the usual next step is pasting the error into an issue.

- src/secret.rs: `Secret` deserializes by hand from a table holding `raw` or `command` (`cmd` its alias). A bare string, number or boolean, or an empty table, is refused with "a secret is a table: `raw` for a literal value, `command` for a command printing it"; a `raw` holding no string with "a secret's `raw` is a string"; another key and both keys each with a sentence naming keys only. Serialization is unchanged.
- src/toml.rs: `Unlisted`, which rewrote an unknown key's message inside the TOML error's display, becomes `ParseError`, built for every parse error of `from_paths`, the base and layer files of the merged path included: "TOML parse error at line {line}, column {column}: {message}", the position computed from the error's span, or "TOML parse error: {message}" when merged files lose it. The source line and its caret are gone for every key, not only a secret's: a syntax error has no type to tell a secret by.
- Tests: `a_bare_secret_is_never_echoed` and `a_parse_error_shows_its_position_never_its_line` load a fake value through one file and through two layers, and fail against the old code (the value in the quoted line); `a_refused_secret_never_echoes_its_value` checks each refusal's exact message. The four unknown-key tests follow the new format.

Users lose the quoted line and its caret, and keep the path, the line and the column.

Capabilities moved: secret (A refused secret never echoes its value, added), toml (A parse error shows its position, never its line, added; the unknown-key requirement points to it).
