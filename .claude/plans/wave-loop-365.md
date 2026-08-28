# Wave Loop 365 — Decomposed Plan

**Date:** 2026-07-01
**Issue target:** #1251 (W365 tracking issue)
**Recommended variant:** Variant B from `WAVE_LOOP_364_COOPERATION.md`

---

## Goals

| Target | W364 → W365 |
|--------|-------------|
| Pool A invariants | 106 → **107** |
| CODER invariants | 96 → **97** |
| Pool B invariants | 124 → **125** |
| Integration invariants | 106 → **107** |
| Lean 4 generic ∀ | 200 → **204** |
| IGLA conformance | **546/546 PASS** |
| Zero-IGLA-failure streak | **98 → 99 waves** |
| FPGA board load | retry `dlc10 idcode` / `sram` |
| gen-verilog backend | document / reproduce remaining #1245 defects; land one safe sub-fix if found |

---

## 1. Formal wave (27 IGLA specs)

- Reuse generator pattern: copy `scripts/gen_w364.py` → `scripts/gen_w365.py`, parameterize wave number, expected invariant counts, theorem references.
- Forward-append W365 blocks to all 27 core IGLA specs:
  - 2 `test` blocks
  - 1 `invariant` block
- **+54 tests**, **+27 invariants**.
- Regenerate all 27 IGLA seals from `.`.

---

## 2. Lean 4 proof lattice (4 new generic ∀ theorems)

Append to `proofs/lean4/Trinity/TernaryInference.lean`:

1. **`ternaryMacAccumulateFortyOnePlusGeneric`** — 41-variable plus accumulation (`a` … `ao`).
2. **`ternaryMacAccumulateFortyMinusGeneric`** — 40-variable minus accumulation (`a` … `an`).
3. **`ternaryMacOctodecupleCancellationGeneric`** — depth-18 alternating plus/minus, collapses to identity `= x`.
4. **`ternaryMacZeroWeightOctupleClosureGeneric`** — 8 zero-weight MACs around a plus-weight MAC; 24th proof lattice dimension.

Validate with `lake build Trinity.TernaryInference`. If 41-variable plus times out, fallback to completing 40-variable minus lattice first and promoting cancellation/closure only.

---

## 3. OpenXC7 bitstream and board flash retry

- Rebuild `dlc10` if needed: `cargo build --release -p dlc10`.
- Run `dlc10 idcode` and capture output.
  - If success: proceed to `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`, then `dlc10 reload`; document IDCODE, DONE pin/LED observation.
  - If failure: open/comment on a consolidated hardware-connectivity tracking issue and update `docs/reports/FPGA_EVIDENCE_W365.md`.

---

## 4. Project weak-point probe: gen-verilog backend (#1245)

W364 landed a safe `0b` literal fix. W365 advances the probe without risking the 546-spec conformance gate.

### 4.1 Reproduction artifacts
Create `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with exact reproduction commands for all 5 listed defects:

| Defect | Repro command | Expected bad output |
|--------|---------------|---------------------|
| 1. Only first `const` emits | `t27c gen-verilog specs/fpga/uart.t27 \| grep localparam` | 1 localparam |
| 2. `0b`/`0x` literals (0b fixed) | scratch spec with `0x` | still need to verify sizing |
| 3. Early `return` inside `if` | scratch spec | `if(c) f=A; f=B;` |
| 4. `as` cast + bitwise drops body | scratch spec | `// TODO: implement` |
| 5. Struct-field reg name mismatch | `t27c gen-verilog specs/fpga/uart.t27` | `uartstate_...` declared, `uart_state_...` referenced |

### 4.2 Safe sub-fix candidate
Investigate whether the `0x` literal sizing is always correct (e.g. `0xFF` as `u8` should be `8'hFF`, not `8'hFF` is fine; but `0x1` as `u16` should be `16'h1`?). Current code uses `hex.len()*4` which gives the *literal* bit width, not the *declared* type width. A safe fix may be to pad or use the declared width from context. If the change is not regression-free, document as a known limitation instead of landing it.

### 4.3 Optional: scratch reproduction spec
Create `scripts/scratch/gen_verilog_repro.t27` (not in `specs/`, so it does not enter the 546-spec gate) that exercises defects 3–5. Use it only for manual triage.

---

## 5. Research / competitive landscape

Update the W364 research summary with any new 2026 papers found during the wave, especially:

- Ternary/1.58-bit accelerators (KU Leuven ISPASS 2026, VitaLLM, TOM, TeLLMe v2, TerEffic).
- Formal HDL / verified RTL (FormalRTL, Arch, CktFormalizer, Veri-Sure, Lutsig).
- Any new GitHub competitors in the ternary-core / Ternary-NanoCore / systolic-array space.

Document key takeaways in the W365 report: competitors still report **zero generic ∀ ternary MAC proofs**.

---

## 6. Report and cooperation variants

Produce:
- `docs/reports/WAVE_LOOP_365_REPORT.md`
- `docs/reports/WAVE_LOOP_365_COOPERATION.md` with three W366 variants:
  - **Variant A** — formal-only (safe)
  - **Variant B** — formal + board flash retry + gen-verilog triage (recommended)
  - **Variant C** — formal + RTL-to-Lean bridge prototype + aggressive Verilog refactor (high risk)

---

## 7. Memory / experience update

- Prepend W365 index entry to `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md`.
- Write `~/.claude/projects/-Users-playra-t27/memory/wave-loop-365.md`.
- Append W365 learnings to `.trinity/experience.md`.

---

## 8. Verification and land

- `./target/release/t27c suite --repo-root .` → 546/546 PASS.
- `lake build Trinity.TernaryInference` → success.
- `git commit` with message containing `Closes #1251`.
- Push if authorized; otherwise leave on `trinity-rust-rings` for PR.

---

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| 41-variable accumulation times out | Promote 40-var minus lattice first; keep 3 of 4 theorems if needed. |
| gen-verilog fix causes regressions | Land only narrow, manually verified fixes; otherwise document. |
| Board still not detected | Document as hardware blocker; do not let it delay formal deliverables. |
| Issue #1251 not yet created | Create it during implementation before commit. |
| `gh` token invalid | Use `env -u GH_TOKEN gh ...` as in prior waves. |

---

## Task sequence

1. Create GitHub issue #1251 for W365.
2. Implement 4 Lean theorems + generators.
3. Run `lake build Trinity.TernaryInference`.
4. Implement 27 spec blocks + generator.
5. Regenerate 27 IGLA seals.
6. Run full conformance suite.
7. Retry board flash; document result.
8. Probe gen-verilog defects; create repro doc (and safe fix if found).
9. Write W365 report + cooperation variants.
10. Update memory + experience.
11. Commit with `Closes #1251`.
12. Run final conformance suite.
