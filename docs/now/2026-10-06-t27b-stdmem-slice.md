# NOW -- t27b: std.mem.eql and std.mem.indexOf on bytes in a buffer (2026-10-06)

## byte slices and byte-array pointers compare as strings (Closes #6961)

- New conformance spec `specs/tri/t27b/conformance/std_mem_byte_slice.t27`. It gives 6 pass under `t27c test-report` (none vacuous, 21 runtime asserts), and the same 6 under t27b.
- In the reference, `buf[a:b]` of a `var buf: [N]u8` is a `[]u8` and `&buf` is a `*[N]u8`. Zig coerces both to `[]const u8` where `std.mem.eql(u8, x, y)` or `std.mem.indexOf(u8, x, y)` wants one, so they compare by content. t27b refused them as `ExprCall(std.*)` ("not a string"); it now coerces exactly these two shapes to a string before the call.
- A slice of, or a pointer to, any other element type stays refused as `ExprCall(std.*)` ("not a string").
- Code is in `cli/t27b/src/lower/stdmem.rs`, tests in `cli/t27b/tests/stdmem.rs`. Rust is edited under the owner's `owner-approved-foreign` approval on #6063 and #6961; the two files are listed in `tools/policy/foreign-exceptions.txt`.
- Lab, same tree and same reference before and after: `specs/port/browseros/trios/agent-server/apps/server/src/tools/filesystem/ls.t27` moves from blocked to pass (7 tests, 26 asserts), the new spec passes, and per-test mismatch stays 0.
- `specs/port/scripts/gen_w384_lean.t27` gets past `ExprCall(std.*)` and is now refused one step later as `codegen`: `FnDecl(frame size)`, 16448 bytes of local aggregates in one frame where the limit is 16384. It stays not-pass; its ledger entry records the new blocker.
- The ledger `docs/reports/t27b_expectations.json` is master's ledger plus this PR's own three moves: `ls.t27` blocked to pass, `gen_w384_lean.t27` blocked to codegen, and `std_mem_byte_slice.t27` enters as pass. 0 entries move down. The cap goes from 55 to 54, with no `--accept-new`.
