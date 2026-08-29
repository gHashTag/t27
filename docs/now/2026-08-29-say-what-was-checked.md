# NOW -- Say what was checked (2026-08-29)

## Say what was checked (Refs #2762)

- `tri skill check` ended every run with `Numbering holds in 5 file(s).` Four of
  those five SKILL.md files contain **zero** numbered sections, so the line
  counted four files where nothing was checked -- the same shape as "13 gates
  green" when two of them never ran.
- It also read as "the sequence is intact" while §126 has never existed in the
  file's history (`git log -S"## 126."` returns nothing; it was skipped, not
  deleted).
- The GATE is not wrong. A gap is deliberately not a failure -- a section can be
  removed, and refusing would make an append-only log unmergeable. The SUMMARY
  LINE was wrong: it named a verdict the check had not earned.
- Now: `No number is used twice: 229 section(s) across 1 of 5 file(s) read.` plus
  a line for the files that contributed nothing and a line for every unused
  number, marked explicitly as not a failure.
- `--gaps` no longer gates that output; unused numbers are always stated. The
  flag says so rather than silently doing nothing.
- Control re-run: planting a duplicate section number still exits 1 and names
  both titles. Restored, exits 0. ci-gates 230, 266 tests pass.
