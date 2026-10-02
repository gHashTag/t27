# NOW -- Drop 80 placeholder tests from the coder pipeline spec (2026-10-02)

## Tests that assert nothing are removed and the ratchets follow (Refs #5472)

- `specs/igla/coder/pipeline.t27` carried 80 tests whose whole body was `assert true` (written as `{ /* verify baseline */ }` placeholders). They executed nothing and counted as tests. They are removed; the file keeps its 196 real test blocks. `specs/port/fpga/vivado/gf16_matmul_top.t27` loses two comment-only tests the same way.
- The pipeline seal is made again with the t27c built from this tree (`coder_igla-coder-pipeline.json`, all hashes MATCH). `gf16_matmul_top.t27` has no seal on master and none is added here.
- The ratchets move down in the same commit: the corpus ledger drops the `gf16_matmul_top` entry, which no longer discards (134 / 134, `RATCHET: CLEAN`); the assertionless baseline 3905 -> 3761; the published test-block count 14330 -> 14314. Master measured 14394 at 4c597ec7, 64 above its pin, because port PRs do not re-pin.
