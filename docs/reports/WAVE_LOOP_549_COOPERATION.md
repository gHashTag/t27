# Wave Loop 549 → 550 — three cooperation variants

**Date:** 2026-08-09 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Source:** [`WAVE_LOOP_549_RESEARCH.md`](WAVE_LOOP_549_RESEARCH.md) · [`WAVE_LOOP_549_REPORT.md`](WAVE_LOOP_549_REPORT.md)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

> **These variants replace the set drafted earlier in the wave.** That set was
> written before the corpus finding: it proposed hardware bring-up, a vacuity
> gate, and a VerilogEval score. All three are now second-order. You cannot
> score a code model whose corpus does not compile, and gating vacuity would
> only stop the loop from writing the one thing it *can* write about specs it
> cannot build. Priorities below follow the evidence, not the earlier plan.

Each variant states a hypothesis, deliverables, a validation contract, and what
would falsify it. Pick exactly one.

---

## Variant A (recommended) — Close the two syntax gaps; make 69k lines real

**Hypothesis.** The IGLA corpus is two parser productions away from compiling.
`if`-expressions and floats are both already implemented; what is missing is a
brace-delimited block-expression and two entries in a cast whitelist. Closing
that gap converts 69,000 lines of dead specification into a corpus the backend
can actually consume — and it is the precondition for every other IGLA claim,
including the FPGA work and any benchmark score.

**Order matters.** Step 1 is not optional: the build is currently broken for
anyone who touches `compiler.rs`.

**Deliverables.**
1. **Unblock the build.** Six committed documents violate L3 / LANG-EN and are
   not allowlisted, so `bootstrap/build.rs` panics whenever it re-runs
   (§4.4 of the research report). Either translate them or add them to
   `docs/.legacy-non-english-docs` — **the allowlist requires Architect
   approval, so this is a human decision, not an agent one.**
2. Apply [`docs/patches/W550-f32-cast-whitelist.md`](../patches/W550-f32-cast-whitelist.md)
   — add `f32`/`f64` to `VALID_CAST_TYPES`. Reseal `FROZEN_HASH`.
3. Add a **block-expression** production: `{ stmts…; tail_expr }` valid in
   expression position, so `if (c) { a } else { b }` parses. This is the
   12-spec class and the real work of the variant.
4. Re-measure with `t27c synth-gate --specs-dir specs/igla/race` and record the
   new gen/synth rates against this wave's 8/17 and 7/17 baseline.

**Validation contract.**
- `cargo build --release -p t27c` green *after* a `compiler.rs` edit — which is
  the condition that currently fails.
- IGLA gen-verilog rises from 8/27; every remaining failure has a named class.
- `t27c suite` no worse than the pre-wave baseline.
- The three formal theorems still pass (`fpga/formal/`).

**What would falsify it.** If block-expressions turn out to conflict with the
existing statement-`if` grammar in a way that needs a larger redesign, the
variant's premise ("two productions") is wrong and the honest answer is
route 2 — rewrite the specs. Decide that from a real grammar conflict, not
from difficulty.

---

## Variant B — Close the hardware loop

**Hypothesis.** The ternary MAC is now the best-evidenced artifact in the
project: T1 proves it computes exact integer MAC for all inputs, T2 measures it
at zero DSP48 against one DSP48E1 for the equivalent multiplier design, and T3
proves the on-board LED signature is a falsifiable prediction. The only thing
missing is an observation on silicon.

**Why it is not A.** It is independent of the corpus finding — the MAC RTL is
hand-written and already synthesizes — so it can run in parallel. It is second
only because A unblocks everything else.

**Deliverables.**
1. Gate G1 of [`IGLA_FPGA_LAUNCH_PLAN.md`](../fpga/IGLA_FPGA_LAUNCH_PLAN.md):
   produce `ternary_mac_demo_top_v2_200t.bit` via Docker openXC7. `-abc9`
   mandatory, `-nocarry` always.
2. Record routed resources and slack against the `cfgmclk` constraint — the
   first real timing number for any IGLA RACE design.
3. Gates G2/G3 once the board is attached:
   `t27c fpga-flash --board wukong-a200t --mode sram`.
4. Commit the observed LED signature (photo or logic-analyzer capture) as the
   witness, and state whether it matches T3.

**Validation contract.**
- nextpnr reports routing complete, no unrouted nets.
- `t27c fpga-flash --dry-run` reports `READY` rather than `BLOCKED`.
- Observed: `led_r23` blinking ≈1 Hz, `led_t23` dark.

**What would falsify it.** A lit `led_t23` contradicts T3. That is the most
valuable outcome available in this variant — it would mean silicon disagrees
with a machine-checked model, and finding out why is worth more than a clean
pass.

**Blocked by.** `nextpnr-xilinx` not installed; no board attached
(`openFPGALoader --scan-usb` → "No USB devices found").

---

## Variant C (fallback) — Stop the loop from lying to itself

**Hypothesis.** Three independent silent-failure modes combined to hide a total
corpus failure for hundreds of iterations: a parser that dropped malformed
input, a readiness metric that never invoked a synthesizer, and an appender
that only ever asserted `true`. Two are now fixed or measurable. The remaining
work is to make the loop unable to repeat this.

**Why it is the fallback.** It adds no capability and proves nothing new. Take
it only if A is blocked on the Architect decision and B on hardware.

**Deliverables.**
1. Wire `t27c synth-gate` into CI for `specs/igla/**` with
   `--min-pass-rate` pinned at the current measured rate, so the number can
   only go up. Retire `synth-readiness` from any readiness claim.
2. Wire `t27c validate-vacuity` as a **reporting** gate (no failure threshold
   yet — see the note at the top of this file).
3. Fix the appender itself: whatever emits `test <name>_wNNN_batch_depth_
   invariant_2 { … }` produced an unterminated block in W339 and replicated it
   to 27 specs. Find it and make it emit balanced braces, or delete it.
4. Fix the Verilog reserved-word bug found in §3: identifiers like `input` are
   escaped in declarations but emitted bare at use sites. The Zig backend
   already solved this exact problem with `@"name"` at every value-identifier
   site; port the approach.
5. Merge the two colliding wave-loop counters (see the numbering note in
   [`WAVE_LOOP_549_PLAN.md`](WAVE_LOOP_549_PLAN.md)).

**Validation contract.**
- `synth-gate` reproduces 8/17 and 7/17 on the current tree.
- `validate-vacuity` reproduces 2160/3788 and 1917/3314.
- `ternary_inference.t27` synthesizes after the reserved-word fix.

**What would falsify it.** If the appender turns out to be a human editing
pattern rather than a script, there is nothing to fix in code and the remedy is
the skill charter alone — already updated this wave.

---

## Recommendation

**Variant A.** Everything the IGLA line claims rests on a corpus that does not
compile, and this wave established the distance to fixing that is two syntax
productions. B is genuinely parallel and worth starting the moment a board and
`nextpnr-xilinx` are available. C is hygiene that A and B will both benefit
from but neither needs first.

**One item needs a human before any variant proceeds:** the LANG-EN allowlist
decision in A1. The build is broken for compiler work until it is made.

---

*φ² + φ⁻² = 3 | TRINITY*
