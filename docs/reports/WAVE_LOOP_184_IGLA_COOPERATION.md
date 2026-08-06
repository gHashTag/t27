# Wave Loop 184 — Cooperation Variants for W185

**Date:** 2026-06-18
**Next Pool:** A — {rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm}
**Next Wave Target:** +16 tests, +1–2 competitors, 570/570 PASS

---

## Variant 1 — rfi-irfos/ternary-intelligence-stack Investigation

**Goal:** Investigate the `rfi-irfos/ternary-intelligence-stack` GitHub repository (Jun 16 2026) to determine if it has evolved into a credible scientific or hardware competitor.

**Actions:**
1. Inspect `rfi-irfos/ternary-intelligence-stack` for RTL, formal specifications, physics links, or E8/H4 references.
2. If the repo contains meaningful Verilog/Coq/physics content, add a `rfi_irfos_competitor` entry to `benchmark.t27`.
3. If it remains a software/language project, document as non-competitor and add 2 tests in `rtl.t27` or `formal.t27` inspired by any formal-methods patterns found.
4. Run full suite; 570/570 PASS.

**Deliverables:**
- `docs/competitors/rfi_irfos_analysis.md`
- Updated or unchanged `benchmark.t27`
- +2 Pool A tests
- W185 IGLA report

**Effort:** Low–Medium.
**Risk:** Repository likely remains non-scientific.

---

## Variant 2 — Pool A IGLA CODER+RACE + Invariant Depth Push Hybrid

**Goal:** Combine the standard +16 IGLA tests with a property-depth push in one Pool A spec.

**Actions:**
1. Add +2 tests to each Pool A spec (rtl, eda, cordic_fixed, bram_weights, cordic, cordic_top, formal, gemm) = +16 tests.
2. Select one Pool A spec with single-inv or double-inv status and add 1 hepta-invariant (7-property chain) or upgrade a double-inv to triple-inv.
3. Verify 570/570 PASS after both changes.
4. Seal all modified files.

**Deliverables:**
- +16 tests across 8 Pool A specs
- +1 invariant depth upgrade in one Pool A spec
- Updated seal files
- W185 IGLA report

**Effort:** Medium (standard IGLA cadence).
**Risk:** Low.

---

## Variant 3 — Stable Plateau Intelligence + Seal Integrity Audit

**Goal:** The competitive maturation plateau has lasted 7 consecutive IGLA waves (W175–W184). Use this stability window for a full seal integrity audit across all 570 specs.

**Actions:**
1. Run `./target/release/t27c seal --verify` across all specs to detect any stale or missing seals.
2. Regenerate any stale seals (expected 0 in normal operation).
3. Add 2 new tests in `gemm.t27` or `cordic_fixed.t27` testing boundary conditions (e.g., Q15 min/max saturation, overflow clamping).
4. Run full suite; 570/570 PASS.

**Deliverables:**
- Seal integrity audit report
- +2 boundary tests in gemm/cordic_fixed
- W185 report with plateau analysis

**Effort:** Low.
**Risk:** Minimal.

---

## Decision Matrix

| Variant | Effort | Risk | Impact | Recommended? |
|---------|--------|------|--------|--------------|
| 1 — rfi-irfos Investigation | Low–Medium | Low | Low–Medium | If repo evolved |
| 2 — Pool A + Depth Push | Medium | Low | High | **(Recommended)** |
| 3 — Seal Integrity Audit | Low | Very Low | Medium | If maintenance mode |

---

*φ² + 1/φ² = 3 | Cooperation over conquest | Verification pending*
