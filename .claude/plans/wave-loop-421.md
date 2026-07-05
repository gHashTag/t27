# Wave Loop 421 Decomposed Plan

**Issue:** #1363  
**Branch:** `wave-loop-421`  
**Date:** 2026-07-06  
**Selected variant:** C (fallback — hardware still unreachable)

---

## 1. Weak-point analysis

### 1.1 W420 dependency / branch state
- `wave-loop-421` was created from `master` (101fd0748) **before** PR #1362
  (Wave Loop 420) merges. This means the W421 branch initially lacked the W420
  VCD exact-terminator and auto-threshold work.
- **Mitigation:** reset `wave-loop-421` onto `wave-loop-420` so W421 builds on
  top of W420. Once PR #1362 lands, W421 can be rebased onto `master` cleanly.

### 1.2 VCD parser remaining gaps
- `$date` / `$version` / `$comment` sections now use exact-token terminators
  (W420), but `$timescale` still uses the old substring heuristics
  `contains(" $end")` / `ends_with(" $end")`. A `$timescale` comment that
  mentions `$end` could therefore mis-parse.
- Real-valued net auto-threshold works, but has no regression test for a
  non-default `$timescale` unit (e.g., `1 us` or `1 ps`).

### 1.3 PVT envelope
- W419/W420 proved each axis independently (temp monotone, VCCINT antitone,
  corner monotone). The combined ordering (temp ↑, VCCINT ↓, corner worse) is
  not yet a single lemma/test, which is what a worst-case operating-point
  search actually needs.

### 1.4 Competition
- Sparkle/Verilean (Lean 4 native HDL) is the closest competitor and is
  accelerating in 2026: RV32IMA SoC, networking stack, crypto, formal proofs.
- Clash Formal and CIRCT/Chisel LTL/Verif dialects are also closing the formal
  gap from the mainstream side.
- t27 must keep differentiating through: (a) ternary/balanced-trit compute + Lean
  proof lattice, (b) spec-first `*.t27 → gen/` sealed pipeline, (c) physical
  boot-evidence instrumentation.

### 1.5 Hardware
- The Digilent FTDI cable is present (`0x0403:0x6014`), but `openFPGALoader
  --detect` reports **0 devices**. The board is either not powered, not wired,
  or the JTAG header is disconnected. Variant A is therefore still blocked.

---

## 2. Work items

| # | File | Change | Test |
|---|------|--------|------|
| 1 | `cli/tri/src/fpga.rs` | Apply `vcd_line_ends_with_token` to `$timescale` terminator | `test_parse_vcd_timescale_with_embedded_end_token` |
| 2 | `cli/tri/src/fpga.rs` | Add non-default `$timescale` real-net parse test | `test_parse_vcd_real_auto_threshold_us_timescale` |
| 3 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Add combined PVT monotonicity lemma | `lake build Trinity.TernaryFPGABoot` |
| 4 | `cli/tri/src/fpga.rs` | Add combined PVT monotonicity Rust test | `test_pvt_half_ns_monotone_combined` |
| 5 | `docs/reports/T27_VS_FORMAL_HDL_2026.md` | Competitor comparison note | visual review |
| 6 | `fpga/HARDWARE_SSOT.md` | §3.6.18 documenting W421 parser + PVT improvements | visual review |
| 7 | `docs/NOW.md` | W421 close-out / W422 setup | visual review |
| 8 | `.trinity/experience.md` | W421 learnings | visual review |
| 9 | `docs/reports/*` | W421 report, evidence, W422 cooperation | visual review |
| 10 | git/PR/issue | Commit, open PR #? for W421, create W422 issue + branch | CI passes |

---

## 3. Verification checklist

- [ ] `cargo test -p tri vcd`: all PASS (was 13).
- [ ] `cargo test -p tri pvt`: all PASS (was 10).
- [ ] `cargo test -p tri fpga::tests`: all PASS (was 48).
- [ ] `lake build Trinity.TernaryFPGABoot`: PASS.
- [ ] `./scripts/tri test`: no new failures beyond the 16 pre-existing yosys
      smoke failures from weak point #1245.

---

*φ² + φ⁻² = 3 | TRINITY*
