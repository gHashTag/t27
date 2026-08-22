# NOW — the marker changed, the hazard did not

Last updated: 2026-08-22

## Assert guard bands instead of translate_off markers (Closes #2395)

- Branch: `fix/2395-translate-guard-bands`
- Issue: #2395 · part of #2386

### Что легло

`bootstrap/tests/verilog_translate_off.rs`, plus two lines pruned from
`scripts/ci/test-baseline.txt` (375 → 373).

Both tests counted standalone `// synthesis translate_off` lines and got 0, for two
independent reasons the tests never followed: W458 replaced the markers with standard
```ifndef SIMULATION``` guards, and `gen-verilog` stopped lowering bench blocks at all
(#2391).

The hazard R-TR-1 names is unchanged — yosys treats `translate_off` as a line-range skip,
so a marker sharing the `initial begin :NAME` line swallows the matching `end`. A
standalone ```ifndef``` cannot, which is why W458 moved to it. So "standalone" is still
the property; only the token changed.

### Границы честности (BINDING)

- **No emitter change.** Fourth stale test today after #2384, #2389 and #2391.
- **This is 2 of the 7 in #2386. Two remain**, and whether any of the seven ever passed is
  still unestablished.
- **Two over-strict drafts were caught by running the check, not by reading it**, and both
  are recorded in the helper's doc comment:
  - requiring a guard on **every** `initial begin` failed on `uart.t27`'s `uart_state`
    initialiser — anonymous `initial` is register power-on init, synthesizable, and must
    **not** be guarded or the reset state leaves the bitstream;
  - requiring the guard on the **immediately preceding line** failed on
    `uart_initially_idle_test`, because one band wraps `$dumpfile` and several test
    blocks together.

  Either would have made a green gate red on correct code. This is the anti-vacuity
  discipline running in the other direction: a check must also be shown **not** to fire
  where nothing is wrong.

### Evidence

Mutant at `bootstrap/src/compiler.rs:12119` (the per-bench guard band), verified planted,
with a local `FROZEN_HASH` reseal:

```
R-TR-1 regression in gen-verilog-for-simulation: line 47:
  `initial begin : probe_latency_a_bench` is outside any `ifndef SIMULATION band
R-TR-1 regression on real uart.t27 via gen-verilog-for-simulation: line 476:
  `initial begin : uart_tx_ready_latency_bench` is outside any `ifndef SIMULATION band
```

Both restored; `2 passed; 0 failed`. Neither `compiler.rs` nor `FROZEN_HASH` is in this diff.
