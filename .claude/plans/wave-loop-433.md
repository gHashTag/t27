# Wave Loop 433 Decomposed Plan

**Issue:** #1393  
**Branch:** `wave-loop-433`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### Physical bench (still blocked)
- P12 (CFGCLK / CCLK_0) is not wired to a logic-analyzer channel.
- No relay / remote-power cold-POR gate is wired.
- The on-board Xilinx DLC10 / Platform Cable USB II is not connected.
- **Implication:** Variant A (real CCLK capture) and Variant B (live XADC + real board) are not executable unless the user has wired new hardware since W432.

### Master-merge debt (still blocked)
- W432 probed `origin/master` and found the `gen-verilog` fix set (`701d79b3b`, `507408f47`) is on a divergent `master` lineage not safely reachable from `wave-loop-432`.
- A direct cherry-pick of `507408f47` conflicts heavily with `bootstrap/src/compiler.rs`, seals, and `docs/NOW.md`.
- **Implication:** Variant C1 (master-merge to clear #1245) is high-risk and would destabilize the FPGA/formal work. It should only be attempted in a dedicated wave when the boot-evidence line is not the primary focus.

### Formal gap left by W432
- W432 added per-process-corner raw-ns OSCFSEL theorems at the **worst-case** envelope corner (temp = +85 °C, vccint = 900 mV).
- A live `tri fpga read-xadc` measurement will almost always be **inside** the envelope but not exactly at the worst-case corner.
- The W431 bridge (`xadc_envelope_implies_raw_ns_satisfies_any_in_envelope`) shows that any raw-ns capture safe at the worst-case corner is also safe at any in-envelope point.
- **Gap:** there is no single theorem that says "for any OSCFSEL and any in-envelope live XADC point, the nominal raw-ns period is safe under the measured PVT context." Closing this gap makes the W432 corner theorem directly usable with real XADC data.

### Tooling/reporting gap
- `tri fpga sweep-report --json` exists but does not surface per-variant PVT context or process-corner metadata.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` needs a late-July 2026 refresh (firtool 1.152.0 published July 4, Sparkle IP.Net PR #66 still open, Clash 1.11 candidate).
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` needs a W433 triage entry.

---

## 2. Competitor research (summary)

- **Sparkle / Verilean:** PR #66 ("IP.Net") remains the headline signal — USB web server + memcached + compiler perf, +27k lines. Last public push 2026-07-03. No new public PRs after that. The RV32 divider proof (commit `9c7809c`, June 25) remains the deepest formal IP-level proof t27 has not yet matched.
- **Clash 1.11.0:** still a Hackage candidate as of late July 2026; no promoted release. Latest official release remains 1.10.0 (April 2026).
- **CIRCT / firtool:** `firtool-1.152.0` was published 2026-07-04; it is a maintenance release (ImportVerilog/Moore, Arc dialect, FIRRTL inliner). The major formal-verification expansion was `firtool-1.143.0` (March 2026). PR #10387 (`ifdef SYNTHESIS` guards for SV lowering) was merged in May and later reconsidered.
- **Aria-HDL / fpga-meta-compiler-public:** Rust-based meta-compiler with `--emit-lean4` proof extraction and `--emit-sby` backend. Recent 2026 updates around Leiserson-Saxe retiming, constraint annotations, and PCIe BAR testing.
- **CktFormalizer:** arXiv 2605.07782 (autoformalization into dependently-typed Lean 4 HDL) — no new public July signal, but reinforces the "Lean 4 as hardware proof backend" trend.

---

## 3. Selected primary variant

**Variant C3 — formal bridge fallback**

Land a board-less formal lemma that connects a live `XadcOperatingPoint` to the
W432 per-process-corner raw-ns OSCFSEL theorem. This closes the remaining gap in
the boot-to-proof pipeline without touching the compiler or requiring physical
hardware.

### Acceptance criteria
- New theorem `xadc_envelope_justifies_cclk_variant_raw_ns_pvt` in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` passes `lake build`.
- New transaction variant `xadc_envelope_justifies_cclk_variant_transaction_ok`.
- `cargo test --bin tri fpga::` passes.
- `./scripts/tri test` passes with the documented 7-failure baseline.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` and
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` are refreshed.
- W433 report, evidence note, and W434 cooperation variants are written.
- GitHub issue and branch for W434 are created.

---

## 4. Decomposed tasks

1. **Formal lemma**
   - Add `xadc_envelope_justifies_cclk_variant_raw_ns_pvt`.
   - Add `xadc_envelope_justifies_cclk_variant_transaction_ok`.
   - Build `Trinity.TernaryFPGABoot`.

2. **Rust validation**
   - Run `cargo test --bin tri fpga::`.

3. **Full CI sweep**
   - Run `./scripts/tri test` and record baseline.

4. **Documentation refresh**
   - Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
   - Add W433 triage to `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

5. **Close-out artifacts**
   - Write `docs/reports/WAVE_LOOP_433_REPORT.md`.
   - Write `docs/reports/FPGA_LOOP_EVIDENCE_W433_2026-07-01.md`.
   - Write `docs/reports/FPGA_LOOP_COOPERATION_W434_2026-07-01.md`.

6. **Next-wave setup**
   - Create GitHub issue #? for W434.
   - Create and push branch `wave-loop-434`.
   - Update `.trinity/current-issue.md` and `docs/NOW.md`.
   - Append W433 learnings to `.trinity/experience.md`.
   - Save persistent memory entry.

---

## 5. Fallback if formal lemma is blocked

If the Lean composition proves unexpectedly difficult, redirect to one of:

- **C2-bis:** harden `tri fpga sweep-report --json` with per-variant `process_corner` / `pvt_context` fields (when boot-log JSON contains XADC data).
- **C4:** deeper competitor refresh only, with a clear note that no code changed.

---

*φ² + φ⁻² = 3 | TRINITY*
