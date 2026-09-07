# NOW -- A frozen file that nothing refused to open (2026-09-07)

## A frozen file that nothing refused to open (Closes #3368)

- `docs/NOW.md` says FROZEN ARCHIVE on line 1, and every mention of it under `.github/workflows/`, `scripts/` and `.githooks/` was a comment -- measured, zero lines rejected an edit. An author following a stale instruction could reopen it and pass all four required checks.
- Both detection rules were measured and BOTH fail: 'refuse any diff touching it' blocks the one legitimate repair in the population, and so does 'refuse an added `## ` heading' -- that repair adds three, because it restored headings whose bodies were destroyed.
- So the exception is DECLARED rather than detected: `Archive-Repair: <reason>` in the commit message, the same shape as `# tri:no-dispatch` and `# tri:cause-removed`. The gate does not judge the reason; it requires one to exist.
- Six controls on a scratch repository -- PR and push, with and without the trailer, a bare trailer, and an untouched file. Historical control: `458ec0bd6` carries no trailer and would be refused, correctly. Blast radius: 2 commits in 600.
