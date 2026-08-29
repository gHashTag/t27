# NOW -- Not failing on it, never reaching it (2026-08-30)

## lake build has never compiled the 647 theorems (Refs #2895)

- `proofs/lean4/Trinity.lean` is the library root and imports NINE modules; `Trinity.IcarusLowerable.*` is not among them
- the eleven files under that directory import each other -- a closed subtree with no edge from the root -- so `lean_lib «Trinity»` never reaches any of it
- in the graph: 7 447 lines. Outside it: 15 447 lines and 647 theorems, including all 250 `native_decide`
- 67% of the Lean development has never been compiled by anything
- the evidence was one grep of the dispatched run's log: `Icarus` appears ZERO times in 481 lines
- the two most recent runs DID fail, on `H4Lagrangian.lean` unsolved goals -- a real failure in the built part, and a decoy: fixing it adds no IcarusLowerable file to the graph
- "the build is red" and "the build does not compile this" are different facts, and a red build hides the second behind the first
- this explains #2893 (a fabricated signature surviving) and the 114 empty models: nothing ever type-checked any of them
- the repair is one `import` line, it will almost certainly go red, and the sequencing is a decision rather than a measurement -- not made here
