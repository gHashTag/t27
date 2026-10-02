# NOW -- Six fixed artefact names in one shared TMPDIR: two concurrent corpus runs corrupt each other's columns, with every unresolved counter reading 0 (published 2026-09-24)

## A bee's work on #3034, published from `queen-3034` (Closes #3034)

- The branch changes 2 file(s): `bootstrap/src/service.rs`, `bootstrap/tests/corpus_unresolved.rs`.
- `git diff --stat origin/master...queen-3034` reads: 2 files changed, 30 insertions(+), 11 deletions(-)
- This entry is written by the publisher, not by the bee. A pull request must
  add exactly one `docs/now/` entry and a bee has no way to know that: its brief
  names a boundary file and acceptance criteria, and `docs/now/` is neither.
- What this entry does NOT establish: that the work is correct. The gates on the
  pull request judge that, and they are the same gates every other change meets.
