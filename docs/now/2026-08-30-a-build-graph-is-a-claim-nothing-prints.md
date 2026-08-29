# NOW -- A build graph is a claim nothing prints (2026-08-30)

## A build graph is a claim nothing prints (Refs #2895, Refs #2898)

- `proofs/lean4/Trinity.lean` is twelve lines and reaches eleven files; twelve more sit beside them, 15553 lines, compiled by nothing
- it is twelve stranded files, not the eleven reported when this was found: `GoldenFloatRoundTrip.lean` is not under `IcarusLowerable/` and the subtree framing lost it
- the reachable half is 7355 lines, not the 7447 reported then
- `lake build` prints what it compiled and never what it skipped, so a stranded file produces no output at all
- four of the five `sorry` counted by `lean-proofs.yml` are in files nothing opens: that gate greps the directory, the build compiles the closure
- `tri lean reach` refuses rather than answering when the lakefile sets `globs`, names no lib, or the root reaches only itself
