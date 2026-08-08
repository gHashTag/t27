# Wave Loop 554 Report — the metric I built to catch overstatement was overstating

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_553_REPORT.md`](WAVE_LOOP_553_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W554 set out to do the no-hardware half of W553's Variant A: turn the chipdb
recipe into a command so it cannot rot, and get a comparable resource number
for a ternary GEMM array. The first succeeded. The second produced the finding
of the wave, and it is about my own instrumentation.

**0 of 7 "synthesising" IGLA RACE specs produce any hardware.**

---

## 1. Delivered: `t27c fpga-chipdb`

W553 produced the first bitstream by discovering that `bbaexport` must run
natively (7.06 GB peak) while the rest of the flow stays in Docker. That
knowledge lived only in a markdown recipe. It is now a command:

```bash
t27c fpga-chipdb --device xc7a200tfbg676-1
```

- Extracts prjxray-db, metadata, python and constids from the image on first
  run; skips extraction afterwards.
- Runs `bbaexport` **natively** — the step that cannot fit in Docker Desktop's
  allocation.
- Runs `bbasm` in Docker.
- Idempotent: an existing chipdb is reported and kept unless `--force`.
- **Reports exit 137 as "KILLED by the OOM killer"** with the ~7 GB figure,
  because the tool prints nothing when that happens — the exact silence that
  cost Waves 549–552 two wrong diagnoses.
- Treats a zero-byte `.bba` after a "successful" exit as a failure.

Artifacts follow `<device>.bba` / `<device>.bin`. W553's hand-named files were
renamed to match, and the full place-and-route was re-verified against the
renamed chipdb: **identical result — 150.63 MHz, 0 errors, byte-identical
81,315-byte FASM.**

---

## 2. The finding

`COMPETITORS.md` said the comparable number — a ternary GEMM array — "does not
exist here yet". Checking whether that was still true:

```
$ t27c gen-verilog specs/igla/race/ternary_gemm.t27   → 1,596 lines, OK
$ yosys synth_xilinx -abc9 -nocarry -arch xc7          → OK, 0 errors
```

So it looked obtainable. The netlist says otherwise:

```
463 cells = 459 $print + 3 IBUF + 1 OBUF
Estimated number of LCs: 0
```

**Zero logic cells.** The generated module carries a fixed
`clk / rst_n / en / ready` interface, drives only `assign ready = 1'b1;`, and
emits the spec's arithmetic as five Verilog `function` definitions that
**nothing instantiates** — so synthesis optimises all of it away.

Measured across `specs/igla/race`:

| Stage | Result |
|---|---|
| gen-verilog succeeds | 8 / 17 |
| yosys synthesises it | 7 / 17 |
| **produces any logic** | **0 / 17** |

### It is not specific to IGLA — and the fair reading

Sampling 40 generating specs across the whole tree, plus the `specs/fpga/`
family (`uart`, `gf16_accel`, `memory`): **none** produces a non-zero
logic-cell count. The emitted Verilog for `specs/fpga/uart.t27` contains
**0 `always` blocks, 1 `assign`, 6 `function` definitions, 39 `$display`
statements**.

That shape is a function library inside a simulation harness, not a datapath —
and it is what the project actually validates. `README.md` lists the gates a
spec must pass: `parse`, `icarus-lowerable`, `icarus-simulate`,
`icarus-cocotb`, `seal --save`. **Synthesis is not among them.**

So the honest conclusion is narrower and more useful than "the backend is
broken": **the Verilog backend targets Icarus simulation, not synthesis.**
"Compiles to Verilog" is true; "compiles to synthesisable RTL" is a different
claim and is not demonstrated. The risk is that a reader assumes the second
follows from the first — which is what `COMPETITORS.md` now spells out.

### What this means for IGLA RACE

**The ternary MAC that works, that theorems T1–T3 prove, and that is inside the
W553 bitstream, is hand-written Verilog** —
`fpga/verilog/ternary_mac_synth.v`, 59 LUT / 32 FF. The `.t27` spec of the same
name generates no hardware.

For this line specifically, the spec-to-RTL claim in `COMPETITORS.md` §3 is not
demonstrated. That is now stated there.

---

## 3. The mistake was mine

W549 introduced `synth-gate` *precisely* to stop static metrics overstating
hardware readiness — grounded in Fu et al.
([arXiv:2603.11287](https://arxiv.org/abs/2603.11287)), which shows simulation
pass rates overstate silicon readiness. The gate then did the same thing: it
counted "yosys exited 0" as synthesis success and reported **7/17 (41.2 %)**,
a figure that appeared in the W549 research report and in issue #1959.

The correct figure is **0/17**.

**Fixed:** `synth-gate` now parses yosys's `Estimated number of LCs` and reports
each spec as `<n> LC` or `HOLLOW (0 logic cells)`, with a total split and an
explanation. Corrections applied to `WAVE_LOOP_549_RESEARCH.md` §0 and §3 and to
`COMPETITORS.md` §4.2.

**The general lesson, added to the skill:** when you add a metric to catch
overstatement, immediately ask what its own hollow-success case looks like.
*"The tool exited 0" is never the measurement.* Find the quantity that would be
zero if nothing happened, and report that. For synthesis it is logic cells, not
exit status.

This is the fifth integrity claim in this chain found satisfiable by content
that means nothing — after vacuous tests (57 % `assert true`), static readiness,
vacuous seals (`none` matches `none`), and inflated invariant counts. The first
four were the repository's. This one was mine.

---

## 4. Competitive positioning, updated with measured numbers

W553 produced the first measured figures for IGLA RACE, so `COMPETITORS.md`
§4.2 now records them and — more importantly — what they do **not** license:

| | |
|---|---|
| Place-and-route | 0 errors, `xc7a200tfbg676-1` |
| Max frequency | **150.63 MHz** (constraint 80 MHz) |
| Resources | 120 SLICE_LUTX, 60 SLICE_FFX, **0 DSP48** |

Read with T1 and T2, this licenses exactly one competitive claim: *for a single
8-bit × ternary MAC cell, the multiplier-free implementation is exact and costs
zero DSP48 where the equivalent `*`-based design costs one.*

It does **not** license comparison with FINN or hls4ml. Those report
network-level accelerator resources, and a literature search for single-cell MAC
costs on comparable parts returns nothing directly comparable. Comparing one
cell to a whole accelerator is a category error in either direction.

---

## 5. Three cooperation variants for W555

### Variant A (recommended) — Find out why generated modules have no datapath

**Hypothesis.** `gen-verilog` emits spec functions as Verilog `function`
definitions and gives every module the same `clk/rst_n/en/ready` interface with
`assign ready = 1'b1;`. Nothing connects a spec's data types to module ports, so
no datapath is ever instantiated. If that is the mechanism, it is a single
backend gap, and it is the difference between t27 being a spec language for
hardware and being a spec language that *describes* hardware.

**Why first.** It is the root cause of the wave's finding, it is unblocked (the
emitter lives in `compiler.rs`… — see the caveat below), and every hardware
claim for the IGLA line depends on it.

**Resolved during this wave:** the emitter *is* in
`bootstrap/src/compiler.rs` (the `assign ready = 1'b1;` line is at
`compiler.rs:6887`), which `build.rs` watches — so this variant **is blocked by
the LANG-EN gate**, exactly like Variant B. Checked rather than assumed.

**Deliverables.**
1. Determine how `gen-verilog` chooses module ports, and whether any spec in
   the repo produces a non-hollow design. If some do, diff them against IGLA's.
2. If none do: the backend has never produced a datapath, and that is the
   single most important fact about the project's spec-to-RTL claim.
3. Extend `synth-gate --strict` to fail on hollow designs and wire it into CI.

**What would falsify it.** If some specs *do* synthesise to real logic, the
mechanism is spec-side rather than backend-side, and the variant becomes "fix
the IGLA specs" instead.

### Variant B — Clear the LANG-EN gate

Unchanged since W550 and still **blocked on a human decision**. Six documents
violate L3 and are not allowlisted, so `build.rs` panics on any `compiler.rs`
edit. This now blocks Variant A as well as the ~84-spec syntax-gap work, which
makes it the single highest-leverage approval outstanding.

### Variant C — Flash the board

Everything software-side is done: bitstream built, `fpga-flash` pre-flights
clean, T3 gives a falsifiable prediction. Needs only the Wukong V1 and its
Digilent HS2 cable.

---

## Recommendation

**Variant A, if it turns out not to need `compiler.rs`** — check that first,
because the answer decides whether anything can proceed without the LANG-EN
approval. If it does need `compiler.rs`, then **Variant B is the only thing
that unblocks the project**, and it is a one-line decision.

---

*φ² + φ⁻² = 3 | TRINITY*
