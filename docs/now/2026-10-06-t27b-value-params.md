# NOW -- t27b: arrays and structs the body assigns (2026-10-06)

## value parameters and locals (Closes #6569)

- New conformance spec `specs/tri/t27b/conformance/value_params.t27`. It gives 10 pass under `t27c test-report`, and the same 10 under t27b.
- An array or struct parameter that the reference makes a `var` (`var p = p_arg;`) is copied into a slot of the callee at entry, and the body writes that copy. The reference decides this with t27c's own `collect_mutable_names` test, and the caller's value never changes, as in the reference.
- An array or struct local that the same test names (`const` or `let`, typed or not) is a writable slot of its own, as the reference's `var` is.
- A name written only deeper (`p.f[i] = ...`, or a write inside a block the test does not walk) stays a constant in the reference, which refuses the write. t27b still refuses it too.
- Code is in the new submodule `cli/t27b/src/lower/refvars.rs`, with small hooks in `lower.rs`. Tests are in `cli/t27b/tests/valueparams.rs`. One case in `tests/source.rs` that expected `const p = Pt{..}; p.x = 3;` to be refused is dropped, because the reference accepts it.
- The ledger `docs/reports/t27b_expectations.json` is blessed in this PR (Q53). The run is the lab's master run 597216da4, with this branch's own t27b verdicts for the 3 specs it unlocks and for 3 specs newer than that run. 0 entries move down.
- The cap goes from 43 to 44: 3 entries move up, and 4 new not-pass entries are added with `--accept-new`. Those 4 are `specs/numeric/gft{128,256,512,1024}.t27`, which are blocked on wide integer types.
