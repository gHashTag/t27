# NOW -- Ask the root what it reaches (2026-08-30)

## Ask the root what it reaches (Refs #2895, Refs #2900)

- five build systems, one shape: Lean imports, Rust `mod`, Cargo `members`, `_CoqProject`, a Python allowlist -- each has a short root file that is the whole specification of what gets checked
- a build system prints the work it did; a coverage claim is about the work it did not, and nothing prints that
- a red build hides it: "the build is red" and "the build does not compile this" are different facts, and the first is loud
- a `paths:` filter makes it worse than silence -- editing an uncompiled file triggers a workflow that compiles the listed ones and reports success
- edges are directional and orphan subtrees import each other busily, which reads as connectedness; undirecting them reports zero and looks like good news
- the count going DOWN is the silent direction: 358 to 354 tests, no gate reads that number
