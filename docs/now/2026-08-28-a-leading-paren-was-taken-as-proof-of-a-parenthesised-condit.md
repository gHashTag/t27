# NOW -- A leading paren was taken as proof of a parenthesised condition (2026-08-28)

## A leading paren was taken as proof of a parenthesised condition (Closes #2735)

- one parse_condition replaces three byte-identical copies; corpus parsing 558 -> 559
- the first version of the fix cost six specs and the suite stayed green -- a per-spec before/after parse of all 650 caught it
- the Rust/Lean completeness test reported 1 disagreement out of 73; it aborted on the first
- 40 of those 73 are theorems about an EMPTY module: native_decide proving that nothing is lowerable
- parse-conform Case gains discards, so a row can demand accepted-and-nothing-dropped instead of a rejection
