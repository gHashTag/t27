# NOW -- the drafts moved into the bodies, and the lanes filled (2026-09-24)

## A draft nobody moves across is a draft that does nothing (Closes #4707)

- `--write-body` wrote a boundary only where an issue named exactly one path and it was a `.t27`: 44 issues. That left **123 carrying a draft in a comment and nothing in the body** -- and the body is the only place the Queen reads. Owner's decision: move them.
- A body line is a CLAIM on a file, which a comment is not. So `--unchecked` drops the one-spec guard and keeps the citation rule: a `docs/` path is carried into a comment and left out of a body, because a document an issue quotes is the line most likely to be wrong and reserving it blocks whoever really owns it. An issue whose every path is a citation is skipped rather than given an empty section -- `boundaryPathsOf` reads "no section" and "empty section" alike, so an empty one would look like a boundary only to a person.
- **122 bodies written.** Alongside it, 46 issues stuck behind an empty branch were re-filed (`refile.py`), which releases a dispatch row that spent its ceiling and would otherwise hold its issue for good.
- Measured across the ticks after: `missingBoundary` **565 -> 398**, `claimed` 41 -> 74, `refusal` `None` on every tick, **active bees 1 -> 12-16 of 20**.
- What did NOT happen, and why: the 45 issues sitting `accept`-but-open were NOT closed. Their branches carry nothing master lacks, which reads like "already merged" and is not - master holds no commit naming any of them, and the files two of them were accepted for (`specs/port/tools/fuzz_trainer.t27`, `specs/port/tools/check_catalog_integrity.t27`) do not exist there. Closing them would have closed work that was never delivered.
