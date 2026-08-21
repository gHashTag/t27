# NOW -- weight_prefetch_ctrl retires prefetch_done on entry to IDLE (2026-08-21)

## fix(bitnet): clear prefetch_done in IDLE, not only inside the start guard (Closes #1985)

- **The defect was live on master verbatim.** #1985 reports itself as fixed, but its fixes
  landed on a branch that never merged. `bootstrap/src/bitnet_buffers.rs:181-182` on
  `origin/master` (4ea72c322) still emitted
  `IDLE: if (start_prefetch) begin` / `state <= FETCH; prefetch_active <= 1'b1; prefetch_done <= 1'b0;`
  -- the clear sat *inside* the guard, so `DONE_ST` raised the flag and nothing lowered it
  until a new request had already been sampled
- **Why that is one cycle too late.** The clear is a non-blocking assignment, so it takes
  effect the cycle *after* `start_prefetch` is seen. `multilayer_sequencer` does
  `PREFETCH: begin start_prefetch<=1'b1; state<=WAIT_PF; end` then
  `WAIT_PF: if(prefetch_done) state<=RUN;` -- it tests the flag in the same cycle
  `start_prefetch` is high, which is exactly the cycle the stale `1` is still there
- Fix: emit `IDLE: begin prefetch_done <= 1'b0; if (start_prefetch) begin ... end end`.
  The flag is retired on entry to IDLE, so it is genuinely the one-cycle pulse the module
  doc-comment already claimed it was. No other signal, state or port changed
- **Mutant proof, unit level.** New test `prefetch_done_retired_in_idle_before_start_guard`
  slices the `IDLE` case arm out of the emitted text and requires the clear to precede the
  guard. Planting the mutant (clear moved back inside the guard) fails it with
  `prefetch_done must be cleared on entry to IDLE, before the `if (start_prefetch)` guard`,
  printing the offending IDLE arm. Reverting: 23 passed, 0 failed
- **The assertion is anchored to the IDLE arm deliberately.** The reset block also contains
  `prefetch_done <= 1'b0;`, so a plain `contains` check returns `true` on the *defective*
  emitter -- measured. An unanchored guard here would have been vacuous
- **Mutant proof, RTL level (icarus).** Two transactions, sampling `prefetch_done` in the
  cycle the second `start_prefetch` is raised: defective emitter reads
  `sampled_done_t2=1`, fixed reads `0`, with controls `done_rises=2` and `we_count=4`
  unchanged in both
- **End-to-end, real `multilayer_sequencer` + real `weight_prefetch_ctrl`, two layers.**
  Defective: `overlap_cycles=1 layer_start_during_prefetch=1` -- layer 1 starts computing
  while its own weights are still being written into the weight BRAM, which is the overlap
  in the issue title. Fixed: both `0`
- **The defect shipped under a green check.** Master's own 22 unit tests all pass on the
  defective emitter. No workflow runs `cargo test -p t27c` -- `corpus-ratchet.yml` records
  that the step was removed by #2292 after going red on master (1602 passed / 13 failed).
  The new guards are therefore proved locally and are not executed by CI; that gap is
  pre-existing and is recorded here rather than papered over
- **Only defect one is fixed.** #1985 also reports a missing request/acknowledge in
  `multilayer_sequencer` (`bootstrap/src/bitnet_pipeline.rs`), a different module. The
  measurement above shows this fix alone closes the observable overlap for that consumer,
  but the level-triggered handshake is still edge-insensitive by construction. Filed
  separately rather than bundled into this diff
