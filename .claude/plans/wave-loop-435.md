# Wave Loop 435 — Decomposed Plan

**Issue:** #1398  
**Branch:** `wave-loop-435`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### Physical bench
- **P12 is still unwired** to a logic-analyzer channel, so **Variant A** (real cold-POR CCLK capture) remains infeasible.
- **No relay / remote-power gate** exists, so automated cold-POR sweeps are still manual.
- **No DLC10 cable** (VID=0x03FD), so the in-repo `dlc10` driver cannot be used; the reachable cable is the Digilent FTDI (`0x0403:0x6014`), which works with `openFPGALoader`.
- The FPGA **is reachable** over JTAG and live XADC readout succeeds (`tri fpga read-xadc` returns temp≈41 °C, VCCINT≈1.00 V, VCCAUX≈1.81 V).

### Tooling gaps discovered this wave
- `tri fpga read-xadc` emits `temp_c` / `vccint_v` / `vccaux_v` as `f64`, but `tri fpga pvt-envelope --pvt-context` expects integer `temp_c` (`i64`) and `vccint_mv` / `vccaux_mv` (`u64`). The conversion (`XadcContext::to_pvt_context`) is implemented internally but **not exposed as a standalone CLI export**, so the user must round the values by hand or call the helper from Rust tests.
- `tri fpga measured-to-lean --json` summary does not yet include the **source operating point** (`temp_c`, `vccint_mv`, `vccaux_mv`, `process_corner`) even though the PVT context is used to generate the theorem. Downstream dashboards cannot correlate the theorem with the live silicon state without parsing the generated Lean snippet.
- No end-to-end integration test exercises `read-xadc → pvt-envelope → measured-to-lean` as a single pipeline. The W434 path was validated manually; W435 should lock it with a regression test.

### Strategic / competitive
- The 7 residual `gen-verilog` yosys smoke failures (#1245) are unchanged. A master-merge of the full fix set is still too risky for a wave whose primary goal is FPGA boot-evidence formalization.
- Competitor signals since W434:
  - **Sparkle / Verilean** remains the closest Lean-native threat. PR #66 is still open; PR #65 (RV32 divider proof) demonstrates IP-level depth; July 2026 Functional Matsuri talk positions Lean 4 as an RTL core.
  - **CIRCT / firtool 1.152.0** published July 2026; LTL/Verif dialects and ChiselTest formal compatibility layer keep advancing.
  - **Clash 1.11.0** is still a Hackage candidate; **Clash 1.10** (April 2026) remains the latest official release.
  - **Aria-HDL** Rust meta-compiler with `--emit-lean4` and `--emit-sby`; retiming + PCIe BAR test added.
  - **CktFormalizer** (arXiv 2605.07782) and ternary-compute projects (**TernaryCore**, **BitNet-RISCV-Multicore**) validate the ternary direction.
  - **Takahe** multi-radix synthesis supports `--radix 3` balanced ternary with `--equiv` formal equivalence checking, a new signal in the ternary hardware space.
  - **ternlang-hdl** Rust crate and **KULeuven-MICAS/ternary-lut-dse** (Chisel, ISPASS 2026) add more ternary-accelerator activity.

---

## 2. Competitor scan summary

| Competitor | Update for W435 | Implication for t27 |
|---|---|---|
| **Sparkle / Verilean** | PR #66 open, PR #65 divider proof closed; public talk July 11 2026; IP catalog growing. | Closest structural competitor; t27's differentiation is the sealed spec→bitstream loop + physical boot evidence. |
| **Clash** | 1.11.0 still a Hackage candidate (no final release); 1.10 remains latest official. Issue #3153 on verification operator translation still open. | Functional-HDL maturity but external proof; no physical evidence loop. |
| **Chisel / FIRRTL / CIRCT** | `firtool-1.152.0` published July 2026; LTL/Verif dialects and ChiselTest formal compatibility layer advancing; `circt-bmc` / `circt-lec` maturing. | Industry adoption; formal reasoning still RTL/SVA/external, not source-level dependent types. |
| **Aria-HDL** | Rust meta-compiler with `--emit-lean4` and `--emit-sby`; retiming + PCIe BAR test added. | Validates spec→proof→bitstream pipelines but no ternary focus or sealed hashes. |
| **CktFormalizer** | arXiv 2605.07782; LLM-to-circuit autoformalization in Lean 4 with Yosys/OpenROAD flow. Claims 95–100% backend success and closed-loop PPA optimization. | Another signal that **Lean 4 as HDL proof backend** is crowded. |
| **TernaryCore / BitNet-RISCV-Multicore** | Ternary inference and multicore RISC-V ternary PEs simulating; no formal proofs yet. | Confirms ternary compute hardware is visible; t27 must keep formal ternary IP ahead. |
| **Takahe** | `--radix 3` balanced ternary synthesis with `--equiv` formal equivalence (≤24 inputs exhaustive). | New ternary formal-hardware signal; watch for scaling claims. |
| **ternlang-hdl / KULeuven ternary-lut-dse** | Rust ternary Verilog/VHDL lowering and Chisel ternary matmul accelerator accepted at ISPASS 2026. | More evidence that ternary hardware tooling is gaining momentum. |

---

## 3. Variant selection

**Selected: Variant B** — board is reachable over JTAG, live XADC readout works, but P12 / relay are still blocked, so real CCLK capture is not possible. This wave hardens the live XADC → PVT context → `measured-to-lean` pipeline and extends the formal library with a synthetic CCLK coverage matrix for OSCFSEL 0..7 under the live operating point.

If P12 or the relay gate becomes available during the wave, switch to **Variant A** immediately.

---

## 4. Decomposed tasks

### Task 1 — Expose `XadcContext → PvtContext` export from `tri fpga read-xadc`
- [ ] Add `--process-corner <corner>` to `tri fpga read-xadc` (default `ss`).
- [ ] Add `--to-pvt-context <file>` to write the rounded `PvtContext` JSON directly.
- [ ] Keep the existing full XADC JSON output on stdout; the new flags are additive.
- [ ] Validate the emitted JSON parses as `PvtContext`.

### Task 2 — Extend `measured-to-lean --json` summary with source operating point
- [ ] Add `operating_point` field to the summary when a PVT context is present:
  ```json
  {
    "operating_point": {
      "source": "pvt_context_file" | "xadc",
      "temp_c": 41,
      "vccint_mv": 1000,
      "vccaux_mv": 1807,
      "process_corner": "ss"
    }
  }
  ```
- [ ] For `--pvt-worstcase`, source is `"worstcase"`; for `--pvt-context <file>`, source is `"pvt_context_file"`.
- [ ] Update `build_measured_to_lean_summary` signature and unit tests.

### Task 3 — Integration test for end-to-end live XADC → theorem pipeline
- [ ] Add a test that:
  1. Constructs an `XadcContext` matching the W434 live capture.
  2. Rounds it to `PvtContext` via `to_pvt_context(ProcessCorner::Ss)`.
  3. Writes the PVT context to a temp JSON file.
  4. Creates a synthetic raw-ns CCLK fixture (40/20/20 ns).
  5. Calls `measured_to_lean(..., raw_ns=true, pvt_context=<file>, validate=true, standalone=true, json=true)`.
  6. Asserts the summary `recommendation` is `"in_spec"`, `margin_ns >= 0`, and the generated Lean snippet builds in a standalone `lake` package.
- [ ] Use a temp directory and clean up afterwards.

### Task 4 — Generate synthetic OSCFSEL 0..7 theorem matrix from live XADC context
- [ ] In `proofs/lean4/Trinity/TernaryFPGABoot.lean`, add:
  - `XADC_LIVE_W434_OPERATING_POINT` (reuse from W434).
  - `xadc_live_w434_all_oscfsel_raw_ns_pvt_satisfies_flash_spec` — quantified theorem over OSCFSEL 0..7 using `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt`.
  - Per-OSCFSEL concrete theorems `xadc_live_w434_oscfsel_0_raw_ns_pvt_satisfies_flash_spec` ... `xadc_live_w434_oscfsel_7_raw_ns_pvt_satisfies_flash_spec`.
  - Matching transaction theorems `xadc_live_w434_oscfsel_N_transaction_ok`.
- [ ] These are `decide`-cheap because they reuse the quantified bridge.

### Task 5 — Add computable combined OSCFSEL + XADC envelope check
- [ ] Add `cclk_variant_and_xadc_envelope_check (oscfsel : Nat) (pt : XadcOperatingPoint) : Bool`.
- [ ] Prove equivalence with `oscfsel ≤ 7 ∧ xadc_operating_point_within_envelope pt`.
- [ ] Add a theorem linking the combined check to `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` for any in-envelope point and documented OSCFSEL.

### Task 6 — Documentation and baseline refresh
- [ ] Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` date/header and W435 note.
- [ ] Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W435 triage entry: 7 residual failures remain; master-merge deferred; no new narrow defect.
- [ ] Extend `fpga/HARDWARE_SSOT.md` §9.6.2 with the `tri fpga read-xadc --to-pvt-context` recipe and the OSCFSEL 0..7 synthetic theorem matrix.

### Task 7 — Close-out artifacts
- [ ] Write `docs/reports/WAVE_LOOP_435_REPORT.md`.
- [ ] Write `docs/reports/FPGA_LOOP_EVIDENCE_W435_2026-07-01.md`.
- [ ] Write `docs/reports/FPGA_LOOP_COOPERATION_W436_2026-07-01.md` with three variants for W436.
- [ ] Update `docs/NOW.md` and `.trinity/current-issue.md` for W436.
- [ ] Create GitHub issue #1401 and branch `wave-loop-436`.

### Task 8 — Verification
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri --bin tri fpga::` passes.
- [ ] `./scripts/tri test` passes with the documented 7 pre-existing gen-verilog yosys smoke failures.

---

## 5. Definition of done

- [ ] `tri fpga read-xadc` can emit a valid `--pvt-context` JSON directly.
- [ ] `measured-to-lean --json` summary includes the source operating point.
- [ ] At least one new integration test exercises the end-to-end live XADC → theorem pipeline.
- [ ] `TernaryFPGABoot.lean` contains the OSCFSEL 0..7 synthetic theorem matrix under the live W434 XADC point and the combined computable envelope check.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `cargo test -p tri --bin tri fpga::` passes.
- [ ] `./scripts/tri test` passes with documented 7 residual failures.
- [ ] Competitor snapshot and gen-verilog baseline updated.
- [ ] Close-out report and W436 cooperation variants written.
- [ ] Issue/branch for W436 created.

---

*φ² + φ⁻² = 3 | TRINITY*
