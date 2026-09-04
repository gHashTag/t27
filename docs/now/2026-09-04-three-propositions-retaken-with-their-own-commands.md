# NOW -- Three propositions re-taken with their own commands (2026-09-04)

## Every one had drifted, and each command was named in the document

- **lex-dropped: 1,135 -> 1,953.** `t27c lex-dropped --specs-dir specs` prints
  `1953 TOTAL across 30 spec(s)`. Two things moved in opposite directions and
  the figure predates both: the corpus went 608 -> 650 walked specs, and five
  days AFTER the number was published `#` became a line comment (W661), which
  *removes* drops. A build-free ruler modelling the lexer's drop arm reproduces
  **1,135 exactly** on the publishing commit's tree, which is what makes 1,953
  a competing count rather than an estimate -- and the canonical compiler agrees
  with the ruler to the digit.
- **cc-gate: 101 of 397 -> 290 of 650.** `specs scanned 650 / COMPILE 290 /
  FAIL 291 (UNWRITTEN 96) / no header 69`. Both halves of the ratio moved, the
  denominator by 253.
- **impl-status: 608 specs -> 650, 2,854 functions -> 4,579, 667 no-body -> 817.**
  And the comparison is not a subtraction: W689 split `specs with NO functions`
  (69 today) out of `fully implemented`, which the command says in its own
  output -- *"61 specs holding one module line and two `use`s ... overstating it
  by 21%"*. The W586 232 and today's 316 are not the same measurement.
- Each re-take sits beside the original, anchored to a commit, with the first
  measurement kept as the record it is labelled to be.
- A guard so the re-takes do not rot in turn: every `RE-TAKEN AT` block must
  name a commit, and the corpus size they quote must still match the tree.
  Three mutants -- strip one anchor, let the corpus figure go stale, rename the
  marker -- all three fail it.
