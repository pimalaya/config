---
cairn: log
change: template-value-is-text
landed: 2026-11-18
---

# A notification template writes a value as text

Findings KIT-3 and CRL-3 of the CLI family analysis of 2026-11-15: `NotificationTemplate::render` pushed every expanded value into the summary and the body raw. A daemon advertising `body-markup` (dunst, mako, GNOME Shell) reads Pango markup there, and dunst and mako read it in the summary too through their format string, so a mail subject or an invitation's SUMMARY holding `<a href="...">Open invoice</a>` wrote a link into a trusted surface, and a sender `Alice <alice@example.org>` made the markup invalid. Hook variables come from mail and calendar data a stranger writes; carillon and comodoro both render through this path.

- src/notify.rs: `expand` appends a value through `push_value`. A line break or a tab becomes a space, other control characters (escape sequences among them) and the bidirectional embeddings, overrides and isolates are dropped, and, where `MARKUP` holds (`cfg!(all(unix, not(target_os = "macos")))`, the XDG back end of notify-rust), `&`, `<`, `>`, `"` and `'` become entities. macOS and Windows show the body as plain text, where an entity would be shown as written.
- The template's own text and a `${name:-default}` default are kept as written: they come from the user's configuration, which may hold markup on purpose.
- Tests: `a_value_is_escaped_and_the_template_kept` (XDG platforms) and `a_value_loses_its_control_characters`, both failing against the old code with the value written raw.

Consumer impact: carillon's `an_absent_variable_expands_to_nothing_or_its_default` expects the summary ` from Alice <alice@example.org>`, which now reads ` from Alice &lt;alice@example.org&gt;` on Linux; it follows when carillon moves to this release.

Capabilities moved: notify (A value is text, never markup, added).
