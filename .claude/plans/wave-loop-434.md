# Wave Loop 434 — Decomposed Plan

**Issue:** #1395  
**Branch:** `wave-loop-434`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### Physical bench
- **P12 is still unwired** (no logic-analyzer channel on the CCLK pin), so **Variant A** (real cold-POR CCLK capture) remains infeasible.
- **No relay / remote-power gate** exists, so automated cold-POR sweeps are still manual.
- **No DLC10 cable** (VID=0x03FD), so in-repo `dlc10` driver cannot be used; the reachable cable is the Digilent FTDI (`0x0403:0x6014`), which works with `openFPGALoader`.
- The FPGA **is reachable** over JTAG and live XADC readout succeeds (`tri fpga read-xadc` returns temp≈41 °C, VCCINT≈1.00 V, VCCAUX≈1.81 V).

### Tooling gaps discovered this wave
- `tri fpga read-xadc` emits `temp_c` / `vccint_v` / `vccaux_v` as `f64`, but `tri fpga pvt-envelope --pvt-context` expects integer `temp_c` (`i64`) and `vccint_mv` / `vccaux_mv` (`u64`). The conversion (`to_pvt_context`) is implemented internally but not exposed as a standalone `--to-pvt-context <file>` export, so the user must round the values by hand.
- `tri fpga measured-to-lean --pvt-context` generates a `decide` theorem for the exact rounded point, but it does **not** automatically reference the W433 quantified theorem `xadc_envelope_justifies_cclk_variant_raw_ns_pvt`. The formal bridge exists in the library but is not wired into the generator.

### Strategic / competitive
- The 7 residual `gen-verilog` yosys smoke failures (#1245) are unchanged. A master-merge of the full fix set is still too risky for a wave whose primary goal is FPGA boot-evidence formalization.
- Competitor `Sparkle / Verilean` remains the closest Lean-native threat; its PR #66 is still open, PR #65 (RV32 divider proof) demonstrates IP-level depth, and the July 2026 Functional Matsuri talk positions Lean 4 as an RTL core. Other signals: `firtool-1.152.0` published; `Clash 1.11.0` still a Hackage candidate; `Aria-HDL` retiming/PCIe BAR updates; `CktFormalizer` and ternary-compute projects (`TernaryCore`, `BitNet-RISCV-Multicore`) validate the ternary direction.

---

## 2. Competitor scan summary

| Competitor | Update for W434 | Implication for t27 |
|---|---|---|
| **Sparkle / Verilean** | PR #66 open, last public push 2026-07-03; PR #65 divider proof is a concrete IP-level correctness milestone; public talk July 11 2026. | Closest structural competitor; t27's differentiation is the sealed spec→bitstream loop + physical boot evidence. |
| **Clash** | 1.11.0 still a Hackage candidate (no final release); 1.10 remains latest official. | Functional-HDL maturity but external proof; no physical evidence loop. |
| **Chisel / FIRRTL / CIRCT** | `firtool-1.152.0` published July 2026; LTL/Verif dialects and ChiselTest formal compatibility layer advancing. | Industry adoption; formal reasoning still RTL/SVA/external, not source-level dependent types. |
| **Aria-HDL** | Rust meta-compiler with `--emit-lean4` and `--emit-sby`; retiming + PCIe BAR test added. | Validates spec→proof→bitstream pipelines but no ternary focus or sealed hashes. |
| **CktFormalizer** | arXiv 2605.07782; LLM-to-circuit autoformalization in Lean 4 with Yosys/OpenROAD flow. | Another signal that Lean 4 as HDL proof backend is crowded. |
| **TernaryCore / BitNet-RISCV-Multicore** | Ternary inference and multicore RISC-V ternary PEs simulating; no formal proofs yet. | Confirms ternary compute hardware is visible; t27 must keep formal ternary IP ahead. |

---

## 3. Variant selection

**Selected: Variant B** — board is reachable over JTAG, live XADC readout works, but P12 / relay are still blocked, so real CCLK capture is not possible. Use the live XADC operating point as the PVT context and a synthetic CCLK fixture for proof-of-pipeline.

---

## 4. Decomposed tasks

### Task 1 — Live XADC operating point → PVT context (tooling)
- [x] Probe: `openFPGALoader --detect -c digilent_hs2` succeeds.
- [x] Capture: `tri fpga read-xadc` returns valid JSON.
- [x] Convert manually to integer `PvtContext` JSON: `{ "temp_c": 41, "vccint_mv": 1000, "vccaux_mv": 1807, "process_corner": "ss" }`.
- [x] Validate: `tri fpga pvt-envelope --pvt-context ... --json` reports in-envelope with `margin_ns = 5`, `min_sck_half_ns = 11`.
- [ ] Optionally expose `tri fpga read-xadc --to-pvt-context <file>` or `--process-corner <corner>` in CLI (deferred to W435 if scope grows).

### Task 2 — Generate proof artifact from live XADC context
- [x] Create synthetic raw-ns CCLK fixture: `period_ns=40, sck_low_ns=20, sck_high_ns=20` (OSCFSEL=6 nominal 25 MHz, 50% duty).
- [x] Run `tri fpga measured-to-lean --raw-ns --file <fixture> --pvt-context <xadc> --validate --standalone --name xadc_live_w434 --out <file> --json`.
- [ ] Add a hand-written theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean` that applies the W433 quantified theorem `xadc_envelope_justifies_cclk_variant_raw_ns_pvt` to the live XADC point and OSCFSEL=6, closing the formal loop.

### Task 3 — Rust test coverage for live XADC → PVT conversion
- [ ] Add a unit test in `cli/tri/src/fpga.rs` asserting `XadcContext::to_pvt_context` rounds the captured live values (41.4422 °C → 41, 1.00049 V → 1000 mV, 1.80688 V → 1807 mV) correctly.

### Task 4 — Competitor and defect refresh
- [ ] Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` date/header and W434 note.
- [ ] Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W434 triage entry: 7 residual failures remain; master-merge deferred; no new narrow defect.

### Task 5 — Documentation
- [ ] Update `fpga/HARDWARE_SSOT.md` §3.6 with the W434 live XADC validation recipe and the synthetic-CCLK proof-of-pipeline note.

### Task 6 — Close-out artifacts
- [ ] Write `docs/reports/WAVE_LOOP_434_REPORT.md`.
- [ ] Write `docs/reports/FPGA_LOOP_EVIDENCE_W434_2026-07-01.md`.
- [ ] Write `docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md` with three variants for W435.
- [ ] Update `docs/NOW.md` and `.trinity/current-issue.md` for W435.
- [ ] Create GitHub issue #1397 and branch `wave-loop-435`.

### Task 7 — Verification
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri --bin tri fpga::` passes.
- [ ] `./scripts/tri test` passes with the documented 7 pre-existing gen-verilog yosys smoke failures.

---

## 5. Definition of done

- [ ] Variant B acceptance criteria met: live XADC validated in PVT envelope; at least one `measured-to-lean` theorem generated from the live XADC context; W433 quantified theorem referenced in the formal library for the live point.
- [ ] New unit test for live XADC → PVT context rounding passes.
- [ ] Competitor snapshot and gen-verilog baseline updated.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri --bin tri fpga::` passes.
- [ ] `./scripts/tri test` passes with documented 7 residual failures.
- [ ] Close-out report and W435 cooperation variants written.
- [ ] Issue/branch for W435 created.

---

*φ² + φ⁻² = 3 | TRINITY*
