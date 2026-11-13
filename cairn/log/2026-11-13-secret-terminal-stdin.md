---
cairn: log
change: secret-terminal-stdin
landed: 2026-11-13
---

# The secret command reads a terminal

`Secret::get` spawned the secret command with a null standard input, since d784794 dropped io-process and for no recorded reason, so a command prompting for a password, `systemd-ask-password` first, found no terminal and waited for a password agent instead, hanging the caller ([pimalaya/config#2](https://github.com/pimalaya/config/issues/2)). The command now inherits standard input when it is a terminal (`std::io::IsTerminal`) and keeps a null one otherwise: input piped to the caller, a message sent with `himalaya message send < file` or the JSON-RPC stream of `himalaya mcp`, never reaches it. Standard output and error stay captured, the value and the failure message being read from them.

No test came with it: cargo test gives no terminal on standard input, so the terminal branch cannot run there, and the null branch passes against the old code as well.

Capabilities moved: secret (new spec, holding this requirement).
