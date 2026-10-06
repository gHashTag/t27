# NOW -- t27b: locals declared with no type (2026-10-06)

## untyped var from an integer literal, unread undefined const (Closes #6995)

- New conformance spec `specs/tri/t27b/conformance/untyped_local.t27`. It gives 6 pass and 0 vacuous under `t27c test-report`, and the same 6 under t27b.
- `var i = 0;` is typed the way t27c's Zig backend types it (`var i: u32 = 0;`, from `zig_int_literal_default_type`). A literal that fits u32 is u32 and a larger one is u64. Hex and binary literals are typed by value, and a suffix such as `1u64` keeps its width. A negative or a float literal gets no type; Zig refuses those, and t27b still does too.
- `const c = undefined;` with no type that nothing reads is a no-op. The reference prints `_ = c; // dead after const-inlining` after it. If the const is read, or its name appears inside a string literal (the reference's use count is a text scan), t27b still refuses it.
- The code is a small edit to `cli/t27b/src/lower.rs`, with tests in `cli/t27b/tests/untypedlocal.rs`. It is owner-approved foreign code (label on #6063), listed in `tools/policy/foreign-exceptions.txt`.
- Lab, master 6540a678f against this branch, on the same tree and reference: 766 -> 769 of 828 reference-passing specs (522 -> 525 pass, 244 vacuous). Mismatch is 0 and reference disagreements are 0. Moved to pass: `orbitofrontal_value.t27`, `submit.t27` and the new spec. `tnf17_jtag.t27` goes from blocked to fail, and the reference fails it in the same test.
- The ledger `docs/reports/t27b_expectations.json` is master's plus these 3 moves to pass. The cap goes from 55 to 54.
