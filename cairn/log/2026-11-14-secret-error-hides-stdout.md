---
cairn: log
change: secret-error-hides-stdout
landed: 2026-11-14
---

# A failing secret command never reports its standard output

`SecretError::Output` carried a failing command's trimmed output, standard output first, so a command printing the secret and then failing (`pass show mail && some-hook`, the hook failing) put the secret in the error the applications print and log. Reported with a fix by SORALNV in [pimalaya/config#1](https://github.com/pimalaya/config/pull/1), which reported the exit status alone; reworked on its branch to keep standard error, the diagnostic channel (`gpg: decryption failed`, an entry missing from the store), and 48c8c63's program and status for a command writing none.

- src/secret.rs: the error is the trimmed standard error, else `"{program} failed with {status}"`, where 48c8c63 said `printed nothing`, which a command printing on standard output alone made false.
- Tests: `a_failed_helper_never_exposes_its_stdout` (stdout with stderr, stdout alone; `Display` exact, `Debug` free of the marker) fails against the old code with `Secret command error: SECRET_MARKER`; `a_successful_helper_still_returns_the_first_line` from the PR; `a_silent_failing_command_names_its_program_and_status` follows the new wording.

A command reporting its errors on standard output alone now fails with its program and status only: the price of never echoing the channel the secret travels on.

Capabilities moved: secret (A failing secret command never reports its standard output, added; the capture requirement points to it).
