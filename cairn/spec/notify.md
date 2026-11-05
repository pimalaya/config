---
cairn: spec
capability: notify
status: current
---

# Notifications

The `notify` module reads desktop notifications from a configuration: `Notification`, ready to show, and `NotificationTemplate`, whose variables the caller fills when it shows it. Both are always compiled; the `notify` cargo feature decides whether anything is behind them.

### Requirement: A notification template reads as a notification does
`notify::NotificationTemplate` SHALL deserialize from the table `notify::Notification` reads, `{ summary, body }`, the summary required and not blank, the body defaulting to empty, unknown keys refused, and SHALL serialize back to it, omitting an empty body. A malformed variable SHALL be refused when the template is read. Without the `notify` cargo feature, deserializing or serializing one SHALL fail naming the feature.

#### Scenario: A blank summary
- **GIVEN** `notify = { summary = " ", body = "x" }`
- **WHEN** it is read as a template
- **THEN** reading fails, saying the summary is empty

### Requirement: A template renders with the shell's syntax
`render` SHALL expand `$name` and `${name}` to the value the caller's lookup answers, `${name:-default}` to the default when that value is empty, and `$$` to a `$`, in both the summary and the body, and SHALL answer the ready notification. A name starts with an ASCII letter or `_` and goes on with ASCII letters, digits and `_`; a `$` followed by neither `$`, `{` nor a name SHALL stay as it is, and a default SHALL be taken as written, never expanded. A name the lookup does not know SHALL fail the render, naming it. Any other `${...}` form, `${name:default}` and an unclosed `${` among them, SHALL be refused naming it, so a template written for another syntax is never half expanded. Nothing else SHALL be expanded: no tilde, no path, no command.

#### Scenario: A value and a default
- **GIVEN** the template `summary = "New mail from $senderName"`, `body = "${subject:-no subject}"`
- **WHEN** it renders with `senderName` = `Alice` and `subject` empty
- **THEN** the notification reads "New mail from Alice", "no subject"

#### Scenario: An unknown name
- **GIVEN** the template `summary = "$sendr"`
- **WHEN** it renders with a lookup knowing `senderName` alone
- **THEN** rendering fails, naming `sendr`

#### Scenario: A literal dollar
- **GIVEN** the template `summary = "Pay $$5, $5 or $"`
- **WHEN** it renders
- **THEN** the notification reads "Pay $5, $5 or $"

#### Scenario: Another syntax
- **GIVEN** `notify = { summary = "s", body = "${subject:none}" }`
- **WHEN** it is read as a template
- **THEN** reading fails, naming `${subject:none}`

### Requirement: A template is checked against the names a caller declares
`check` SHALL fail, naming the first variable the caller does not declare, and SHALL succeed otherwise, a `${name:-default}` included, without any value being asked for.

#### Scenario: A configuration read
- **GIVEN** a caller declaring `id` and `collection`
- **WHEN** it checks `summary = "$id in ${collection:-?}"`, `body = "${mailbox}"`
- **THEN** the check fails naming `mailbox`

### Requirement: No consumer needs a cfg
`NotificationTemplate::new`, `render`, `check`, `summary` and `body` SHALL exist in every build, and in a build without the `notify` feature `render` SHALL fail naming the feature.
