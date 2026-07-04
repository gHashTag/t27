# Wave Loop 420 Report — Variant C fallback: VCD exact-terminator + auto-threshold, PVT corner monotonicity

**Issue:** #1361  
**Branch:** `wave-loop-420`  
**Variant:** C (bench still blocked: P12 unwired, DLC10 cable missing, no relay).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 420 continued the FPGA/formal evidence line while the physical bench
remains unavailable. The wave delivered three instrument-import / formal
guarding improvements:

1. **VCD `$comment` exact-token terminator.** The W419 report claimed this
   hardening landed, but the merged commit (`101fd0748`) did not touch the VCD
   parser. The heuristic `ends_with("$end")` / `contains(" $end")` was still in
   place and could terminate a `$comment` section early on embedded `$end`-like
   strings. W420 added a `vcd_line_ends_with_token` helper and applies an exact
   `$end` token check to `$date`, `$version`, and `$comment` sections.
2. **Real-valued VCD auto-threshold.** Previously, real-valued VCD nets required
   an explicit `--vcd-threshold-v`. W420 computes the threshold as
   `50% (vmin + vmax)` from the observed voltage swing when the option is
   omitted, removing a manual step for oscilloscope imports.
3. **PVT process-corner monotonicity.** Added the Lean 4 lemma
   `pvt_half_ns_monotone_in_process_corner` and a Rust operating-rectangle test
   verifying that the half-period bound respects the `ff ≤ tt ≤ ss` ordering.
   Temperature monotonicity and VCCINT antitonicity from W419 are now joined by
   the last independent shape axis.

No new silicon evidence was produced — the physical-evidence gap remains until
P12 is wired and the cable/relay hardware is available.

---

## What changed

### 1. VCD exact-token section terminator

`cli/tri/src/fpga.rs`:

- Added `vcd_line_ends_with_token(line, token)` which checks only the last
  whitespace-delimited token. This is stricter than the old substring heuristics.
- Replaced the `$date`, `$version`, and `$comment` terminator checks with the
  exact-token helper.
- Added `test_parse_vcd_comment_with_embedded_end_token`, which constructs a
  `$comment` block containing the literal text `$end` before the real `$end`
  terminator, followed by a scalar `$var cclk`. If the heuristic terminator is
  used, the parser swallows the `$var` declaration and fails with "VCD has no
  scalar or selectable logic net".

### 2. Real-valued VCD auto-threshold

`cli/tri/src/fpga.rs`:

- When a selected VCD net is real-valued (`$var real ...`) and
  `--vcd-threshold-v` is not supplied, all sampled voltages are collected in a
  `real_samples` buffer.
- After the parse loop, `vmin`/`vmax` are computed and the threshold is set to
  their midpoint. The threshold is printed so the user can audit it.
- The collected samples are then converted into transitions using the computed
  threshold.
- Added `test_parse_vcd_real_auto_threshold` which exercises a synthetic 0 V /
  3.3 V 25 MHz real-valued square wave without supplying a threshold.

### 3. PVT process-corner monotonicity

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- Added `pvt_half_ns_monotone_in_process_corner (t : Int) (v : Nat) (c1 c2 : ProcessCorner) :
  c1.worse_than c2 → bound c1 ≤ bound c2`. The proof expands the existing
  `ProcessCorner.worse_than` order and discharges the three corner cases with
  `omega`.

`cli/tri/src/fpga.rs`:

- Added `test_pvt_half_ns_monotone_in_process_corner`, an operating-rectangle
  sweep over temperature, VCCINT, and the three corner pairs `(ff, tt)`,
  `(tt, ss)`, `(ff, ss)`.

### 4. Documentation

`fpga/HARDWARE_SSOT.md`:

- Added §3.6.17 documenting the VCD exact terminator, real-net auto-threshold,
  and PVT corner monotonicity work.

---

## Verification

- `cargo test -p tri vcd`: **PASS** (13 tests, was 11).
- `cargo test -p tri csv`: **PASS** (11 tests, unchanged).
- `cargo test -p tri pvt`: **PASS** (10 tests, was 9).
- `cargo test -p tri fpga::tests`: **PASS** (48 tests, was 45).
- `cargo test -p tri test_measured_to_lean_standalone_lake_package_builds`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245 (unchanged).

---

## Weak points closed

| Weak point | Location | Fix |
|------------|----------|-----|
| `$comment` heuristic terminator can terminate early on embedded `$end` | `cli/tri/src/fpga.rs` | Exact-token terminator helper |
| Real-valued VCD imports require manual threshold | `cli/tri/src/fpga.rs` | Auto-threshold from observed swing |
| PVT envelope lacks process-corner shape lemma | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | `pvt_half_ns_monotone_in_process_corner` |

---

## Competitor scan

The credible formal-HDL competition is real and accelerating, especially
**Sparkle / Verilean** in the Lean 4 space. Other players: **Clash** (Haskell,
external formal), **Chisel/FIRRTL/CIRCT** (mainstream, SVA/model-checking),
**Bluespec** (rule-based, Coq bridge via Kami), **Coq Kami / Silver Oak**
(dependent-type hardware), **ACL2** (specification/proof only).

None of the listed competitors combine:
1. Lean 4 native theorem proving,
2. a ternary/balanced-trit ISA and MAC proof lattice,
3. a sealed, spec-first `*.t27 → gen/` → bitstream pipeline, and
4. physical boot-evidence instrumentation (VCD/CSV import + PVT envelope).

That intersection remains the gap t27 is positioned to fill.

---

## Not done (blocked on hardware)

- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.
- Safe gen-verilog #1245 sub-fix deferred; the remaining tracked gap (RAM style
  inference) is not a narrow regression-free sub-fix suitable for a Variant C
  wave.

---

*φ² + φ⁻² = 3 | TRINITY*
