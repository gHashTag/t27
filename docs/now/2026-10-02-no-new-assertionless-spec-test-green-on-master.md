# NOW -- No new assertionless spec test, green on master (2026-10-02)

## Remove three tests that cannot fail (Refs #5497)

- Corpus Ratchet failed its first step on master: three files gained a test that cannot fail. `specs/igla/coder/pipeline.t27` added one whose body is `assert true` (#4796); `specs/port/fpga/verilog/ternary_mac_synth.t27` (`on_clock_reset`) and `specs/port/fpga/vivado/gf16_matmul_top.t27` (`led_r23_toggles`) added ones whose bodies are only comments.
- None of the three can be given an assertion that runs: `t27c test-report` reports all three specs BLOCKED in Zig, so no test in them executes. Remove the three bodies, as the gate asks. The pilot measurement in pipeline.t27 stays as written, with its invariant line.
- Lower the ledger by the one row that was already slack: `specs/igla/race/cordic.t27` 144 -> 0. Total 4049 -> 3905.
- Reseal `coder_igla-coder-pipeline.json`, stale on master since #4796 changed the spec; `t27c seal --verify` now reports all hashes MATCH. All four backends generate the three specs before and after.
