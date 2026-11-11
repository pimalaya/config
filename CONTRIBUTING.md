# Contributing guide

Thank you for investing your time in contributing to pimalaya-config.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation, starting with src/lib.rs: it is the architecture document of this crate;
4. the cairn/ folder for the development history and living plans (the Cairn convention: spec/, changes/, log/).

Everything below documents only what differs from the Pimalaya standards.

## Deliberately std

pimalaya-config loads files, spawns commands and expands environment variables, so it exposes no I/O-free coroutines and the no_std layer checks of the org guide do not apply: there is no coroutine core to keep std-free. The layers to build against are the feature-gated modules:

```sh
cargo build --no-default-features                          # the notify types alone, failing when used
cargo build --no-default-features --features toml          # TOML loader, account resolution, shell expansion
cargo build --no-default-features --features secret        # secret resolution + command adapter
cargo build --no-default-features --features notify        # desktop notifications
cargo build --no-default-features --features sample        # sample configuration variants, for tests
cargo build                                                # toml + secret + notify (default)
```
