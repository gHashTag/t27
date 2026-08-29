# NOW -- The arm's own comment named the case it was missing (2026-08-30)

## Two lessons from the dropped `if` (Refs #2871)

- the match arm covers StmtForRange / StmtWhile / StmtFor and its own comment says "a `for`/`while`/`if` was dropped" -- `if` named in the sentence, absent from the pattern
- cost: a top-level `if` in a test block became a comment, so the flag was set true and never false and the test reported PASSED
- when a comment ENUMERATES cases, that list is checkable against the pattern above it in one glance, and nobody had glanced
- my first probe read the wrong spec: `lucas_accumulator.t27` nests its `if` inside a `while`, which routes through a path that handles it, and I nearly recorded the report as not reproducing
- the claim said "TOP-LEVEL if" and I tested an if; the position was part of the claim
- getting that wrong costs more than a missed defect: it produces a confident refutation of a true finding
