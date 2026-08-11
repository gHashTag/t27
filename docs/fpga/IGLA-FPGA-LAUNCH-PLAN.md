# IGLA on real silicon — the launch plan, with what is proved and what is not

**Date:** 2026-08-10 · **Waves:** W568–W602 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Hardware SSOT:** [`fpga/HARDWARE_SSOT.md`](../../fpga/HARDWARE_SSOT.md) — that file wins on every hardware fact.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 0. The one-line status

```
Everything that can be verified without the board HAS been.
The remaining blocker is a USB cable.
```

That is not a figure of speech. `t27c` has been run against the whole corpus, the
FPGA spec family measures **300 / 300 tests passing**, T1–T3 machine-check the
RTL, and `dlc10 idcode` returns:

```
Error: open DLC10
Caused by: DLC10 cable not found (VID=0x03FD)
```

## 1. What is proved, and to what standard

| | Result | Standard of evidence |
|---|---|---|
| **T1** | The ternary MAC RTL is equivalent to its spec | **machine-checked** — yosys SAT miter, ports bound by name |
| **T2** | The lowering is multiplier-free | **machine-checked** — no `$mul` cell survives `synth_xilinx` |
| **P1** | Timing closes at 150.63 MHz | tool report (nextpnr-xilinx / openXC7), not a proof |
| **P15** | `specs/fpga/` 246 tests, `specs/boards/` 54 tests | **measured**, 0 failures |
| ~~**P12**~~ | ~~No remaining blocker in the RACE kernels is a compiler defect~~ | **REFUTED W623** — four compiler-defect classes; one fixed, 9 sites, spec text unchanged (T18) |

> **Labels in this table name claims in
> [`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md).**
> Until W623 the **T3** row named a different claim than T3 in that document
> (which is the accumulator invariant); the timing sentence is **P1** there and
> is disclaimed as a theorem. Corrected above. Never write a bare label — see
> **T15**.

**What is *not* proved:** that the bitstream on real silicon behaves as the
simulation does. No wave in this chain has produced that evidence, and no wave
can without the board. Every claim above is about specs, RTL and tool reports.

## 2. Hardware — from the SSOT, non-negotiable

| | |
|---|---|
| Board | **QMTech Wukong V1**, `XC7A100T-FGG676` (`xc7a100tfgg676-1`) |
| IDCODE | `0x13631093` |
| Programmer | **in-repo Rust driver `cli/dlc10`** — `dlc10 idcode\|sram\|flash\|reload` |
| **Not** | `openFPGALoader` — it cannot drive the `0x03FD` Xilinx cable |
| **Not** | Arty A7 / `csg324` — different package, different part |
| Synthesis | Vivado-in-Docker or OpenXC7. **No native macOS Vivado exists.** |

## 3. The plan

### Phase 0 — before the cable arrives (all of this is done)

| Step | State |
|---|---|
| RTL equivalence proved | ✅ T1 |
| Multiplier-freedom proved | ✅ T2 |
| Timing closure reported | ✅ T3 — 150.63 MHz |
| FPGA spec family green | ✅ 300/300 |
| Per-spec test measurement is a command | ✅ `t27c test-report` |
| Board spec present | ✅ `specs/boards/xc7a100t_minimal.t27` — 23/23 |

**Nothing in Phase 0 remains.** This is why every wave since W568 has reported
Variant C as blocked rather than in progress.

### Phase 1 — the cable, in order

1. **Connect** the Digilent/Xilinx programming cable (VID `0x03FD`) over USB.
2. **`dlc10 idcode`** — the single gating check. It must return `0x13631093`.
   Any other value means the wrong board, and everything downstream is invalid.
3. **`dlc10 sram <bitstream>`** — volatile load first. Survives a power cycle by
   vanishing, which is exactly what you want for a first attempt.
4. Read back status; confirm DONE asserts.
5. **`dlc10 flash <bitstream>`** — only after SRAM load is confirmed good.
6. **`dlc10 reload`** — verify the flashed image boots on its own.

**Do not skip step 2 to save time.** It is the only step that distinguishes "the
design is wrong" from "this is not the board the design targets", and every
subsequent failure is ambiguous without it.

### Phase 2 — first evidence worth having

The goal of the first successful load is **not** a benchmark. It is to convert
one simulation claim into a hardware claim:

1. **Loopback / identity**: drive a known input vector through the ternary MAC
   and read back the accumulator. T1 says simulation and RTL agree; this says
   silicon and simulation agree.
2. **The multiplier-freedom claim, physically**: T2 is a netlist property.
   Confirm the utilisation report shows **0 DSP48** slices used.
3. **Clock**: T3 reports 150.63 MHz from the tool. Measure whether the design
   actually runs at the constrained frequency without errors.

Each is a single, falsifiable statement. Collect them in that order; the first
failure tells you the most.

### Phase 3 — the honest benchmark

Only after Phases 1–2. A performance number taken before the correctness
evidence is a number nobody can interpret, and this chain's whole record is that
**a stage which cannot fail cannot be trusted.**

## 4. The improvement plan for the model side

Ordered by what is blocked on what.

| # | Item | Blocked on | Size |
|---|---|---|---|
| 1 | **CORDIC argument reduction (T6)** | owner sign-off — it changes behaviour | small; T6 gives the algorithm |
| 2 | `ternary_mac` argument order | **a decision** — 91 call sites say `(acc, a, w)`, 80 say `(a, w, acc)`, inside the declaring module. 849 assertions | one line, once decided |
| 3 | `systolic_ternary_array` output length | **a decision** — an invariant says `len == size`, a test says `len == 0` | small |
| 4 | `OP_ADD` / `OP_SUB` vs the sacred opcode set | **a decision** — the ISA encoding table | small |
| 5 | `PpaMetrics` field mismatch | **a decision** | small |
| 6 | The 5 false CORDIC assertions | **a decision** — arithmetic already written down (T6, K(n) monotonicity, table index) | trivial once decided |
| 7 | 25 stub specs | **a decision** — write them, or delete them | large or zero |
| 8 | 540 BLOCKED specs | genuine engineering | large |

**Items 2–6 are all the same shape: a maintainer's judgement, with the
arithmetic already done and recorded.** None is blocked on capability. Item 1 is
the only known *code* defect in the measured corpus.

## 5. What would falsify this plan

- `dlc10 idcode` returns anything other than `0x13631093` → the SSOT's board
  identification is wrong and §2 must be rewritten before anything else.
- The utilisation report shows DSP48 usage → **T2 is false on the real toolchain**
  even though it holds on the netlist yosys produced.
- The design fails at 150.63 MHz on hardware → T3 was a tool report, and this
  is precisely the gap between it and a proof.

Each of these is a specific, cheap check that turns a claim into knowledge. That
is the only reason to want the board.

---

*φ² + φ⁻² = 3 | TRINITY*
