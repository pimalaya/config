---
cairn: log
change: secret-command-named
landed: 2026-11-18
---

# A failing secret command is named for the user to run

Finding COH-1 of the CLI family analysis of 2026-11-15 (with ORT-2, and the first collision of KIT-6, the same one), ruled by the owner on 2026-11-18. A program of the family prints its failure on standard output, and `Secret::get` never reports a failing command's standard output, where the secret comes out (2026-11-14, pimalaya/config#1). A failing `ortie token show` therefore reached himalaya, carillon and neverest as "Secret command error: /bin/sh failed with exit status: 1", whether no token was stored, the refresh token had expired or the endpoint was down.

The ruling keeps both rules and adds a hint:

- The caller still never reads a failed helper's standard output: it reports the program, the exit status and the standard error.
- Standard error stays for logs. Moving failures there has broken callers before (vim among them), and standard output holds the result or the failure, family-wide.
- No environment variable or configuration option tells a helper it runs as a secret command. The `PIMALAYA_SECRET_COMMAND` attempt (ecfc799) was reverted (771d950) and never released.

What landed:

- src/secret.rs: `SecretError::Output { command, reason }` (breaking), whose message ends with ", run `{command}` yourself to see why". It names the command as configured and nothing else of the run, no output and no environment.
- src/command.rs: `Display` for `CommandConfig`. A shell line is written as it was configured, and a program with its arguments is joined by spaces, each word quoted for a POSIX shell when it needs to be, so the hint can be pasted into a shell.
- Tests: `a_failing_command_is_named_for_the_user_to_run` covers a shell line and an argv with a quote to escape. It failed on 771d950 without the hint. `a_failed_helper_never_exposes_its_stdout` now builds its marker with `printf SECRET_%s MARKER`, because the command line is now part of the message and must not contain the marker it checks for. The other exact messages now end with the hint.

Apps need only the pimalaya-config bump: none matches on `SecretError::Output`. ortie stays as it is.

Capabilities moved: secret (A failing secret command never reports its standard output: the hint and the COH-1 ruling added).
