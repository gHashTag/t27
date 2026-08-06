# Wave Loop 430 Decomposed Plan

**Issue:** #1388  
**Branch:** `wave-loop-430`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. OBSERVE summary

- We are on `wave-loop-430`, issue `#1388`.
- W429 closed with PR #1387; next wave should continue the FPGA boot-evidence line.
- The bench is **partially unblocked**: a Digilent HS2 FTDI cable is connected to
  the XC7A200T Wukong V1 board (`idcode 0x03636093`). `openFPGALoader -c digilent_hs2`
  works for SRAM load, STAT read, and **XADC readout**.
- Live XADC probe returned:
  ```text
  temp: 42.8206 °C
  vccint: 1.00049 V
  vccaux: 1.80615 V
  ```
- **P12 CCLK probe is still unwired**, so real CCLK capture (Variant A) is not
  possible today.
- No external CSV/VCD captures exist in the repo, so pure import-only Variant B
  is also not directly available.
- Therefore the default selection rule points to **Variant B via real XADC
  readout**, with a small formal-extension side-car so the wave also advances
  the PVT-aware theorem library.

---

## 2. Weak points

1. **XADC readout is not wired into `tri fpga`.** `boot-log`, `cold-por`, and
   `cclk-sweep` still emit `"source": "not_read"` for `xadc`. The live operating
   point cannot yet flow into generated Lean theorems or JSON logs.
2. **P12 CCLK probe unwired.** Real CCLK capture for OSCFSEL 6/7 remains blocked.
3. **SPI flash cold-POR automation needs a relay gate.** Without it, the physical
   OSCFSEL sweep still requires manual power cycles.
4. **Gen-verilog #1245 residual 7 failures** are tied to tuple-return / `let`
   destructuring / ROM arrays / CORDIC. They are not safe as a single-wave
   regression-free sub-fix.
5. **Competitive pressure from Lean-native HDL** (Sparkle/Verilean, CktFormalizer,
   Aria-HDL) is rising; the ternary + physical-evidence angle is the key
   differentiator.
6. **PVT derating coefficients remain conservative placeholders**, not Micron
   datasheet values. Any new XADC-derived operating point must be explicitly
   framed as "within the modeled envelope" rather than "exact datasheet".

---

## 3. Competitor snapshot (July 2026)

- **Sparkle / Verilean**: last public push 2026-07-03; PR #66 (IP.Net + compiler
  perf) and PR #65 (RV32 divider proof) remain the headline 2026 signals. Still
  the closest Lean-native competitor.
- **Clash**: 1.11.0 is only a Hackage candidate; latest release remains 1.10.0
  (April 2026). No new formal headline.
- **Chisel / CIRCT / firtool**: Chisel 7.13.0 (June 2026) is the latest release;
  firtool 1.152.0 (July 4 2026) is the latest available. No 7.14.0 / 1.153
  evidence yet.
- **CktFormalizer**: arXiv 2605.07782 (May 2026) — LLM-to-circuit autoformalization
  in Lean 4, 95–100% synthesis/P&R success, 35% area / 30% power reduction with
  closed-loop PPA optimization. Validates Lean 4 as a hardware proof backend.
- **TernaryCore**: `shepherdscientific/ternarycore` (April–May 2026) — BitNet b1.58
  ternary inference on FPGA, zero-DSP MAC/dot/GEMM, 31/31 simulation tests.
- **ternfpga**: `Neumann-Labs/ternfpga` (June 2026) — multiplier-free ternary LLM
  engine on Arty A7-35T, claims better energy-per-token than RTX 3060.
- **KU Leuven MICAS ternary-lut-dse** (April 2026) — Chisel RTL generator for
  LUT-based 1.58-bit LLM accelerators, validated in TSMC 16nm.
- **Aria-HDL**: no new July signal; keep watching.

Strategic implication: the Lean-native + ternary + physical-evidence triangle is
still the unique intersection. W430 should harden the physical-evidence loop by
consuming real XADC data.

---

## 4. Variant selection

**Primary: Variant B — real XADC readout.**

The board is reachable and `openFPGALoader --read-xadc` already returns live
data. Implementing a parser and wiring it into `tri fpga boot-log` /
`cold-por` / `cclk-sweep` advances the physical boot-evidence story without
needing the P12 probe or relay gate.

**Side-car: a small formal extension.**
Add a Lean lemma that any operating point with temp ≤ 85 °C, vccint ≥ 900 mV,
vccaux ≥ 1800 mV lies inside the modeled PVT rectangle, plus a theorem that the
real XADC values (≈43 °C, ≈1000 mV, ≈1806 mV) satisfy this rectangle. This gives
AC-C1-style coverage and links the live measurement to the PVT envelope
family.

**Gen-verilog #1245:** re-evaluate; if no narrow subclass is safe, explicitly
defer and update the defects doc.

---

## 5. Decomposed implementation steps

### 5.1 Parse `openFPGALoader --read-xadc` output
- Add `XadcContext` struct in `cli/tri/src/fpga.rs`:
  - `temp_c: f64`, `max_temp_c: f64`, `min_temp_c: f64`
  - `vccint_v: f64`, `max_vccint_v: f64`, `min_vccint_v: f64`
  - `vccaux_v: f64`, `max_vccaux_v: f64`, `min_vccaux_v: f64`
  - `raw: serde_json::Value`
- Add `read_xadc_via_openfpgaloader(cable: &str) -> Result<XadcContext>`.
  - Run `openFPGALoader -c <cable> --read-xadc`.
  - Parse the emitted pseudo-JSON with `serde_json` after a normalization pass
    (remove trailing commas, collapse whitespace). Fallback to regex if
    normalization fails.
- Add unit tests with captured sample output (the live probe above).

### 5.2 Wire live XADC into `tri fpga` commands
- Add `--xadc` flag to `FpgaCmd::BootLog`, `FpgaCmd::ColdPor`, and
  `FpgaCmd::CclkSweep` (default: false / auto-detect only when the flag is
  passed, to avoid breaking board-less CI).
- When `--xadc` is set and a Digilent cable is detected:
  - Call `read_xadc_via_openfpgaloader`.
  - Populate the `xadc` field of `BootLog` / `ColdPorLog` / `SweepLog` with
    `source: "xadc"` and live values.
- When `--xadc` is not set or the board is absent, keep the existing
  `"not_read"` / PVT-context fallback so board-less CI stays green.
- Add `test_read_xadc_context_parses_sample_output` and a test that a missing
  board yields an actionable error.

### 5.3 Formal bridge in Lean 4
- Add `pvt_operating_rectangle` predicate in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean`:
  - `temp_c ≤ PVT_TEMP_MAX_C`
  - `vccint_mv ≥ PVT_VCCINT_MIN_MV`
  - `vccaux_mv ≥ PVT_VCCAUX_MIN_MV`
- Add `xadc_operating_point_within_pvt_envelope` theorem: the real operating
  point (≈43 °C, ≈1000 mV, ≈1806 mV) satisfies the rectangle.
- Add `xadc_context_implies_pvt_envelope` theorem: any XADC context within the
  rectangle implies the worst-case PVT corner bounds the flash timing. This
  links the live measurement to the existing `measured_cclk_from_raw_ns_with_pvt`
  family.

### 5.4 Documentation and evidence
- Update `fpga/HARDWARE_SSOT.md` §3.6.19:
  - Add the canonical `tri fpga read-xadc` / `--xadc` recipe.
  - Document the captured sample values and the `--pvt-context` workflow.
- Create `docs/reports/FPGA_LOOP_EVIDENCE_W430_2026-07-01.md` with the live XADC
  capture and the JSON log artifact.

### 5.5 Competitor refresh
- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md`:
  - Refresh date to W430.
  - Add the July 2026 signals above.
  - Note that the live XADC readout further differentiates the physical-evidence
    loop.

### 5.6 Gen-verilog triage
- Re-run `./scripts/tri test` at start of wave and confirm the same 7 failures.
- If no narrow safe subclass appears, update
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with W430 triage decision.

### 5.7 Verification
- `cargo test --bin tri fpga::`: must pass.
- `lake build Trinity.TernaryFPGABoot`: must pass.
- `./scripts/tri test`: must pass except the 7 pre-existing #1245 failures.
- Manual: `tri fpga read-xadc` or `tri fpga boot-log ... --xadc` produces a JSON
  log with `xadc.source: "xadc"` and live values.

### 5.8 Close-out
- Write `docs/reports/WAVE_LOOP_430_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md` with Variant A/B/C
  for W431.
- Create GitHub issue #1389 and branch `wave-loop-431`.
- Update `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`, and
  persistent memory.

---

## 6. Acceptance criteria

- AC-B1: `tri fpga read-xadc` (or `--xadc` flag) returns live temp/vccint/vccaux
  from the board and emits `source: "xadc"`.
- AC-B2: `boot-log`, `cold-por`, and `cclk-sweep` can include live XADC context
  in their JSON logs.
- AC-B3: At least one new Lean theorem links the live/any XADC operating point
  to the PVT envelope and builds with `lake build`.
- AC-B4: The XADC recipe is documented in `fpga/HARDWARE_SSOT.md`.
- AC-C1-equivalent: the formal extension above satisfies the Variant C fallback
  theorem requirement.

---

## 7. Files to touch

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
- `docs/reports/WAVE_LOOP_430_REPORT.md` (new)
- `docs/reports/FPGA_LOOP_EVIDENCE_W430_2026-07-01.md` (new)
- `docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md` (new)
- `docs/NOW.md`
- `.trinity/current-issue.md`
- `.trinity/experience.md`

---

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| openFPGALoader `--read-xadc` output format changes | Parser uses normalization + regex fallback; unit test against captured sample. |
| Board not powered during CI | `--xadc` is opt-in; board-less CI uses existing placeholder path. |
| Live XADC values drift across runs | Evidence file records one representative capture; the formal theorem bounds the rectangle, not a single point. |
| No narrow gen-verilog fix | Explicitly defer and update defects doc; do not broaden scope. |

---

*φ² + φ⁻² = 3 | TRINITY*
