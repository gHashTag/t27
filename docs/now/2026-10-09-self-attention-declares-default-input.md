# NOW -- self_attention.t27 declares default_input, the same verdict on every host (2026-10-09)

## specs/ml/recurrent/self_attention.t27 (Closes #8051, Refs #6063)

- The test called an undeclared `default_input()`. t27c's W585 scaffold printed `const input = undefined;`
  and `forward()` looped over that slice, so the reference passed on a Mac and failed on the Railway lab.
- `default_input(buf)` now writes 1.0, 2.0, ..., buf.len into a buffer the test owns and returns it. It
  takes the buffer on purpose: a no-argument `default_input()` bound by `given` is still lowered to
  `undefined` without calling the helper (#7733).
- `forward()` returns the sum it computed. The test is a block test with 7 runtime asserts: the length,
  the first and last components, and the sum 10.0. Changing the sum bound to 10.99 makes it fail.
- t27c test-report on the lab: 1 FAIL before, 1 pass (7 asserts) after. t27b: StmtAssign(undeclared)
  before, pass (7 asserts, mismatch 0) after. The ledger gets a `pass` row; max_not_pass stays 27.
