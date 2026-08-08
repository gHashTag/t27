# Machine-checked theorems for the IGLA RACE ternary MAC

**Wave:** 549 · **Date:** 2026-08-09 · **Tool:** yosys 0.63 (minisat backend)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Three theorems about `fpga/verilog/ternary_mac_synth.v` and the on-board demo
built on it. All three are **machine-checked on this host with yosys alone** —
no Coq, no Lean, no SymbiYosys, no vendor tools. Each is reproducible with one
command and exits non-zero if it fails, so they are usable as CI gates.

> Why this matters. The project's TOPS/W figures are projections from models.
> These theorems are not projections: they are proofs about the RTL that is
> actually synthesized, and T2 is a measurement. They are the smallest honest
> claims available, which is exactly why they are worth making.

---

## T1 — Functional correctness: the multiplier-free MAC *is* integer MAC

**Statement.** For **all** 8-bit signed activations `a`, **all** 2-bit weight
codes `w_code`, and **all** 32-bit signed accumulator inputs `acc_in`, the
shipped `ternary_mac_top` produces exactly the same `acc_out` as
`ternary_mac_golden` — a reference model written with a real `*` operator and
a signed 2-bit weight.

**Method.** A sequential miter over the two designs, unrolled two clocked
transitions from a common zero initial state, handed to SAT: *find any input
assignment where the outputs differ.* No such assignment exists.

```bash
yosys -s fpga/formal/prove_ternary_mac.ys
```

| | |
|---|---|
| Problem size | 6,635 variables, 18,490 clauses |
| Verdict | `SAT proof finished - no model found: SUCCESS!` |
| Exit code | 0 |

**Why it is not circular.** The two implementations are structurally
different, and yosys says so: the miter netlist contains one `$mul` (from the
golden model) and one `$neg` (from the shipped conditional-negate path). The
proof relates a multiply to a sign-flip, not a design to a copy of itself.

**Conclusion.** The ternary datapath is not an approximation of multiply-
accumulate. It is exact, over the entire input space, including both zero
encodings and the `a = −128` sign-extension corner where a naive 8-bit
negation would overflow.

---

## T2 — Structural: exact MAC without a multiplier (R-SI-1, "no star")

**Statement.** Synthesized for Artix-7 with identical settings, the shipped
`ternary_mac_top` contains **zero** DSP48 primitives and zero multiplier
cells; the functionally equivalent golden model consumes **one DSP48E1**.

```bash
yosys -s fpga/formal/prove_no_multiplier.ys
```

| Design | DSP48 | LUTs | FFs |
|--------|-------|------|-----|
| `ternary_mac_top` (shipped) | **0** | 59 (4×LUT2, 12×LUT3, 12×LUT4, 11×LUT5, 20×LUT6) | 32 FDCE |
| `ternary_mac_golden` (real `*`) | **1 × DSP48E1** | — | — |

Both were synthesized with `synth_xilinx -abc9 -nocarry -arch xc7`.

**Conclusion.** Read with T1, this is the quantified ternary advantage on this
cell: *the same arithmetic function, one hard multiplier block cheaper.* On a
device with a finite DSP budget this is the entire argument for ternary
weights in one line — and unlike a TOPS/W projection, it is a count anyone can
reproduce with an open toolchain in under a minute.

**Scope.** One MAC cell. This does not extrapolate to a GEMM array, and
nothing here claims it does.

---

## T3 — The on-board pass criterion is a prediction, not a hope

**Statement.** Once the power-on reset is released, `ternary_mac_demo_core`'s
accumulator is confined to `{0, +1}`. Therefore it is never negative
(**T3a**), the sign LED `led_t23` never lights (**T3b**), and the activity LED
is exactly the "accumulator non-zero" predicate (**T3c**).

**Method.** Properties live inside `fpga/verilog/ternary_mac_demo_core.v`
behind `` `ifdef FORMAL ``. Two independent checks:

1. **Bounded model check** — 64 clocks at `PRESCALE_BITS=2` (16 datapath
   steps: the 3-step reset plus more than three complete `{+1, 0, −1, 0}`
   weight cycles).
2. **Temporal induction** — an *unbounded* proof, so the invariant holds for
   all time rather than merely the checked window.

```bash
yosys -s fpga/formal/prove_demo_core.ys
```

| Check | Problem size | Verdict |
|-------|--------------|---------|
| BMC, `-seq 64` | 92,564 variables, 257,076 clauses | `SAT proof finished - no model found: SUCCESS!` |
| Temporal induction | converged at **length 10** (22,912 variables, 67,464 clauses) | `Induction step proven: SUCCESS!` |

Exit code 0.

**Scope, stated precisely.** The machine-checked instance uses
`PRESCALE_BITS = 2`; the synthesized board design uses 24. The invariant
argument is independent of the prescaler width — the prescaler only decides
*when* a step happens, never *what* the step computes — but the proof was run
at width 2, and that distinction is the honest one. Induction length 10 is the
history the solver needs to exclude unreachable combinations of the reset,
phase and prescaler counters.

**Conclusion — this is the point of the whole exercise.** The launch plan
predicts that a correctly flashed board shows `led_r23` blinking and `led_t23`
dark. T3 turns that from a hope into a **falsifiable prediction**: if a real
board lights `led_t23`, then either the silicon disagrees with the model or
the minus-weight decode is broken, and either way it is a finding rather than
a shrug. The v1 demo could not be falsified by any observation — that is what
made it worthless as evidence.

---

## Relation to the project's other formal work

`coq/IGLA/` and `trios-coq/IGLA/` prove properties of an **abstract opcode
alphabet** — e.g. `holographic_no_star`, that no `holo_op` reduces through a
Kleene star. Those are statements about a model of the ISA.

The theorems here are about **the Verilog that is actually synthesized**. They
are weaker in ambition and stronger in grounding, and the two should not be
conflated when either is cited. Closing the gap — a refinement relation from
the `.t27` spec to the emitted RTL — remains unclaimed; see `COMPETITORS.md`
on why [Vericert](https://github.com/ymherklotz/vericert) is strictly stronger
than t27 on the compiler-correctness axis.

---

## Running all three

```bash
cd fpga/formal && yosys -s prove_ternary_mac.ys && yosys -s prove_no_multiplier.ys && yosys -s prove_demo_core.ys
```

Each script exits non-zero on failure, so the sequence is CI-ready as written.

---

*φ² + φ⁻² = 3 | TRINITY*
