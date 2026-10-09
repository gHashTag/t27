# NOW -- t27b frame-address families: where the refusal stops (2026-10-09)

## a conformance spec for caller-owned slices and cells (Closes #8053)

- `ExprReturn(frame address)` and `StmtAssign(frame address)` led `tri t27b next` on lab run
  7a07828fc, two specs each. The refusals are right (#7550, #7735); #8047 rewrites the two port
  specs behind them (tool-registry.t27, gen_lockfree_stack.t27) so the caller owns the memory.
- New `specs/tri/t27b/conformance/frame_address_caller_owned.t27` pins where the refusal stops:
  a slice of a slice parameter, and a pointer parameter stored through another pointer parameter,
  survive a call that fills 4 KiB of its own stack with 0x55. Sealed on the t27c lab.
- Reference 2/2 (11 runtime asserts, 0 vacuous); t27b pass with 11 runtime asserts, reference
  disagree 0, mismatch 0. 4 of 4 mutants killed by both. Ledger: one new pass row, cap unchanged.
- The two conformance specs that pin the refusal stay blocked, reference-bug, by design: with
  #8047 merged, each family blocks only its own refusal spec.
