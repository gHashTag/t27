# NOW -- t27b: tail expressions and discarded calls as the reference prints them (2026-10-06)

## value-ignored statements (Closes #7114)

- New conformance spec `specs/tri/t27b/conformance/value_ignored.t27`. Under `t27c test-report` it gives 4 pass and 0 vacuous; t27b gives the same 4. A negative control (one wrong expected value per test) fails 3 of 4 on both sides.
- Since #6315, the reference's Zig backend returns a non-void fn's last bare expression (`zig_tail_returns`), and goes into the last statement of each if/else branch at any depth. t27b now marks the same tails and lowers them as `return`.
- A statement that only calls a module fn returning a value is printed `_ = f(x);` by the reference (`call_returns_value`). t27b now evaluates the call and drops its value. Any other value-ignored statement is still refused where Zig analyzes it.
- `return undefined;` (and a tail `undefined;`) is handled by where it sits:
  - In a fn nothing reaches, it is a trap no test can hit.
  - In a reached fn with a scalar result, it is refused as `ExprReturn(undefined)`, because t27b does not guess Zig's undefined value.
  - In a reached fn with an aggregate or optional result, the result is left unwritten, as before.
- Two extra conformance specs go in with no Rust change, since they cover cases that #6998's and #7045's own specs do not. `comptime_float_f128.t27` (7 tests) covers difference, int*float, quotient, negation, the golden residue and f32 midpoint literals. `untyped_local_uses.t27` (3 tests) covers untyped `var` locals that are reassigned and passed on, and `const x = undefined;` in a test. Both pass 7/7 and 3/3 under the reference and t27b, and their negative controls fail on both sides.
- Code: `cli/t27b/src/lower.rs`. Tests: `cli/t27b/tests/{source,tail}.rs`, each new case checked against the reference first.
- Lab (master 96cf7c8 tree and reference, before and after): 5 ledger entries go from blocked to pass (`specs/compiler/{zig_value_ignored,rust_tail_returns,verilog_branch_context}.t27`, `specs/fpga/bpsk.t27`, `specs/isa/ternary_encoding.t27`). The unlisted `specs/automation/review-jam.t27` passes and is added. `specs/tri/crypto/hex.t27` stays blocked, now on `ExprCall` (after master's #7111). On the merged tree: cargo test all ok, UNEXPECTED FAILURE 0, JIT/interpreter mismatch 0, reference disagree 0 of 809 files.
- Ledger: master's plus these moves only. The cap goes from 34 to 29.
