# NOW -- eleven gates were silent on stacked PRs (2026-08-21)

## ci: merge-critical workflows must not filter pull_request by branch (Closes #2167)

- **A gate written `on: pull_request: branches: [master]` does not run when the PR base is another branch.** On a stacked PR every such gate is absent and `gh pr checks` prints a green list -- the green of a gate that never fired. Observed here on three gates at once; those were patched, and this closes the rest
- **Measured, by parsing every workflow rather than reading names: eleven, not seven.** A previous work order said seven; that number came from a filtered `grep` and was wrong. Recorded rather than quietly replaced, because the seven was quoted downstream
- `paths:` filters are kept -- they select by what changed, not by target. `push:` branch filters are kept too: restricting post-merge runs is a cost decision, not a gating hole
- **The fix is a configuration test, not vigilance.** `scripts/ci/check_pr_branch_filters.py` lists merge-critical workflows explicitly, in code, reviewed as code. A test that inferred the list -- "everything named `*-gate`" -- would stop covering a gate the moment someone renamed it. Negative-tested: 11 violations before the patch, 0 after
- **Separately measured**: `auto-merge-ready-prs.yml` does not parse as YAML, so GitHub cannot load it and it does not run. Left untouched on purpose -- auto-merge is disabled by policy, and repairing the file would restore an automation that must not run. The test reports it as a warning, since a gate that lands red and stays red for a reason nobody may fix teaches everyone to ignore red
- **`schema-validation.yml` resolved against master, not taken verbatim.** Master had independently added `push:` and `workflow_dispatch:` triggers this branch never saw; applying the branch's version wholesale would have deleted both while removing the `branches:` filter. Only the `branches:` line was dropped, so all three triggers survive -- verified by parsing the merged YAML, and `check_pr_branch_filters.py` reports CLEAN on the result
- Entry migrated from `docs/NOW.md` to `docs/now/` (the layout #2298 introduced); the original entry was dated 2026-08-15
