# NOW -- An empty match is not a number (2026-09-04)

## `tri wave` said the next lesson is 1; the document's last one is 898

- `.claude/skills/t27-wave-loop.md` is 242 KB and present, its worked examples
  are headed `## Worked example -- Wave Loop 898`, and
  `grep -cE '^\*\*[0-9]+\.'` on it is **0**. The format moved past the matcher.
- `last_lesson_no` tested that the file EXISTS and then returned whatever the
  matcher produced. Empty output with exit 0 is not a failure, so
  `$(last_lesson_no || echo '-')` never substituted the `-`, and
  `$(( "" + 1 ))` is 1.
- The write path took the same route. `tri lesson` carries the guard written for
  this -- `no="$(last_lesson_no)" || die "no numbered lessons found"` -- and
  `bash -x` shows `no=` then `next=1` with the die SKIPPED. Nothing was written
  only because an unrelated downstream step failed on the same empty anchor.
- Both readers now return 1 when the matcher produced no number, so the `-` and
  the `die` work again, and `tri wave` says `UNREADABLE -- no line matched in
  <file>` rather than inventing a number.
- `theorem` is fixed the same way although it reads correctly today: the two
  halves of one display should not differ in whether they can lie.
- Not done, deliberately: widening the matcher to the `## … Wave Loop N` form.
  Which heading is canonical belongs to whoever owns the document; saying "I
  cannot read this" has to land first either way.
- Two mutants: the old reader kills the three defect assertions, a reader that
  always fails kills all three CONTROLS -- a document that does carry lessons
  must still be read and written, or the test cannot fail.
