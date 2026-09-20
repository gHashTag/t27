# NOW -- Ninety-two issues no bee could take again (2026-09-20)

## Re-filed, because a spent dispatch row keeps its issue for good (Closes #4440)

- A dispatch row that spends its retry ceiling is released once, after an hour, and then stays `rejected` on purpose -- "a third identical failure is evidence about the issue, not about the attempt". The rule is right. Its consequence was not acted on: the ISSUE stays claimed, so no bee can take it again, however much the brief improves.
- Measured: **101 open issues carried a `queen-*` branch, no pull request and no running bee.** 92 of them were stuck behind a branch that was empty or had **no merge base with master at all** -- orphan debris from before the container stopped cloning shallow. Every one had been held since 2026-09-17, through two corrections to the brief they were written against (#4296, #4302).
- There is no external way to release a row, so `tools/queen/refile.py` copies the work into a new issue -- which has no row -- and closes the old one with a pointer. Title, body and `## Boundary` carry over byte for byte; the new issue additionally carries today's toolbelt.
- **Measured after: the review column fell from 59 to 15, the backlog rose from 553 to 590, and the swarm's next dispatch was #4429 -- one of the re-filed issues.**
- The tool found its own blind spot first. `git diff A...B` on two histories with no merge base prints `fatal: ... no merge base` to STDERR and exits non-zero; the helper returned stdout and stderr together, so that sentence read as "this branch changed one file" and 93 branches were skipped as work to publish. Reading stderr as data is the same defect class as a feeder that exits green when it cannot see the swarm.
- What it refuses: a branch with real commits (that is work to PUBLISH), a branch newer than the cutoff, an issue with a pull request, an issue a bee is running right now. And if the board does not answer, it refuses to run at all rather than re-file an issue somebody is working on.
- The census moved because this adds a workflow: `quiet` 59 -> 60 workflow files, `shell` 59 -> 60 files, 80 -> 81 jobs, 272 -> 273 run-steps.
