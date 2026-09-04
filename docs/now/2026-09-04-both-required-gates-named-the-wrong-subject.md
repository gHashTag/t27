# NOW -- Both required gates named the wrong subject (2026-09-04)

## A gate that could not compare must not report what it did not find

- `scripts/ci/now-sync-gate-diff.sh:39` -- `${PR_BASE_SHA:?}` catches unset and
  empty, not a SHA absent from the checkout. `git diff` exits 128, the `|| true`
  that absorbs grep's no-match absorbs that identically, and the gate printed
  `SYNC REQUIRED: this PR/push adds no docs/now/ entry` about a comparison it
  never made. Reproduced with a bad base SHA: `fatal: bad object`, then the
  wrong verdict, exit 1.
- `.github/workflows/issue-gate.yml:44` -- `workflow_dispatch` carries no
  `pull_request` object, so `PR_TITLE` and `PR_BODY` are empty and the gate
  printed `L1 TRACEABILITY violation: No issue reference found in PR
  title/body` against a pull request that does not exist.
- Both are required contexts in `docs/BRANCH-PROTECTION.md`, so each wrong
  message lands on a check nobody can merge past.
- The NOW gate now verifies both revisions with `git cat-file -e` and exits **2**
  when one is absent, naming the variable and the value. Resolvable revisions
  produce exactly the verdict they did before.
- The Issue Gate reads `PR_NUMBER` and, when there is no pull request, says it
  examined nothing and exits **0**. POSIX 1003.3 calls that UNTESTED: no
  subject, which is different from an instrument that failed.
- Four mutants, four kills, and two of them are caught only by the controls:
  refusing every revision kills the resolvable-pair control, and taking the
  no-PR branch always kills both real-PR controls.
- The test extracts the Issue Gate's `run:` body from the workflow rather than
  restating it, so editing the YAML and not the test is caught.
