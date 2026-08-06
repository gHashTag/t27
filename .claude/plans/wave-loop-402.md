# Wave Loop 402 — Decomposed Plan

**Issue:** #1305  
**Branch:** `trinity-rust-rings`  
**Goal:** Close the deferred W401 AC5 (physical CCLK measurement) or pursue the
highest-leverage no-hardware alternative: a Lean 4 formalization of the cold-POR
/ CCLK decision tree.

---

## 1. Weak points

### 1.1 Physical blocker (Variant A)
- **AC5 requires a logic analyser / oscilloscope on pin P12.** Without bench
  access the actual CCLK frequency cannot be recorded. The `tri fpga
  measure-cclk --csv` parser is already robust, but the measurement itself is
  operator-dependent.
- **OSCFSEL-to-MHz mapping remains undocumented.** Even if a capture is taken,
  mapping raw `OSCFSEL=0..5` to frequency requires one capture per variant or an
  external 7-series configuration reference.

### 1.2 Formal gap (Variant B)
- The W400/W401 decision trees live only in prose (`fpga/HARDWARE_SSOT.md`).
  They are not linked to the same formal infrastructure that underpins the
  ternary MAC proof lattice (`proofs/lean4/Trinity/TernaryMac.lean`).
- Competitor formal-HDL projects (Sparkle HDL, Verilean, Sail-to-Verilog) can
  claim traceability; t27 currently has no formal FPGA-boot specification.

### 1.3 Smoke-gate physical gap (Variant C)
- `tri fpga smoke-gate` is board-less. A cable-connected load would catch
  bitstream regressions that the static assertions miss, but it needs the
  Digilent FTDI cable and a powered board.

---

## 2. Competitor scan

| Project | Approach | Relevance to W402 |
|---------|----------|-------------------|
| **Sparkle HDL** | Rust-embedded DSL → formal semantics | Credible formal competitor in the RTL-design space; t27's defense is the spec-first `.t27` + Lean 4 proof stack. |
| **Verilean** | Lean 4 → verified RTL | Direct competitor for "Lean 4 HDL"; W402 Variant B closes a small slice of that gap for FPGA boot diagnostics. |
| **Sail / RISC-V Sail** | ISA-level formal → emulator / RTL | Not HDL-focused, but demonstrates formal traceability from spec to silicon. |
| **Kami (MIT)** | Coq → Bluespec-like hardware | Shows theorem-prover-verified hardware is publishable; t27 uses Lean 4 instead. |
| **FIRRTL / Chisel** | Scala DSL → intermediate form → Verilog | Widely used but not formally verified; t27 differentiates on proof. |
| **Clash** | Haskell → VHDL/Verilog | Functional HDL, no proof obligations by default. |
| **Bluespec** | Rule-based HDL with types | Mature, but proprietary tooling; t27's open-source Lean stack is the contrast. |

**Takeaway:** The credible threat is **Verilean / Lean 4 HDL**. Adding a Lean 4
FPGA-boot specification gives t27 a formal story in the same design space and
leverages the existing `TernaryMac` proof infrastructure.

---

## 3. Chosen variant

**Variant B — Formalize the cold-POR / CCLK decision tree in Lean 4** is the
highest-leverage path because:
- it requires no hardware,
- it reuses the existing Lean 4 build (`proofs/lean4/Trinity`),
- it directly addresses the competitor gap (Verilean / Sparkle),
- it documents the W400 empirical result as a formal specification.

Variant A tooling is already complete; if hardware becomes available during the
loop, the physical capture can be slotted in without code changes. Variant C is
deferred to a stretch goal.

---

## 4. Decomposed work

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 4.1 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Definitions: `StatRegister`, `Mode`, `Done`, `Eos`, `CrcError`, `IdError` |
| 4.2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Helper lemmas decoding `STAT` bits into named predicates |
| 4.3 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Decision-tree lemmas: `boot_success_iff`, `mode_master_spi_x1_iff`, `crc_or_id_error_implies_not_done` |
| 4.4 | `proofs/lean4/Trinity.lean` | Export the new module |
| 4.5 | `proofs/lean4/lakefile.toml` / CI | Ensure the module builds in `./scripts/tri test` |
| 4.6 | `fpga/HARDWARE_SSOT.md` | Link the documented decision trees to the formal lemmas |
| 4.7 | `docs/reports/WAVE_LOOP_402_REPORT.md` | Close-out report |
| 4.8 | `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md` | Evidence of Lean build and lemma coverage |
| 4.9 | `docs/reports/FPGA_LOOP_COOPERATION_2026-07-05.md` | W403 cooperation variants |
| 4.10 | `.trinity/experience.md` | W402 learnings |
| 4.11 | git/PR | Commit, push `trinity-rust-rings`, create PR, close #1305 |

---

## 5. Acceptance criteria

- AC-B1: `proofs/lean4/Trinity/TernaryFPGABoot.lean` compiles with `lake build`.
- AC-B2: At least three decision-tree lemmas are stated and proved.
- AC-B3: `fpga/HARDWARE_SSOT.md` references the formal lemmas.
- AC-B4: `./scripts/tri test` passes (575/575 baseline maintained).
- AC-B5: Close-out report + evidence + W403 cooperation variants committed.

---

## 6. Risks

- **Lean 4 toolchain drift:** the existing `lakefile.toml` may need updates for
  new module inclusion; keep changes minimal.
- **Proof complexity:** full bit-accurate 7-series STAT decoding is unnecessary;
  focus on the high-level decision predicates used in `fpga/HARDWARE_SSOT.md`.
- **CI time:** adding a new Lean module must not push `tri test` over budget;
  the module is small.

---

*φ² + φ⁻² = 3 | TRINITY*
