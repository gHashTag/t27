# NOW -- t27b: @compileAssert lowers as assert, integer `as f64` as @floatFromInt (2026-10-05)

## t27b @compileAssert and integer as f64 (Closes #6319)

- `@compileAssert(cond[, "msg"])` lowers through the same arm as `assert` in `cli/t27b/src/lower.rs`, because t27c's Zig backend does exactly that: `if (!(cond))` fails the compile inside an `invariant` (a `comptime` block) and is a runtime check inside a `test`.
- `x as f64` with an integer `x` lowers as `@floatFromInt` with result type f64, which is what t27c emits (`@as(f64, @floatFromInt(x))`). A float or bool operand and f64 to an integer stay refused as `ExprCast(f64)`.
- Why both: the 9 `specs/numeric/gf*.t27` files whose only lab blocker was `ExprCall(@compileAssert)` stopped next at `EXP_BITS as f64`; with both they pass, invariants checked and the JIT agreeing with the interpreter.
- Verdict check: a false `@compileAssert` in a test fails that test in both paths; in an invariant the reference reports the file as not compiling and t27b reports the invariant broken -- not a pass in either.
- Test: `compile_assert_is_assert` in `cli/t27b/tests/source.rs` (true and false, invariant and test, f64 operands, message rule).
