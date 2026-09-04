# NOW -- Refused because behind is not refused (2026-09-04)

## Exit 4 could not tell a race from a verdict

- `strict_required_status_checks_policy` means every merge that lands makes every other open pull
  request stale. The refusal that follows is **not a verdict about the change** -- it is a race
  the caller wins by taking another round.
- `tri pr ready --merge` returned **4** for both, so a caller looping on 4 spins forever on a
  genuinely dead pull request, and a caller stopping on 4 gives up on one a single command fixes.
- New: **exit 5** says the refusal is the staleness race, and the command prints the remedy
  (`gh pr update-branch`). Unknown wording stays **4** -- stopping on an unrecognised refusal is
  the safe way to be wrong, and looping on one is not.
- Priced, same morning: four pull requests, **5 content commits against 7 `update-branch`
  merges**, each a full re-run of checks that had already passed on the same tree (#3134).

**Mutation found an untested clause the tests hid.** Six tests, all green, and deleting the
`not up to date` clause left every one of them passing: the real `gh` fixture carries *both*
spellings, so the third clause answered it and the first was never exercised. A fixture carrying
only the first spelling now makes that clause the only thing that can answer. Three mutants tried,
three dead -- and the one that survived is the one worth recording.

Refs #3134
