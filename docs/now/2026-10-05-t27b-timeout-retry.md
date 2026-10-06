# NOW -- t27b corpus retries a timed-out file once, alone (2026-10-05)

## A contention timeout no longer costs a pass (Closes #6310)

- On the Railway lab (`--jobs 24`, 60 s timeout under qemu) two consecutive runs each lost one fast file to a timeout: `specs/ml/recurrent/bilstm.t27` (rejected natively in 12 ms) in run e2fb1d87e, and `specs/trinity/capabilities/ops.railway-cli.t27` (passes natively in 3.6 ms, reference passes) in run 0851055e3.
- `t27b corpus` now runs each file that timed out once more, one at a time after the parallel pass, with the same timeout. The retry's verdict is final, so the three genuine infinite loops (axi4_tb, clock_domain_tb, gf16_accel_tb) stay timeouts.
- Honesty: a retried record carries `"retried_after_timeout": true`, `totals.timeout_retried` counts them, the text summary prints a `RETRIED` line per file, and the lab summary gains `timeout_retried`.
- Local `t27b corpus specs --jobs 6`: totals before and after are identical except the new field (pass 296, pass_vacuous 279, timeout 3, mismatch 0); no per-file record changed.
- Test: `retry_timeouts_once` in `cli/t27b/tests/blockers.rs` (times out once then passes, always times out, never retried twice, non-timeouts never re-run). An end-to-end run with a runner that sleeps on the first call per file turns 2 timeouts into 1 pass and 1 blocked; master's binary reports 2 timeouts on the same runner.
- Takes effect on the lab after its next build of master.
