# NOW -- The comment named `if` and the match arm did not (2026-08-30)

## Two ways the self-checking testbench stopped checking (Refs #2869)

- `if (es_prestandard(8) != 0) { ok = false; }` at the TOP LEVEL of a test block became `// (stmt: StmtIf)`, so `ok` was set true and never set false: the test could not fail and reported PASSED
- the match arm handles StmtForRange / StmtWhile / StmtFor, and the comment directly under it says "a `for`/`while`/`if` was dropped" -- `if` was written down and never added
- 11 statements in 2 specs; neither simulates today for unrelated reasons, so no verdict changes now and the class is what closes
- separately: ALL 4702 invariants in the self-checking testbench rendered their predicate as `/* unsupported expr: StmtExpr */`, because the predicate arrives wrapped in a statement node
- unwrapping it -- the same `unwrap_single` the `while` fix needed -- renders 4697 of the 4702; five are genuinely unsupported
- corrected against the audit: the 5635 comments in synthesizable RTL are a MANIFEST, and the header above them says so. Those are a declared omission, not a hidden one. The defect is in the SIMULATION path, which is smaller and worse
