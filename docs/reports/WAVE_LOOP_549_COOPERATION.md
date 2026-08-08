# Wave Loop 549 → 550 — three cooperation variants

**Date:** 2026-08-09
**Source wave:** [`WAVE_LOOP_549_PLAN.md`](WAVE_LOOP_549_PLAN.md) / [`WAVE_LOOP_549_REPORT.md`](WAVE_LOOP_549_REPORT.md)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Each variant states a hypothesis, the deliverables, the validation contract,
and — importantly — **what would falsify it**. Pick exactly one.

---

## Variant A (recommended) — Close the hardware loop

**Hypothesis.** IGLA RACE's entire hardware claim rests on a MAC cell that has
never been observed running. W549 removed every *software* obstacle to
changing that: `t27c fpga-flash` exists, and `ternary_mac_demo_top_v2` is
simulated, synthesized and constrained. Two obstacles remain, and neither is
research — one is a package install, one is a cable. Clearing them converts
"projected TOPS/W" into "we watched it accumulate", which is the single
largest credibility upgrade available to the project.

**Why now.** Everything upstream is green. Deferring again means W549's v2
demo joins v1 as another design that was never checked on silicon.

**Deliverables.**
1. Produce `fpga/verilog/ternary_mac_demo_top_v2_200t.bit` via the Docker
   openXC7 path in [`IGLA_FPGA_LAUNCH_PLAN.md`](../fpga/IGLA_FPGA_LAUNCH_PLAN.md)
   gate G1. `-abc9` is mandatory; `-nocarry` always.
2. Record the routed resource and slack report against the `cfgmclk`
   constraint — the first real timing number for any IGLA RACE design.
3. Fix **W9**: `t27c fpga-build --device` defaults to `xc7a100tcsg324-1`, the
   Arty A7 package, which `HARDWARE_SSOT.md` forbids in Wukong flows. Default
   it to `xc7a200tfgg676-1` and make the board an explicit choice.
4. When the board is attached: gates G2 and G3, and commit the observed LED
   signature (photograph or logic-analyzer capture) as the witness.

**Validation contract.**
- `nextpnr-xilinx` reports routing complete, no unrouted nets.
- `t27c fpga-flash --board wukong-a200t --dry-run` reports `READY` (not
  `BLOCKED`) once the cable is attached.
- Observed: `led_r23` blinking at ≈1 Hz, `led_t23` dark.

**What would falsify it.** If `led_r23` is steady, the accumulator is not
accumulating on silicon even though it does in simulation — which would be a
genuine and valuable finding about the openXC7 flow, not a failure of the wave.

**Blocked by.** `nextpnr-xilinx` absent locally (installable); no board
attached (`openFPGALoader --scan-usb` → "No USB devices found").

---

## Variant B — Kill the vacuity generator

**Hypothesis.** The 2,160 `assert true` blocks under `specs/igla/**` are not
an accident; they are the *output of the wave loop itself*, which appends the
same 2 tests + 1 tautological invariant to every IGLA spec each iteration.
Retrofitting real tests without stopping the generator is bailing a boat with
the hole open. Stop the generator first, then drain.

**Why it matters.** L4 TESTABILITY is currently satisfiable by writing
`assert true`. Any external reviewer who samples the IGLA specs will find that
in under a minute, and it discredits the honest testing elsewhere in the repo.
It also directly undercuts IGLA CODER's only real differentiator
(see [`COMPETITORS.md`](../../COMPETITORS.md) §4.1): "generated code a
validator can reject" is hollow when the validator accepts `assert true`.

**Deliverables.**
1. `t27c validate-vacuity` — reports, per spec, the fraction of `test`/`bench`
   blocks whose entire body is `assert true` and the fraction of invariants
   that are the literal `true`.
2. Wire it into `t27c suite` as a **reporting** gate first (no failure), so
   the baseline is agreed before anyone is blocked.
3. Amend the wave-loop skill and `t27-wave-loop.md` charter so appending
   `assert true` witnesses is explicitly forbidden — a wave that has nothing
   real to assert should add nothing.
4. Convert the worst offender (`specs/igla/race/ternary_inference.t27`, 80/140
   vacuous and **0 real benches**) into genuine assertions derived from
   `ternary_mac_synth.v` behaviour, as the worked example.

**Validation contract.**
- `t27c validate-vacuity` reproduces 2160/3788 and 1917/1931 on the current
  tree (the numbers in this wave's report).
- After step 4, `ternary_inference.t27` vacuity drops below 10 %.
- `t27c suite` shows no regression.

**What would falsify it.** If the appended tests turn out to be load-bearing
for some seal or hash-stability check, removing them breaks seals — in which
case the finding is that seal stability was being proxied by fake tests, which
is worth knowing before touching anything.

---

## Variant C (fallback) — Get a number on the board that everyone else uses

**Hypothesis.** IGLA CODER cannot be compared to
[RTL-Coder](https://github.com/hkust-zhiyao/RTL-Coder) or any commercial code
model because it has never been scored on
[VerilogEval](https://github.com/NVlabs/verilog-eval). A bad published score is
strictly more useful than no score: it establishes a baseline, makes progress
measurable, and removes the suspicion that the benchmark was avoided.

**Why it is the fallback.** It costs the most external setup (harness, model
weights, evaluation compute) and produces the least hardware truth. Take it
only if A is blocked on hardware and B is blocked on seal risk.

**Deliverables.**
1. Wire `specs/igla/evaluation/multi_lang_harness.t27` (currently 110 lines,
   a stub) to the actual NVlabs `verilog-eval` harness.
2. Report `pass@1` for whatever IGLA CODER checkpoint exists, next to the
   published RTL-Coder and GPT-3.5 baselines, in `BENCHMARKS.md`.
3. If no checkpoint is trainable in-wave, publish the harness plus a documented
   `N/A — no checkpoint` row rather than silence.
4. Housekeeping: merge the two colliding wave-loop counters (see the numbering
   note in [`WAVE_LOOP_549_PLAN.md`](WAVE_LOOP_549_PLAN.md)) into one sequence.

**Validation contract.**
- The harness runs end-to-end on at least one reference model, reproducing a
  published baseline within noise — proving the harness itself is correct
  before any IGLA number is trusted.
- `BENCHMARKS.md` gains a row that cites the harness commit and the date.

**What would falsify it.** If the harness cannot reproduce a published
baseline, no IGLA score from it means anything, and the wave's output is the
negative result plus the reason.

---

## Recommendation

**Variant A.** W549 spent its effort removing the software excuses for never
having run on hardware; the honest next step is to run on hardware. B is the
right choice only if the board genuinely cannot be attached this wave — in
which case B is the highest-value work that needs no silicon at all.

---

*φ² + φ⁻² = 3 | TRINITY*
