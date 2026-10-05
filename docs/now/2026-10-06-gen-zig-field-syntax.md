# NOW -- gen-zig field syntax, landed as an owner exception (2026-10-06)

## bootstrap/src/compiler.rs gen-zig fix (Closes #6451, Closes #6532, debt #5980)

- gen-zig wrote four spellings verbatim, and each was a Zig syntax error that stopped the whole file before any test ran: a postfix optional type (`str?`), a field named for a Zig keyword (`error`), a fn parameter named for a keyword (`var`), and an array repeat with a Rust width suffix (`[0i32; 2]`). They now come out as `?[]const u8`, `.@"error"`, `@"var"` and `{ 0 } ** 2`.
- `specs/compiler/zig_field_syntax.t27` is the regression spec; it replaces the hand-written Rust test `bootstrap/tests/field_syntax_zig.rs`.
- `bootstrap/src/compiler.rs` is hand-written Rust, landed under the owner exception from #6579 (label `owner-approved-foreign`); the debt is #5980.
- Reference pass count on the t27c lab, measured on the same base: 768 -> 770 (specs/fpga/linker.t27 and the new spec go to pass), 0 regressions.
