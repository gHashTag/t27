# Wave Loop 366 — Decomposed Plan

**Date:** 2026-07-01
**Issue target:** #1252 (new W366 tracking issue)
**Recommended variant:** Variant B from `WAVE_LOOP_365_COOPERATION.md`

---

## Goals

| Target | W365 → W366 |
|--------|-------------|
| Pool A invariants | 107 → **108** |
| CODER invariants | 97 → **98** |
| Pool B invariants | 125 → **126** |
| Integration invariants | 107 → **108** |
| Lean 4 generic ∀ | 204 → **208** |
| IGLA conformance | **546/546 PASS** |
| Zero-IGLA-failure streak | **99 → 100 waves** |
| FPGA board load | retry `dlc10 idcode` / `sram` |
| gen-verilog backend | land one safe sub-fix OR create focused follow-up issue with repro |

---

## 1. Formal wave (27 IGLA specs)

- Reuse generator pattern: copy `scripts/gen_w365.py` → `scripts/gen_w366.py`, parameterize wave number, expected invariant counts, theorem references.
- Forward-append W366 blocks to all 27 core IGLA specs:
  - 2 `test` blocks
  - 1 `invariant` block
- **+54 tests**, **+27 invariants**.
- Regenerate all 27 IGLA seals from `/Users/playra/t27`.

---

## 2. Lean 4 proof lattice (4 new generic ∀ theorems)

Append to `proofs/lean4/Trinity/TernaryInference.lean`:

1. **`ternaryMacAccumulateFortyTwoPlusGeneric`** — 42-variable plus accumulation (`a` … `ap`).
2. **`ternaryMacAccumulateFortyOneMinusGeneric`** — 41-variable minus accumulation (`a` … `ao`).
3. **`ternaryMacNovemdecupleCancellationGeneric`** — depth-19 alternating plus/minus, residual `mac(x, a, .plus)`.
4. **`ternaryMacZeroWeightNonupleClosureGeneric`** — 9 zero-weight MACs around a plus-weight MAC; 25th proof lattice dimension.

Validate with `lake build Trinity.TernaryInference`. If 42-variable plus times out, fallback to completing 41-variable minus lattice first and promoting cancellation/closure only.

---

## 3. OpenXC7 bitstream and board flash retry

- Ensure `dlc10` is built: `cargo build --release -p dlc10`.
- Run `dlc10 idcode` and capture output.
  - If success: proceed to `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`, then `dlc10 reload`; document IDCODE, DONE pin/LED observation.
  - If failure: update `docs/reports/FPGA_EVIDENCE_W366.md`.

---

## 4. Project weak-point probe: gen-verilog backend

#1245 is closed, but four concrete lowering defects remain reproducible in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`. W366 attempts **one** safe, regression-free sub-fix.

### Candidate fixes (in descending safety order)

1. **`0x` literal width padding** (defect #2)
   - In `gen_verilog_const`, pass the declared type width into a new `gen_verilog_literal` helper and pad `0x` values to that width.
   - Risk: touches the expression-emission call site for every constant; must pass full 546-spec suite.

2. **Early `return` lowering** (defect #3)
   - In `gen_verilog_fn`, detect a trailing `return` after an `if` block with no `else` and emit an `else` branch.
   - Risk: changes control-flow output for many functions; needs careful brace/statement matching.

3. **Struct-field name alignment** (defect #5)
   - Either change declaration to use variable names (requires knowing all variables of the type) or change access to use the lowercased type name.
   - Risk: heuristic; multiple variables of same type break the simple fix.

### Fallback
If no candidate fix passes the full conformance suite, create a focused follow-up issue (e.g. `#1253`) documenting the chosen defect with a minimal reproduction spec and the attempted patch diff. Do **not** leave uncommitted compiler changes in the working tree.

---

## 5. Research / competitive landscape

Update the W365 research summary with any new 2026 sources, especially:

- Ternary/1.58-bit: KU Leuven ISPASS 2026, VitaLLM, TENET, TeLLMe v2, TerEffic, TernaryCore, Ternary-NanoCore.
- Formal HDL: FormalRTL, Arch, CktFormalizer, Veri-Sure, Lutsig.
- Note that #1243 (`specs/fpga/bpsk.t27`) already exists on `origin/master`; the local `trinity-rust-rings` branch does not yet include it.

Document key takeaways in the W366 report: competitors still report **zero generic ∀ ternary MAC proofs**.

---

## 6. Report and cooperation variants

Produce:
- `docs/reports/WAVE_LOOP_366_REPORT.md`
- `docs/reports/WAVE_LOOP_366_COOPERATION.md` with three W367 variants:
  - **Variant A** — formal-only (safe)
  - **Variant B** — formal + board flash retry + one gen-verilog sub-fix (recommended)
  - **Variant C** — formal + RTL-to-Lean bridge prototype (high risk)

---

## 7. Memory / experience update

- Prepend W366 index entry to `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md`.
- Write `~/.claude/projects/-Users-playra-t27/memory/wave-loop-366.md`.
- Append W366 learnings to `.trinity/experience.md`.

---

## 8. Verification and land

- `./target/release/t27c suite --repo-root /Users/playra/t27` → 546/546 PASS.
- `lake build Trinity.TernaryInference` → success.
- `git commit` with message containing `Closes #1252`.
- Push if authorized; otherwise leave on `trinity-rust-rings` for PR.

---

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| 42-variable accumulation times out | Promote 41-var minus lattice first; keep 3 of 4 theorems if needed. |
| gen-verilog fix causes regressions | Revert and fall back to documentation + follow-up issue. |
| Board still not detected | Document as hardware blocker. |
| Issue #1252 number collision | Create issue during implementation; use whatever number GitHub returns. |
| `gh` token invalid | Use `env -u GH_TOKEN gh ...`. |

---

## Task sequence

1. Create GitHub issue for W366.
2. Implement 4 Lean theorems + generators.
3. Run `lake build Trinity.TernaryInference`.
4. Implement 27 spec blocks + generator.
5. Regenerate 27 IGLA seals.
6. Run full conformance suite.
7. Retry board flash; document result.
8. Attempt one safe gen-verilog sub-fix; if risky, document instead.
9. Write W366 report + cooperation variants.
10. Update memory + experience.
11. Commit with `Closes #1252` (or actual issue number).
12. Run final conformance suite.
