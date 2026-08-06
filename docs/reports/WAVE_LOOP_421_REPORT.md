# Wave Loop 421 Report — Variant C fallback: VCD `$timescale` exact terminator, combined PVT monotonicity, competitor snapshot

**Issue:** #1363  
**Branch:** `wave-loop-421`  
**Variant:** C (bench still blocked: `openFPGALoader --detect` reports 0 devices).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 421 continued the formal-only guarding and instrument-import depth
line while the physical bench remains unreachable. The wave delivered three
main results:

1. **VCD `$timescale` exact-token terminator.** W420 hardened `$date`,
   `$version`, and `$comment`, but `$timescale` still used substring
   heuristics. W421 applied the same `vcd_line_ends_with_token` helper to
   `$timescale`, closing the last header-section terminator gap.
2. **Combined PVT monotonicity.** Added the Lean 4 lemma
   `pvt_half_ns_monotone_combined` and a Rust operating-rectangle test that
   proves the half-period bound is monotone under the combined ordering:
   temperature non-decreasing, VCCINT non-increasing, process corner worse.
   This is the shape property a worst-case operating-point search relies on.
3. **Competitor snapshot.** Published `docs/reports/T27_VS_FORMAL_HDL_2026.md`
   comparing t27 to Sparkle/Verilean, Clash, Chisel/FIRRTL/CIRCT, Bluespec,
   Coq Kami/Silver Oak, ACL2, and Knox/HARDENS. Sparkle/Verilean is identified
   as the closest Lean-native threat in 2026.

A process/discovery finding: `wave-loop-421` was originally created from
`master` before PR #1362 (W420) merged. The branch was reset onto
`wave-loop-420` before implementation to avoid building on a stale base.

---

## What changed

### 1. VCD `$timescale` exact-token terminator

`cli/tri/src/fpga.rs`:

- Replaced the heuristic `contains(" $end")` / `ends_with(" $end")` checks in
  the `$timescale` section with `vcd_line_ends_with_token(trimmed, "$end")`.
- Single-line and multi-line `$timescale` blocks are both handled; the
  timescale value is computed only when the section truly ends.

### 2. Regression tests for `$timescale` robustness

`cli/tri/src/fpga.rs`:

- `test_parse_vcd_timescale_with_embedded_end_token`: a multi-line `$timescale`
  block containing the literal substring `$end` in an inline comment before the
  real `$end` terminator. The old heuristic terminated early and swallowed the
  following `$var` declaration.
- `test_parse_vcd_real_auto_threshold_us_timescale`: a real-valued VCD net
  with `$timescale 1 us $end` and no explicit threshold, validating that the
  auto-threshold path works with non-nanosecond timescales.

### 3. Combined PVT monotonicity

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- Added `pvt_half_ns_monotone_combined (t1 t2 : Int) (v1 v2 : Nat) (c1 c2 : ProcessCorner) :
  t1 ≤ t2 ∧ v2 ≤ v1 ∧ c1.worse_than c2 → bound ctx1 ≤ bound ctx2`. The proof
  expands the three existing derating functions and discharges the corner
  enumeration with `omega`.

`cli/tri/src/fpga.rs`:

- Added `test_pvt_half_ns_monotone_combined`, an operating-rectangle sweep that
  checks the combined ordering property across temperature, VCCINT, and process
  corner.

### 4. Competitor snapshot

`docs/reports/T27_VS_FORMAL_HDL_2026.md`:

- Matrix comparing Sparkle/Verilean, Clash, Chisel/FIRRTL/CIRCT, Bluespec,
  Coq Kami/Silver Oak, ACL2, Knox/HARDENS.
- Deep dive on Sparkle/Verilean as the closest Lean-native competitor, with
  sources and strategic implications for t27.

### 5. Documentation

`fpga/HARDWARE_SSOT.md`:

- Added §3.6.18 documenting W421 VCD/PVT improvements and the current hardware
  blocker.

---

## Verification

- `cargo test -p tri vcd`: **PASS** (15 tests, was 13).
- `cargo test -p tri pvt`: **PASS** (11 tests, was 10).
- `cargo test -p tri fpga::tests`: **PASS** (51 tests, was 48).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245 (unchanged).

---

## Weak points closed

| Weak point | Location | Fix |
|------------|----------|-----|
| `$timescale` still used substring terminator after W420 | `cli/tri/src/fpga.rs` | Exact-token terminator helper |
| No regression test for real-valued VCD with non-default timescale | `cli/tri/src/fpga.rs` | `test_parse_vcd_real_auto_threshold_us_timescale` |
| PVT envelope lacks combined monotonicity lemma | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | `pvt_half_ns_monotone_combined` |
| No public competitor comparison | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Competitor snapshot |
| W421 branch based on stale `master` before W420 merged | git workflow | Reset W421 onto W420 |

---

## Competitor scan

The credible formal-HDL competition is accelerating in 2026:

- **Sparkle/Verilean** (Lean 4 native HDL) is the closest structural competitor.
  It now has a broad IP catalog (RV32IMA SoC, networking, crypto) and active
  formal verification work, but no ternary compute line, no spec-first sealed
  pipeline, and no physical boot-evidence instrumentation.
- **Clash** is maturing its formal verification integration (Clash Formal,
  Yosys/SymbiYosys, CIRCT), but proof is external to the source language.
- **Chisel/FIRRTL/CIRCT** is adding LTL/SVA/Verif support rapidly and has
  industry adoption, but formal reasoning is at RTL/SVA level, not native
  dependent-type proof.

T27's defensible intersection remains: Lean 4 native theorem proving + ternary
compute + spec-first `*.t27 → gen/` sealed pipeline + physical boot-evidence
instrumentation.

---

## Not done (blocked on hardware or unsafe)

- Real P12 CCLK capture for `OSCFSEL=6/7` — `openFPGALoader --detect` reports 0
  devices; board not powered/connected.
- Real relay cold-POR gate — no relay board / USB power switch available.
- Safe gen-verilog #1245 sub-fix deferred; the remaining tracked gaps (RAM
  style inference, tuple-return syntax) are not narrow regression-free fixes
  suitable for a Variant C wave.

---

*φ² + φ⁻² = 3 | TRINITY*
