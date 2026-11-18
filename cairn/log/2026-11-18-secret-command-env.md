---
cairn: log
change: secret-command-env
landed: 2026-11-18
---

# A secret command is told it is one

Finding COH-1 of the CLI family analysis of 2026-11-15 (with ORT-2, and the first collision of KIT-6, which is the same one): two kit rules meet on the token helper. A program of the family prints its failure on standard output (pimalaya-cli cairn/spec/error.md), and `Secret::get` never reports a failing command's standard output, where the secret comes out (2026-11-14, pimalaya/config#1). A failing `ortie token show` therefore reached himalaya, carillon and neverest as "Secret command error: /bin/sh failed with exit status: 1", whether no token was stored, the refresh token had expired or the endpoint was down. Reproduced on the live head (07e2c37) by the new test before the fix, with that exact sentence.

Reading the reason off standard output was rejected: piped without `--json`, ortie's failure is the text "Error: ...", which cannot be told from a secret printed before a failure, so any parse would weaken the 2026-11-14 rule. The resolver instead tells its child what it is:

- src/secret.rs: `COMMAND_ENV`, `PIMALAYA_SECRET_COMMAND`, set to `1` on every secret command, after the `PIMALAYA_CLI_SPINNER_CHILD` precedent of pimalaya-cli. Standard output stays unreported.
- Test: `a_family_helper_failing_is_reported_by_its_reason`, a helper writing its failure on standard error only under the variable, failing against the old code with "/bin/sh failed with exit status: 1".

The reason reaches the apps only once the other side lands: pimalaya-cli's `ErrorReport::eval_coded` writes the failure report on standard error, not standard output, when `PIMALAYA_SECRET_COMMAND` is `1`, and ortie moves to that release. The apps resolving secrets through `Secret::get` (himalaya, neverest, carillon, cardamum, calendula, and ortie for its own secrets) need only the pimalaya-config bump; comodoro resolves none. ortie's storage commands surfacing their standard output (ORT-1) are ortie's own and untouched here.

Capabilities moved: secret (A secret command is told it is one, added).
