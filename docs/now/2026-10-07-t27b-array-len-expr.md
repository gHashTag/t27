# NOW -- t27b: an array length written as a constant expression (2026-10-07)

## `[W*H]u8` and `[N]T` where N is itself an expression (Closes #7347)

- New conformance spec `specs/tri/t27b/conformance/array_len_expr.t27`. It gives 5 pass with 0 vacuous under `t27c test-report`, and the same 5 under t27b, with 16 runtime asserts.
- t27b refused an array type whose length is not a bare literal or a bare constant name (`type [N]T`). It now folds the length at compile time: integer literals, module constants (typed or untyped, including one defined by another expression), `+ - *`, the wrapping forms, parentheses and unary minus. A length that does not fold to a non-negative integer is still refused: a division, a call, a run-time value, a negative length, or a typed constant that overflows its type. Module constants with a declared type are folded the same way, so `const R = Q +% 10` with `Q: u8 = 250` is 4, and `const OV = B + 100` with `B: u8 = 200` is refused as the reference refuses it.
- 7 mutants each fail the same tests in t27b and in the reference. Of 22 probes, 9 pass in both, 2 fail in both, and 8 are blocked by the reference and refused by t27b. The other 3 are refused by t27b while the reference passes: a length using `/`, a length using a fn call, and a 50 KB local array over the frame limit.
- Code is in `cli/t27b/src/lower.rs`. Tests are in `cli/t27b/tests/source.rs`. Both files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Lab corpus at --jobs 2 against master: t27b pass where the reference passes 818 -> 819, mismatch 0. Ledger `docs/reports/t27b_expectations.json`: the new spec moves to pass. `routes/memory.t27` now stops at `FnDecl(frame size)`.
Refs #6063. Closes #7347.
