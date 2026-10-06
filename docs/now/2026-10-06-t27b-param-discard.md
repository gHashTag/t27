# NOW -- t27b: `_ = p;` for a parameter (2026-10-06)

## parameter discards (Closes #7057)

- New conformance spec `specs/tri/t27b/conformance/param_discard.t27`. It gives 7 pass under `t27c test-report`, and the same 7 under t27b, with runtime asserts.
- `_ = p;` where `p` is a parameter now lowers to nothing. At the top level the reference's dead-store pass deletes it, so t27b accepts it there always.
- In a nested block the reference prints the discard as written, and Zig refuses it only as a pointless discard, that is when the body also uses `p`. t27b counts the body's mentions of `p` and its `_ = p;` statements, and refuses the nested discard only when the two differ. Discards alone compile: repeated, in both arms, or for a parameter that shares a module declaration's name.
- `_ = x;` for a local or an undeclared name is still refused.
- Code is in `cli/t27b/src/lower.rs`. Tests are in `cli/t27b/tests/source.rs`. Both files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- `queen-rehearsal.t27` was reached for the first time and its interpreter run faulted: `build_content_frame` returned a slice of its own local buffer. The spec now takes a caller-owned buffer. The reference still gives 7 pass, and t27b gives 7 pass with no JIT/interpreter mismatch.
- Ledger `docs/reports/t27b_expectations.json`: `leaderboard.t27`, `queen-rehearsal.t27` and the new spec move to pass. `dqn.t27` now stops at `ExprCall(undeclared fn)` and `gen_gradient.t27` at `StmtExpr statement`. The not-pass count goes from 51 to 49.
