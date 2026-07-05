# Wave Loop 431 Decomposed Plan

**Issue:** #1389  
**Branch:** `wave-loop-431`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. OBSERVE summary

- We are on `wave-loop-431`, issue `#1389`, created during W430 close-out.
- W430 added live XADC readout (`tri fpga read-xadc`) and the formal PVT-envelope
  bridge (`xadc_operating_point_envelope_implies_worst_case_bound`).
- The physical bench is still **partially blocked**:
  - P12 CCLK probe is unwired, so real CCLK capture for OSCFSEL 6/7 is impossible.
  - No relay/remote-power gate, so automated cold-POR sweeps still require manual
    power cycles.
  - This autonomous session has no physical operator to perform power cycles or
    capture waveforms, and no external CSV/VCD captures are present in the repo.
- The working Digilent HS2 cable and `openFPGALoader --read-xadc` exist, but
  cannot be exercised hands-free here.
- Therefore the only shippable path this session is **Variant C (formal/tooling
  fallback)**, with a tooling side-car that makes the next physical Variant B/A
  run easier.

---

## 2. Weak points

1. **XADC data shape mismatch between Rust and Lean.** The Rust `XadcContext`
   stores `temp_c: f64` and `vccint_v: f64`, while the Lean `XadcOperatingPoint`
   uses `Int` °C and `Nat` mV. There is no canonical conversion helper yet, so a
   measured XADC JSON value cannot be fed directly into the formal model.
2. **No decidability/computability for the XADC envelope.** The current
   `xadc_operating_point_within_envelope` returns a `Prop`; automation cannot
   decide it on a concrete JSON value without manual `simp`/`norm_num`.
3. **No theorem closes the measured-raw-ns + measured-XADC → transaction OK loop.**
   W430 proved that an in-envelope point is bounded by the worst-case corner, but
   did not connect that to `transaction_satisfies_flash_spec` for a concrete
   raw-ns capture.
4. **`measured-to-lean --json` summary is still thin.** It lacks actionable
   fields such as `flash_min_half_period_ns`, `margin_ns`, and a closed
   `recommendation` vocabulary, which downstream CI needs to react without
   parsing generated Lean.
5. **P12 / relay blockers remain.** No amount of software fixes them; the next
   waves must explicitly call them out as hardware prerequisites.
6. **Gen-verilog #1245 residual 7 failures** are still tied to tuple-return,
   `let` destructuring, ROM arrays, and CORDIC. None is a narrow single-wave fix.
7. **Competitive pressure is rising on both Lean-native HDL and ternary compute.**
   Sparkle/Verilean keeps expanding; new ternary FPGA engines (TernaryCore,
   ternfpga) validate the ternary direction but also raise the bar for t27 to
   prove physical boot evidence.

---

## 3. Competitor snapshot (July 2026)

- **Sparkle / Verilean** (`Verilean/sparkle`): last public push 2026-07-03. Headline
  2026 signals remain PR #66 (IP.Net + compiler perf) and the RV32 divider proof
  (commit `9c7809c`, June 25). Still the closest Lean-native competitor.
- **Clash**: 1.11.0 is only a Hackage candidate; latest official release is
  1.10.0 (April 2026). No new verification headline.
- **Chisel / CIRCT / firtool**: Chisel 7.13.0 shipped June 1 2026 with firtool
  1.149.0. CIRCT's Verif dialect, LTL dialect, and `circt-bmc` bounded model
  checker continue to mature; firtool 1.143.0 (March 2026) added BTOR2 backend
  support for `verif.formal`. No evidence of a more recent Chisel 7.14 / firtool
  1.152 release.
- **CktFormalizer** (arXiv 2605.07782, May 2026): LLM-to-circuit
  autoformalization in Lean 4, reports 95–100% synthesis/P&R success and 35%
  area / 30% power reduction via closed-loop PPA optimization. Validates Lean 4
  as a hardware proof backend.
- **Aria-HDL** (`zeta1999/fpga-meta-compiler-public`): a 2026 WIP "FPGA
  meta-compiler" that emits Lean 4 proof obligations among ten backends. No new
  July signal.
- **TernaryCore** (`shepherdscientific/ternarycore`, April 2026): BitNet b1.58
  ternary inference accelerator, simulation-verified (31/31 tests), targeting
  Arty A7-100T.
- **ternfpga** (`Neumann-Labs/ternfpga`, June 2026): multiplier-free ternary LLM
  engine on Arty A7-35T, claims ~2.3× lower energy-per-token than RTX 3060.
- **KU Leuven MICAS / TeLLMe v2**: edge-to-datacenter ternary LLM FPGA
  accelerators, 25 tok/s decode on Kria KV260, 12,700 tok/s on Alveo U280.

Strategic implication: t27's unique intersection remains **Lean 4 native proof +
ternary/balanced-trit compute + spec-first sealed `*.t27 → gen/` pipeline +
physical boot-evidence instrumentation**. W431 should harden the last two
(spec-first traceability and the XADC/boot-evidence bridge) while the bench is
blocked.

---

## 4. Variant selection

**Primary: Variant C — formal/tooling fallback.**

The physical prerequisites for A/B are not met in this session. W431 advances
by:

1. Closing the Rust/Lean XADC data-shape gap.
2. Making the XADC envelope computable.
3. Proving the measured raw-ns + measured XADC → transaction OK implication.
4. Thickening the `measured-to-lean --json` summary with `flash_min_half_period_ns`,
   `margin_ns`, and a closed `recommendation`.
5. Refreshing the competitor snapshot.
6. Explicitly deferring gen-verilog #1245 and updating the defects doc.

---

## 5. Decomposed implementation steps

### 5.1 Rust/Lean XADC data bridge

- In `cli/tri/src/fpga.rs` add `xadc_context_to_operating_point`:
  - Convert `temp_c: f64` → rounded `i64` °C.
  - Convert `vccint_v: f64` → rounded `u64` mV.
  - Convert `vccaux_v: f64` → rounded `u64` mV.
  - Accept a `ProcessCorner` argument (default `Ss`) because XADC cannot measure
    process.
- Add a unit test for the conversion and for rejecting out-of-range values.

### 5.2 Computable XADC envelope in Lean

- In `proofs/lean4/Trinity/TernaryFPGABoot.lean` (inside `namespace BitstreamConfig`):
  - Derive `DecidableEq` and add `xadc_operating_point_within_envelope_dec` that
    returns a `Bool` and is provably equivalent to the `Prop` version.
  - Add `xadc_operating_point_within_envelope_dec_eq` theorem.
  - Add `xadc_live_operating_point_example` theorem: a point at 43 °C, 1000 mV,
    1806 mV, `ss` corner is inside the envelope (decided by `decide`).
  - Add `xadc_operating_point_envelope_implies_measured_raw_ns_ok`: given an
    in-envelope operating point with a slow corner, if a raw-ns capture satisfies
    `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` for that point,
    then it also satisfies it for the worst-case context, and therefore the
    resulting SPI transaction is OK.

### 5.3 Harden `measured-to-lean --json` summary

- Extend `build_measured_to_lean_summary` in `cli/tri/src/fpga.rs` to compute and
  include:
  - `flash_min_half_period_ns`: from `n25q128_min_sck_half_ns_pvt` if a PVT
    context is supplied, otherwise the nominal `N25Q128_MIN_SCK_LOW_NS`.
  - `margin_ns`: `min(low_ns, high_ns) - flash_min_half_period_ns` for raw-ns,
    or computed from the measured duty for frequency-mode.
  - `recommendation`: a closed vocabulary (`"in_spec"`, `"out_of_spec"`,
    `"needs_pvt_context"`) based on the margin.
- Update existing unit tests and add new ones for the new fields.
- Update the dispatch in `measured_to_lean` to print the summary when `--json`
  is set.

### 5.4 Competitor refresh

- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md`:
  - Refresh date to W431.
  - Add the July 2026 signals above.
  - Add a note about the new ternary FPGA projects (TernaryCore, ternfpga) as
    validation of t27's ternary direction and as new competition.

### 5.5 Gen-verilog triage

- Re-run `./scripts/tri test` at start of implementation and confirm the same 7
  failures.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W431 triage
  decision: no safe sub-fix this wave; explicitly deferred.

### 5.6 Documentation

- Update `fpga/HARDWARE_SSOT.md` §9.6 with a short note on how a live XADC
  reading is converted to the PVT context and consumed by `measured-to-lean`.
- Create `docs/reports/FPGA_LOOP_EVIDENCE_W431_2026-07-01.md` documenting the
  formal XADC bridge as the wave's evidence artifact (since physical bench work
  is blocked).

### 5.7 Verification

- `cargo test --bin tri fpga::`: must pass.
- `lake build Trinity.TernaryFPGABoot`: must pass.
- `./scripts/tri test`: must pass except the 7 pre-existing #1245 failures.

### 5.8 Close-out

- Write `docs/reports/WAVE_LOOP_431_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W432_2026-07-01.md` with Variant A/B/C
  for W432.
- Create GitHub issue #1390? Wait #1389 is current; create #1390 for W432 and
  branch `wave-loop-432`.
- Update `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`,
  and persistent memory.

---

## 6. Acceptance criteria

- AC-C1: At least one new XADC/PVT theorem is added and builds with `lake build`.
- AC-C2: `measured-to-lean --json` summary includes `flash_min_half_period_ns`,
  `margin_ns`, and `recommendation`.
- AC-C3: One safe gen-verilog sub-fix lands without increasing the 7-failure
  yosys smoke count, or is explicitly deferred if unsafe.
- AC-C4: Competitor snapshot is updated with July 2026 signals.
- AC-C5: Close-out report and W432 cooperation variants are written; issue/branch
  for W432 are created.

---

## 7. Files to touch

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W431_2026-07-01.md` (new)
- `docs/reports/WAVE_LOOP_431_REPORT.md` (new)
- `docs/reports/FPGA_LOOP_COOPERATION_W432_2026-07-01.md` (new)
- `.trinity/current-issue.md`
- `docs/NOW.md`
- `.trinity/experience.md`
- Persistent memory: `wave-loop-431.md` + `MEMORY.md` index

---

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Adding Lean decidability breaks the existing `Prop` version | Keep the `Prop` definition and prove equivalence; do not replace it. |
| `margin_ns` computation differs between raw-ns and frequency modes | Separate code paths with clear unit tests for each. |
| Gen-verilog fix turns out unsafe | Stop and defer; update defects doc. |
| Competitor refresh lacks fresh July signals beyond W430 | Use the web-search results and mark older sources as unchanged. |
| Close-out issue numbering collides | Check GitHub before creating; W430 closed #1388, W431 is #1389, so W432 is #1390. |

---

*φ² + φ⁻² = 3 | TRINITY*
