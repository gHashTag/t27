# NOW — the counters were never lost, the test asked the wrong backend

Last updated: 2026-08-22

## Point the R-VD-1 bench tests at gen-verilog-for-simulation (Closes #2391)

- Branch: `fix/2391-bench-counters`
- Issue: #2391 · part of #2386

### Что легло

`bootstrap/tests/verilog_initial_decl.rs`, plus two lines pruned from
`scripts/ci/test-baseline.txt` (377 → 375).

Both tests asked `gen-verilog` for the hoisted `integer _bench_<name>_cycles = 0;`
counters and got `[]`. `gen-verilog` stopped lowering bench blocks — it emits
synthesizable RTL, and a bench is a simulation construct — writing them as comments under
a header that names the successor. `gen-verilog-for-simulation` emits exactly what the
tests want, hoisted to module scope as R-VD-1 requires.

R-VD-1 is a **rule** about emitted Verilog, so it is now asserted on **both** backends. The
counters are a **mechanism** that exists in one, so they are asserted there. Added the
converse as a tripwire: `gen-verilog` must not carry the counters, since their return to
synthesizable RTL would mean an `initial` block came back.

### Границы честности (BINDING)

- **No emitter change.** Third stale test today after #2384 and #2389; the emitter was
  right each time.
- **Biting was proved for the counter assertions only.** Un-hoisting the counters in
  `compiler.rs:12104` failed both tests with their own messages. The new
  "RTL must not carry counters" tripwire and the R-VD-1 rule itself were **not** given
  mutants — the tripwire is trivially satisfied today, and its value is prospective. Said
  plainly rather than implied.
- **This is 2 of the 7 in #2386. Four remain**, and whether any of the seven ever passed is
  still unestablished.

### Evidence

Mutant at `bootstrap/src/compiler.rs:12104`, verified planted, requiring a local
`bootstrap/stage0/FROZEN_HASH` reseal (editing `compiler.rs` trips the freeze gate in
`build.rs:235`):

```
Expected exactly 2 module-scope `_bench_<name>_cycles` counters from
gen-verilog-for-simulation (one per bench in the synthetic spec), got [].
Expected at least 3 ... on real uart.t27 (3 benches), got 0 ([]).
```

Both restored; `2 passed; 0 failed`. Neither `compiler.rs` nor `FROZEN_HASH` is in this diff.
