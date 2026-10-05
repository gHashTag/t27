# NOW -- t27b: assert(cond, "msg") lowers like the reference (2026-10-05)

## assert with a message (Closes #6305)

- t27c's Zig backend emits `assert(cond, "msg")` as `if (!(cond)) @panic("msg")` and never evaluates the message, so t27b now lowers it exactly like `assert(cond)` when the message is a string literal.
- A non-literal message is refused by name, `ExprCall(assert with non-literal message)`; three or more arguments stay `ExprCall(assert with message)`. No spec under `specs/` hits either refusal today.
- `t27b corpus specs` at master 7e69777bc: pass 258 -> 277, rejected 670 -> 651, pass_vacuous 279 -> 279, JIT/interpreter mismatch 0 -> 0. Each of the 19 newly passing files agrees test for test with `t27c test-report` built from the same master.
- New test `assert_with_message_checks_the_condition_only` in `cli/t27b/tests/source.rs` covers a passing test, a failing one (trap at the assert's line) and an invariant with a message, plus the refusal.
