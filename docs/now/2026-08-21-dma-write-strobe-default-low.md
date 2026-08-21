# NOW -- dma_controller defaults local_we low before the case (2026-08-21)

## fix(bitnet): make the DMA local write strobe a pulse, not a level (Closes #2006)

- **The defect was live on master verbatim.** #2006 reports itself as fixed, but its fixes
  landed on a branch that never merged (no PR, in any state). On `origin/master`
  (ee494b990b795267ce1c955d6009819b57ac55e8), `bootstrap/src/bitnet_dma.rs:165` still emitted
  `end else case (state)` with no default for `local_we`. The only clears were
  `:193` (`end else local_we <= 1'b0;`, inside `READ_DATA`) and `:218` (`DONE_ST`)
- Fix: emit `end else begin` / `local_we <= 1'b0;` / `case (state)`, closing with
  `endcase` + `end`. Any state that does not explicitly drive the strobe now leaves it
  low. No port, state, or other signal changed; the case arms are byte-identical
- **Honest scope: this fix is behaviourally latent on today's FSM, and that was measured,
  not assumed.** A differential icarus simulation running the master emitter and the fixed
  emitter side by side in one testbench, over a 4-beat read, reports identical observables:
  both perform 4 local writes to addresses `1,2,3,4`, both leave `mem[0]` unwritten
  (`ffffffffffffffff`), and both show `local_we high outside READ_DATA = 1 cycles`.
  The reason is structural: `local_we` is only ever raised in `READ_DATA`, and the sole
  exit from `READ_DATA` is `DONE_ST`, which clears it. There is no path back to
  `READ_ADDR` in the single-burst FSM, so the missing default is unreachable today
- **It stops being latent under #1970.** The `READ_ADDR`-between-bursts window named in the
  issue only exists once the burst loop is derived. This lands the defensive shape first so
  the multi-burst change cannot silently reintroduce a stuck strobe. Claiming a behavioural
  win here would have been false, so it is not claimed
- **Mutant proof, guard 1 (`local_we_defaults_low_before_the_case`).** Deleting only the
  emitted default line fails it with
  ``` `local_we` must be defaulted low between the reset block and `case (state)`, so the
  strobe is a one-cycle pulse instead of a level that persists through states which never
  drive it (READ_ADDR between bursts, IDLE, WRITE_ADDR, WRITE_DATA). Text found between
  `end else` and `case (state)` was: "end else begin\n            " ```
  -- the message prints the offending span, which is what shows the assertion read the
  right text. 16 passed, 1 failed. Reverting: 17 passed, 0 failed
- **The guard is anchored deliberately, and the anchor is load-bearing -- measured.**
  Adding a naive `v.contains("local_we <= 1'b0;")` to the *defective* master emitter makes
  it report `ok`: `DONE_ST` already emits that exact string with single spacing, and the
  reset block emits a padded variant. The anchored guard on the same defective emitter
  reports `FAILED`. An unanchored check here would have been vacuous, which is the trap
  that had to be repaired hours after landing elsewhere in this campaign
- **Mutant proof, guard 2 (`always_block_begin_end_balanced`), planted separately.**
  One mutant per guard, not one per file: deleting the closing `end` fails it with
  `unbalanced begin/end in emitted module: 15 `begin` vs 14 `end` (excluding
  `endcase`/`endmodule`)` while guard 1 still passes -- so neither guard masks the other
- **The emitted Verilog parses**: `iverilog -g2005` accepts the fixed module
- **These tests are not executed by CI.** No workflow runs `cargo test -p t27c`;
  `corpus-ratchet.yml` records that the step was removed by #2292 after going red on
  master. They were run with `rustc --test` on the module, which is self-contained.
  That gap is pre-existing and is recorded here rather than papered over
