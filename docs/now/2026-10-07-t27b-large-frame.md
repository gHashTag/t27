# NOW -- t27b runs frames over 16 KiB, probing the stack one page at a time (2026-10-07)

## cli/t27b/src/codegen.rs, specs/tri/t27b/conformance/large_frame.t27 (Closes #7367)

- Lane 5 of #6063. Until now t27b refused every fn or test whose local aggregates take more than 16384 bytes, as `FnDecl(frame size)`. The prologue lowers `sp` in one step, and a single step larger than the guard page could jump over that page without faulting.
- A frame over 16 KiB is now allocated in 4096-byte steps, and each step stores to the new `sp` before the next one (stack probing, the same idea as GCC's `-fstack-clash-protection` on AArch64). The remainder below the last whole page is probed too. Frames up to 16 KiB keep their one-step prologue. The cap rises to 1 MiB, and a larger frame is still refused as `FnDecl(frame size)`.
- New conformance spec `specs/tri/t27b/conformance/large_frame.t27`, written first. The reference (`t27c test-report`) passes it 4 of 4, with 10 runtime asserts and 0 vacuous. It covers:
  - a test body with a 20000-byte array that calls a 16 KiB fn before it checks its own bytes;
  - a fn with 48000 bytes, called twice;
  - 16384 + 64 bytes, the shape of `gen_w384_lean`'s test;
  - a recursion 12 frames deep at 20000 bytes each, where every level checks all of its bytes after the deeper levels return, so overlapping frames show.
- Mutation check on the Railway t27b lab. Each of 3 codegen mutants fails 2 of the 4 conformance tests and crashes `gen_w384_lean`. The mutants are: one page short, the probe loop branching to the store instead of the `sub`, and the remainder never allocated. 5 mutants of the spec's expected values each fail one test.
- `cli/t27b/tests/source.rs` `large_frames_are_probed`: the interpreter and the JIT agree on a 16448-byte test and a 20000-byte-per-frame recursion. A trap inside a 70000-byte frame names its line, and a 1048600-byte frame is refused.
- Ledger: `specs/port/scripts/gen_w384_lean.t27` moves from `codegen` to `pass` (12 of 12, 47 runtime asserts). `large_frame.t27` gets a `pass` row. `max_not_pass` goes from 25 to 24. On the t27b lab, the full corpus run (1526 files) against the ledger shows 0 UNEXPECTED FAILURE.
