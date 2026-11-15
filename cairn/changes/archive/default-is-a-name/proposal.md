---
cairn: change
id: default-is-a-name
status: landed
created: 2026-11-15
---

# `default` is a name like any other

## Why

`take_account` read the name `default` as the account marked default. A word reserved inside a name the user chooses made an account named `default` unreachable whenever another one was marked, and sent `-a default` to that other account without a word. The `-a` help of every binary never mentions it, an absent `-a` already means the default, and the commands running over every account (neverest, carillon `watch`) have no default to point at. The owner chose on 2026-11-15 to drop it everywhere.

## What

An absent or empty name takes the account marked default; any other name, `default` included, is looked up as written. `Missing` for the name `default` adds "omit `-a` to take the account marked default", so a script relying on the old meaning learns what to change.
