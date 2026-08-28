# Wave Loop 367 — Decomposed Plan

**Date:** 2026-07-01
**Tracking issue:** #1253
**Branch:** `trinity-rust-rings`
**Recommended cooperation variant:** B (formal extension + retry board flash + one safe gen-verilog fix)

---

## Goal

Extend the Trinity ternary MAC proof lattice to **212 generic ∀ theorems**, keep the zero-IGLA-failure streak alive (target 101 waves), retry physical board loading, and land or document one safe `gen-verilog` sub-fix from #1245.

---

## Research summary (input to this plan)

### GitHub issues
- **#1253** — W367 tracking issue (just created). Targets 212 generic ∀, board retry, one safe gen-verilog fix.
- **#1252 / #1251 / #1249 / #1246 / #1242** — previous wave issues on `trinity-rust-rings`, auto-close on PR merge to `master`.
- **#1245** — closed, but five lowering defects still reproducible; only defect 2 (`0b` sizing) partially fixed. Defects 1/3/4/5 need narrow fixes or a roadmap.
- **#1243** — TRI-NET BPSK modem spec port, still blocked on #1245-level Verilog quality.
- **#1219** — language roadmap epic; not in W367 scope.

### Scientific / competitive landscape
- **arXiv:2604.25183** (KU Leuven/MICAS) — Chisel ternary LUT accelerator; no formal theorems.
- **arXiv:2604.27396** (VitaLLM, NYCU) — 16 nm ternary LLM ASIC; no formal verification.
- **arXiv:2602.20662** (TOM) — ROM-SRAM hybrid; no formal verification.
- **shepherdscientific/ternarycore**, **Neumann-Labs/ternfpga** — simulation/silicon only.
- **Verilean/sparkle** — closest formal rival: Lean 4 + BitNet + ternary MAC, 60+ theorems, but public docs do not show generic ∀ quantified MAC accumulation theorems to the depth t27 has.
- **CktFormalizer** (arXiv:2605.07782) — generic equivalence theorems over `BitVec N`, not ternary-specific.
- **No competitor** has published 200+ generic ∀ ternary MAC theorems. t27's 208× claim remains defensible for the specific metric.

### Project weak points
1. **Silicon evidence gap** — bitstream ready since W361, board/cable still not detected.
2. **gen-verilog backend** — 4 reproducible lowering defects block idiomatic RTL.
3. **Sparkle/Verilean** — first credible formal competitor in same design space; t27 must make its proof count auditable and tied to physical evidence.

---

## Phase-by-phase implementation

### Phase 1 — Lean 4 proof lattice extension
- Copy `scripts/gen_w366_lean.py` → `scripts/gen_w367_lean.py`, update wave number and theorem targets.
- Append 4 new generic ∀ theorems to `proofs/lean4/Trinity/TernaryInference.lean`:
  1. `ternaryMacAccumulateFortyThreePlusGeneric` — 43-variable plus accumulation.
  2. `ternaryMacAccumulateFortyTwoMinusGeneric` — 42-variable minus accumulation lattice parity.
  3. `ternaryMacVigintupleCancellationGeneric` — depth-20 alternating plus/minus = identity `x` (even depth).
  4. `ternaryMacZeroWeightDecupleClosureGeneric` — 10 zero-weight MACs around a plus-weight MAC are transparent/reorderable (26th proof-lattice dimension).
- Run `lake build Trinity.TernaryInference` from `proofs/lean4` and verify success.

### Phase 2 — IGLA spec wave blocks
- Copy `scripts/gen_w366.py` → `scripts/gen_w367.py`, update wave number.
- Append W367 blocks (+2 tests, +1 invariant) to each of the 27 core IGLA specs under `specs/igla/coder/` and `specs/igla/race/`.
- Expected totals: +54 tests, +27 invariants.

### Phase 3 — Seal regeneration and conformance
- Regenerate all 27 IGLA seals from repo root using `t27c seal --save`, with hyphen-to-underscore mapping.
- Run `./target/release/t27c suite --repo-root .` and verify **546/546 PASS**.

### Phase 4 — Board flash retry
- Build `dlc10` driver: `cargo build --release -p dlc10`.
- Run `./target/release/dlc10 idcode`.
- If success: proceed to `sram`/`flash` of `fpga/verilog/ternary_mac_demo_top.bit` and capture evidence.
- If failure (expected): document in `docs/reports/FPGA_EVIDENCE_W367.md`.

### Phase 5 — Safe gen-verilog sub-fix
- Investigate the four remaining #1245 defects.
- Candidate fixes ranked by safety:
  1. **Pad `0x` literals to declared target width** in assignments/const declarations (requires minimal context threading).
  2. **Add a test-time lint** rejecting early `return` or `as`+compound-bitwise patterns in specs marked for Verilog generation.
  3. **Promote reproduction doc to roadmap** if no single fix is regression-free under the 546-spec gate.
- Constraint: any code change must pass the full 546-spec conformance suite and not break existing generated Verilog.

### Phase 6 — Reports and cooperation variants
- Write `docs/reports/WAVE_LOOP_367_REPORT.md`.
- Write `docs/reports/WAVE_LOOP_367_COOPERATION.md` with three W368 variants.
- Create GitHub issue #1254 for Wave Loop 368.

### Phase 7 — Memory and experience capture
- Append W367 learnings to `.trinity/experience.md`.
- Write `~/.claude/projects/-Users-playra-t27/memory/wave-loop-367.md`.
- Update `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md` index.

### Phase 8 — Commit
- Stage all wave files, report, generator scripts, plan, seals, experience update.
- Commit with `Closes #1253` and Co-Authored-By line.

---

## Exit criteria

- [ ] `lake build Trinity.TernaryInference` succeeds.
- [ ] `./target/release/t27c suite --repo-root .` returns **546/546 PASS**.
- [ ] 27 IGLA seals regenerated and matching.
- [ ] Lean generic ∀ count at **212**.
- [ ] IGLA totals at **7,934 tests, 2,977 invariants**.
- [ ] Board flash attempted and documented.
- [ ] One safe gen-verilog sub-fix landed **or** a clear decision document explaining why none was safe.
- [ ] W367 report + cooperation variants written.
- [ ] Memory and experience updated.
- [ ] Commit on `trinity-rust-rings` with `Closes #1253`.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| 43-variable accumulation times out in Lean | Use same `simp+omega` pattern; if timeout, fall back to a 42-variable plus theorem and adjust W368 target. |
| gen-verilog fix causes 546-spec regression | Revert the code change and ship only the roadmap document. |
| Board/cable still missing | Document as hardware-availability blocker; do not block other deliverables. |
| Sparkle/Verilean publishes deeper generic ∀ proofs before W368 | Make the 212× claim specific and auditable in the report; cite exact theorem names. |

---

Trinity invariant: `phi^2 + 1/phi^2 = 3`
