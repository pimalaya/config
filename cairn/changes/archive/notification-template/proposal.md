---
cairn: change
id: notification-template
status: landed
created: 2026-11-05
---

# Notification templates

## Why

Every hook-based product shows notifications with variables in them, and each fills them its own way. comodoro reads `notify` through this crate's adapter and has no variables. ortie keeps its own `NotifyConfig` and expands `$EXPIRES_IN` with shellexpand (`${VAR:-default}`). carillon keeps its own `NotifyConfig` and expands `$sender_name` with subst (`${name:default}`), while the `command` of the same hook runs through `sh` with the same variables in its environment, where the default is `${name:-default}`: one product, two syntaxes. The adapter cannot help them, since what it reads is a ready `notify_rust::Notification`, and a consumer filling its `summary` and `body` itself needs a `#[cfg(feature = "notify")]` the adapter exists to spare it.

Pimalaya guideline cli-008 now fixes the shape: camelCase variables, the shell's syntax, one spelling for a notification and a command. This crate is where that syntax lives, so every product gets it from one place.

## What

- `notify::NotificationTemplate`: the TOML shape the adapter reads (`{ summary, body }`, the summary required and not blank, the body defaulting to empty), held as text.
- `render(&self, lookup)` expands both with the shell's syntax (`$name`, `${name}`, `${name:-default}`, `$$` for a `$`) and answers a ready `Notification`. `lookup` is the caller's: it answers a name's value, an empty value for a name it knows but cannot fill this time, and nothing for a name it does not know, which fails the render naming the variable. `${name:-default}` takes the default when the value is empty, as the shell does. No tilde or path expansion: a notification is text.
- `check(&self, known)` runs the same parse against the names a caller declares, without values, so a product refuses an unknown variable when its configuration is read rather than when the hook fires.
- `summary()` and `body()` give the templates back, for a product printing its configuration.
- Without the `notify` feature, reading a template fails naming the feature, as reading a `Notification` does; `render` and `check` exist in every build, so a consumer needs no `#[cfg]`.
- The expansion is a parser of the four forms, no dependency added to a `notify`-only build.
- The crate documentation states the naming rule its consumers share: variables are camelCase (guideline cli-008), the crate itself checking only the syntax.

The adapter stays, for a notification with no variables.

## Consumers

- carillon: moves to the template in its change hook-shape-follows-siblings, `${name:default}` becoming `${name:-default}` and its variables camelCase (`$senderName`).
- ortie: moves to the template, `$EXPIRES_IN` becoming `$expiresIn`, within the 3.0.0 its camelCase JSON already forces.
- comodoro: may move to the template when its hooks gain variables; nothing to rename today.

## Not done

- Templating a `command`: a command gets its variables in its environment, and its shell expands them.
