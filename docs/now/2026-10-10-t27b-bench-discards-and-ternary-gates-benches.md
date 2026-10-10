# NOW -- t27b: a bench's `_ = e;` is a discard; ternary_gates' benches type-check once referenced (2026-10-10)

## t27b blocker after c7570433e (Closes #8546; see #8195)

- arty_a7_integration.t27: its bench discards a u32, a u32, then a bool at the top level. t27b's test-binding path bound `_` as a u32 variable and refused the bool (`type mismatch`). t27c prints each one as Zig's `_ = e;`. specs/tri/t27b/discard_plan.t27 now evaluates and drops every `_ = e;` at the top of a bench. The glue changes one line.
- ternary_gates.t27: t27b was right. Once a test calls the benches, zig 0.16 refuses them three ways: signed `%`, an i32 into a u32, and a run-time index into a tuple. The spec now uses a `[3]i32` array and `@intCast(result + 1)`. With a test that calls all four benches, the reference passes 13/13.
- t27b `test --check` passes both specs: 12/12 with 122 asserts and 6/6 with 13. Both ledger rows are back to pass, and max_not_pass goes from 27 to 25.
