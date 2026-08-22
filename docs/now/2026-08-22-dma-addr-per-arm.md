# NOW — a stale test counted a literal the emitter stopped writing

Last updated: 2026-08-22

## Assert the address advance per state arm (Closes #2384)

- Branch: `fix/2384-dma-addr-per-arm`
- Issue: #2384

### Что легло

`bootstrap/tests/bitnet_dma.rs` — `dma_local_addr_autoincrement_both_paths` becomes
`dma_local_addr_advances_on_both_paths`, asserting each path's mechanism inside its own
state arm instead of counting one literal across the module. Plus one line pruned from
`scripts/ci/test-baseline.txt` (383 → 382), since the test now passes.

The test had been red on master since #2345, which replaced the read path's post-increment
with `local_addr <= beat_index` — correctly, that was the #2003 fix — and did not update
the test that counted the old literal. **The emitter is right; the test was stale.**

The property is unchanged and still holds: each path advances the destination address.
`READ_DATA` presents `beat_index` and advances it; `WRITE_DATA` post-increments
`local_addr`, which is a read pointer there and must not be harmonised with the read path.

### Границы честности (BINDING)

- **Not a fix to any emitted RTL.** No Verilog changes. This corrects a test that was
  measuring surface form instead of the property it names.
- Lowering the threshold from `>= 2` to `>= 1` would also have gone green and would have
  re-opened the same blind spot — a global count cannot say which path a match came from.
- The failure was invisible because `cargo test` stops at the first failing target and this
  is the 42nd of 73 (#2382). The ratchet landed in #2383 is what will surface the next one.
- **This is one of the 12 unexamined failures from #2382. Eleven remain**, and their ages
  are still unestablished.

### Evidence

Three mutants, one per assertion, each verified planted (`planted=1`) before the run and
each caught by exactly its own assertion:

- `bitnet_dma.rs:217` removed → `READ_DATA must present the beat index as the address…`
- `:218` removed → `READ_DATA must advance beat_index, or every beat writes address 0.`
- `:235` removed → `WRITE_DATA must advance local_addr, or every beat reads the same word.`

`21 passed; 1 failed` each time; restored, `22 passed; 0 failed`.
