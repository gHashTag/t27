# NOW -- A comment is not a type (2026-08-29)

## A comment is not a type (Refs #2774)

- `tri types dup` decides CONFLICTED vs DUPLICATED by comparing field lists, and
  it read `cell_count : u32,   // number of standard cells` with the comment
  **inside the type**. Two definitions differing only in their comments would be
  called a conflict.
- **Blast radius measured BEFORE the fix: zero.** Of the 80 conflicted names,
  none rests on a comment difference. The published 46 DRIFT / 34 DISTINCT
  classification is untouched.
- Fixed anyway, and the fix moved no verdict: `tri types ratchet` stayed CLEAN at
  80/80 and `tri types classified` stayed OK. That pair is the control.
- A latent defect is one that has not decided anything YET. Measuring first is
  what turns "I found a bug" into "the bug changed nothing, and here is the
  number" -- fixing first destroys the evidence that it was harmless.
- Named by an agent in passing, three iterations after I wrote the code, while
  it was reading my source for a different reason. ci-gates 251. 297 tests pass.
