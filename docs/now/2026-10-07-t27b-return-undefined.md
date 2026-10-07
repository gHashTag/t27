# NOW -- t27b: `return undefined;` in a void fn is a plain return (2026-10-07)

## ExprReturn (Closes #7402)

- Ported specs end a `-> void` fn with `return undefined;` (a deinit that frees nothing, an early exit). In Zig `undefined` coerced to `void` is the one void value, so the statement is a plain `return;`. t27b now lowers it that way. Inside a test block it stays refused: there the Zig result is an error union, and an undefined one is no value t27b can name.
- Conformance spec first: `specs/tri/t27b/conformance/return_undefined_void.t27`, 4 tests. The reference gives 4 pass, 0 vacuous; t27b gives the same 4.
- 5 mutants of the spec each fail the same tests in t27b and in the reference. Of 10 probes, 5 pass in both, 1 fails in both, 1 is blocked in both, 1 (in a test block) is an honest t27b refusal, and 1 (a non-void fn returning `undefined`) is refused where the reference passes 0 tests.
- Disclosed gap: a statement after `return undefined;` is unreachable code, which the reference refuses to compile; t27b runs the file. t27b does not check unreachable code in general; this is not new to this family.
- Lab corpus against master 05e633d03: mismatch 0, crash 0. t27b passes 827 reference-pass files, up from 823. Ledger: `gen_ring_buffer.t27`, `gen_rtree.t27`, `gen_wal.t27` and `return_undefined_void.t27` are added as pass; `gen_work_stealing.t27` gets past `ExprReturn` and now stops at codegen `FnDecl(frame size)` (16448 bytes of local aggregates, the limit is 16384).
- Code is in `cli/t27b/src/lower.rs`, listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.

Refs #6063. Closes #7402.
