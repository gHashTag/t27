# NOW -- t27b: brace-invariant predicates (2026-10-06)

## brace-invariant predicates (Closes #6864)

- New conformance spec `specs/tri/t27b/conformance/brace_invariant.t27`. Under `t27c test-report` it gives 1 pass, with 11 invariants proved and 0 vacuous. Under t27b all 11 invariants hold and 14 runtime asserts are counted.
- As in the reference (#6315), each top-level statement of a brace invariant that is a predicate is checked as `assert(<expr>)`. A predicate here is a binary or unary expression, a name, an index, a field access, `true` or `false`, or a call to a fn declared `-> bool`. A void call stays an action, and an `assert` stays an assert.
- What the reference does not compile stays refused: a non-bool predicate (`{ N + 1 }`), a bare `{ 3 }`, and a predicate nested in an `if`, since only top-level statements are wrapped. A call to a non-bool fn is the `ExprCall(value ignored)` family and is not touched here.
- Code is in `cli/t27b/src/lower.rs` (+54). Tests are in `cli/t27b/tests/tail.rs`. One case in `tests/source.rs` that expected `invariant i { N == 3 }` to be refused is dropped, because the reference proves it.
- Lab numbers, with mismatch 0 and reference_disagree 0 both before and after: t27b passes 485 of the 818 specs the reference passes on master a17dc9d70, and 495 on this branch. That is 9 specs unlocked that now pass with runtime asserts, plus the new spec. `specs/isa/ternary_graph.t27` also moves from blocked to pass_vacuous.
- The ledger `docs/reports/t27b_expectations.json` is edited by hand for those 10 entries, and the new spec is added, and the cap goes from 67 to 57. A full `--bless` refuses 5 t27b timeouts that are already in master's own run, so it is not used.
