# Wave Loop 429 Implementation Plan

**Issue:** #1385  
**Branch:** `wave-loop-429`  
**Date:** 2026-07-06  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Variant choice

**Execute Variant C** (formal / tooling / competitor refresh).

**Rationale:**
- P12 CCLK probe is still unwired.
- No relay / remote-power gate; `cold-por` is `MOCK`-only.
- Xilinx DLC10 cable still missing; only Digilent HS2 is connected.
- Only OSCFSEL 0–5 bitstreams exist; no 6/7 variants.
- No external CSV/VCD captures were provided.
- The board is reachable via HS2, but a real XADC readout implementation would be a multi-file JTAG feature and risks destabilizing the wave-loop line; it is better left for a future Variant B once the hardware state changes.

Variant C keeps W429 bounded, shippable, and aligned with the W428 baseline.

---

## Weak points investigated

| Weak point | Status | Risk |
|---|---|---|
| P12 CCLK probe unwired | unchanged | Blocks Variant A |
| Relay / remote-power gate absent | unchanged | Blocks true cold-POR automation |
| DLC10 cable missing | unchanged | Blocks `dlc10` path; HS2 + openFPGALoader still works |
| OSCFSEL 6/7 bitstreams missing | unchanged | Blocks Variant A capture for fastest variants |
| XADC readout placeholder | unchanged | Variant B possible but not in this wave |
| PVT coefficients are conservative placeholders | unchanged | Model is falsifiable but not datasheet-accurate |
| Gen-verilog #1245 residual 7 failures | unchanged | Tied to tuple-return / `let` destructuring / ROM arrays / CORDIC; unsafe for narrow sub-fix |

---

## Competitor snapshot (W429 boundary)

- **Sparkle / Verilean** — last public push July 3 2026; no post-July-5 commits/PRs indexed. PR #66 (IP.Net + compiler perf) and PR #65 (RV32 divider proof) remain the headline 2026 signals. Still the closest Lean-native competitor.
- **Clash** — 1.11.0 Hackage candidate uploaded July 4 2026; not yet an official release. Latest official remains 1.10.0 (April 2026).
- **Chisel / FIRRTL / CIRCT** — Chisel 7.13.0 (June 1 2026) bundles firtool 1.149.0. Standalone firtool 1.152.0 released July 4 2026, so the bundled tool trails the standalone release.
- **Bluespec BSC 2026.01** (May 2026); 2026.07 release expected but not published.
- **SpinalHDL** — v1.14.0 (Feb 2026), patch 1.14.2 (May 2026); no July release.
- **Emerging signals:** CktFormalizer (Lean 4 autoformalization paper), Aria-HDL / fpga-meta-compiler-public (Rust → Lean4/SBY), TernaryCore, BitNet-RISCV-Multicore, MINRES RISC-V Tournament.

Strategic bottom line: Sparkle remains the only credible direct competitor in the same design space, but the ternary + spec-first sealed pipeline + physical boot-evidence triangle is still unoccupied by anyone else.

---

## Decomposed implementation

### Step 1 — Extend the unified OSCFSEL theorem family in Lean 4

**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Add two quantified theorems that link raw-ns measurements to the W428 unified OSCFSEL theorems:

1. `cclk_variant_raw_ns_worstcase_pvt_satisfies_flash_spec (oscfsel : Nat) (h : oscfsel ≤ 7)`  
   For any documented OSCFSEL selection, a raw capture whose period equals `cclk_period_ns oscfsel` and whose low/high times equal half the period satisfies the worst-case PVT-aware raw-ns flash predicate.

2. `cclk_variant_raw_ns_worstcase_pvt_implies_transaction_ok (oscfsel : Nat) (h : oscfsel ≤ 7) (bits : Nat)`  
   The same raw capture produces a flash-spec-compliant SPI transaction under the worst-case PVT corner.

Both are proved by `interval_cases oscfsel <;> decide` and by applying the existing implication theorem with the worst-case context passed explicitly.

**Why:** W428 unified the `freq_hz/duty_pct` view; W429 closes the raw-ns view into the same quantified theorem family. This makes `measured-to-lean --raw-ns` theorems traceable to a single generic OSCFSEL result.

### Step 2 — Machine-readable `tri fpga measured-to-lean --json`

**File:** `cli/tri/src/fpga.rs`

- Add `json: bool` to `FpgaCmd::MeasuredToLean`.
- Plumb the flag through `measured_to_lean` dispatch.
- When `--json` is set, emit a single JSON object to stdout instead of free-form text:
  - `source`
  - `output_path` (or `null`)
  - `theorem_base`
  - `predicate` (e.g. `measured_cclk_with_pvt_satisfies_flash_spec`)
  - `pvt_context` (or `null`)
  - `validated` (bool)
- Keep the existing human-readable path unchanged for backward compatibility.
- Add unit tests:
  - `test_measured_to_lean_json_summary`
  - `test_measured_to_lean_json_summary_with_pvt_context`

**Why:** `measured-to-lean` is the bridge from bench captures to generated Lean proofs. Making its summary machine-readable lets downstream CI/dashboards consume the theorem metadata without parsing prose.

### Step 3 — Gen-verilog #1245 triage / deferral update

**File:** `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

- Add "Triage decision for W429" section.
- Confirm the same 7 yosys smoke failures from W428.
- State that no new narrow, regression-free subclass appeared.
- Keep the resolution plan: dedicated master merge/rebase wave after the FPGA boot-evidence line is no longer the primary focus.

**Why:** Documenting the deferral prevents re-investigation each wave and satisfies the weak-point investigation requirement.

### Step 4 — Refresh competitor snapshot

**File:** `docs/reports/T27_VS_FORMAL_HDL_2026.md`

- Update date to 2026-07-06 (W429 boundary).
- Add post-July-5 findings:
  - Sparkle: no new indexed commits/PRs after July 5; repo-level last push July 3.
  - Clash 1.11.0 candidate still unreleased.
  - firtool 1.152.0 (July 4) remains latest; Chisel 7.13.0 bundles 1.149.0.
  - Bluespec/SpinalHDL no July releases.
- Add or update "Emerging signals" with Aria-HDL source `zeta1999/fpga-meta-compiler-public`, MINRES RISC-V Tournament.
- Adjust recommendation section to mention W429 progress.

### Step 5 — Close-out artifacts

Create:
- `docs/reports/WAVE_LOOP_429_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W429_2026-07-06.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-06.md` (three variants)

Update:
- `docs/NOW.md` — W429 close-out / W430 setup.
- `.trinity/experience.md` — W429 learnings.
- `.trinity/current-issue.md` — mark acceptance criteria and PR/issue numbers.

### Step 6 — GitHub state

- Commit all changes to `wave-loop-429`.
- Push branch.
- Open PR `#?` with body `Closes #1385`.
- Create issue `#?` for Wave Loop 430 and branch `wave-loop-430`.
- Update memory entry and `MEMORY.md` index.

---

## Acceptance criteria

### AC-C1 (new theorem)
- `lake build Trinity.TernaryFPGABoot` passes with the two new raw-ns OSCFSEL theorems.

### AC-C2 (gen-verilog)
- `./scripts/tri test` gen-verilog-yosys-smoke reports the same 7 pre-existing failures; no increase.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` contains W429 deferral rationale.

### AC-C3 (CLI JSON)
- `cargo test -p tri` passes including new `measured-to-lean --json` tests.
- `tri fpga measured-to-lean --file capture.json --json` emits valid JSON.

### AC-C4 (competitors)
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` is updated for the W429 boundary with new signals.

### Invariants
- `./scripts/tri test` parse/typecheck/gen-zig/gen-rust/gen-c/seal-verify/fixed-point/FPGA-smoke pass.
- `cargo test -p tri` passes.
- `lake build Trinity.TernaryFPGABoot` passes.

---

## Verification plan

1. After Lean edits: `lake build Trinity.TernaryFPGABoot`.
2. After Rust edits: `cargo test -p tri`.
3. After all edits: `./scripts/tri test` and inspect gen-verilog-yosys-smoke count.
4. Manual CLI check: `tri fpga measured-to-lean --file ... --json`.
5. Run L1/L3 git hooks on commit (already enforced by pre-commit).
