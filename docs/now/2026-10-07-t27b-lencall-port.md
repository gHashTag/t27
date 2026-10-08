# NOW -- t27b: the `.len` call decision in t27 (2026-10-07)

## Which calls lower as the `.len` field (Closes #7524)

- Port slice of #6198. `specs/tri/t27b/lencall.t27` decides which call t27b lowers as the length field: a `len` method call with one child, a no-argument dotted path ending in `.len`, or a free `len(x)` when no fn `len` is declared. Its 6 tests take their values from the Rust it replaces; t27c test-report passes 6 of 6, 0 vacuous.
- `gen/rust/tri/t27b/lencall.rs` (99 lines, t27c gen-rust) is mounted in `cli/t27b/src/lower.rs` as `mod lencall`. `cli/t27b/src/lower/lencall.rs` (56 lines) is deleted; `lower.rs` gains 28 lines of glue that build the nodes.
- t27b lab, head vs master bd3bdcda1 on the same 1552 files: pass 844 / 844, blocked 535 / 535, mismatch 0 / 0, no per-file verdict change. `t27b asm` prints the same bytes on all 596 specs that mention `len`. `cargo test -p t27b` passes.
- A slice parameter lowers to an owned `Vec<u8>` (#7449), so the glue copies the call name once per call.

Refs #6198 #6063 #5980.
