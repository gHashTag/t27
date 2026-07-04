# Wave Loop 403 — Decomposed Plan

**Issue:** #1307  
**Branch:** `trinity-rust-rings`  
**Goal:** Close the deferred physical CCLK measurement (Variant A), extend the
Lean 4 formal model to bitstream configuration (Variant B), or add a
physical-cable smoke path (Variant C).

---

## 1. Weak points

### 1.1 Physical blocker (Variant A)
- **CCLK measurement still requires bench access.** Without a logic analyser /
  oscilloscope on pin P12, Variant A cannot close. The `tri fpga measure-cclk`
  parser is ready, but the actual capture is operator-dependent and cannot be
  performed autonomously in a headless session.
- **OSCFSEL-to-MHz mapping is undocumented.** Even if a capture is taken, mapping
  raw `OSCFSEL=0..5` to frequency requires either multiple captures or an
  external 7-series configuration reference.

### 1.2 Formal gap (Variant B)
- `TernaryFPGABoot.lean` models STAT decoding and the cold-POR decision tree,
  but does not yet link the canonical bitstream configuration (`IDCODE`,
  `SPI_BUSWIDTH`, `STARTUPCLK`, `OSCFSEL`) to the boot predicates.
- The smoke-gate assertions in `cli/tri/src/fpga.rs` and
  `scripts/dump_bit_config.py` are executable, but they are not connected to
  the formal specification. A formal proof that the canonical configuration
  implies the preconditions for `boot_success` would close this gap.

### 1.3 Smoke-gate physical gap (Variant C)
- `tri fpga smoke-gate` is board-less. A cable-connected SRAM load would catch
  bitstream regressions that the static assertions miss, but it needs the
  Digilent FTDI cable and a powered board.

---

## 2. Competitor scan

| Project | Approach | Relevance to W403 |
|---------|----------|-------------------|
| **[Verilean / Sparkle](https://github.com/Verilean/sparkle)** | Lean 4-embedded HDL with native proofs and SystemVerilog synthesis | Strongest formal-HDL competitor. Includes verified RISC-V SoC, BitNet accelerator, AXI4-Lite. Does not model FPGA boot configuration, so t27 can differentiate by formalizing the bring-up path. |
| **[Aria-HDL](https://github.com/zeta1999/fpga-meta-compiler-public)** | FPGA meta-compiler emitting Verilog/VHDL/SVA/Lean 4 proof obligations | Competes in the "spec → formal proof → FPGA" space. Does not specifically cover 7-series boot config, leaving a niche for t27. |
| **[seLe4n](https://sele4n.org/)** | Microkernel verified in Lean 4 | Shows Lean 4 can verify low-level system bring-up code; analogous discipline can be applied to FPGA cold-POR. |
| **[USENIX WOOT 2024 Zynq secure boot paper](https://www.usenix.org/system/files/woot24-ravi.pdf)** | Offensive analysis of Zynq-7000 secure boot / bitstream recovery | Motivation: boot configuration mistakes have real security consequences. Formal audit of config registers is a defensive countermeasure. |
| **FIRRTL / Chisel** | Scala DSL → intermediate form → Verilog | Widely used, no proof obligations by default. |
| **Clash** | Haskell → VHDL/Verilog | Functional HDL, no proof obligations by default. |
| **Bluespec** | Rule-based HDL with types | Mature, but proprietary tooling. |

**Takeaway:** The credible threat is **Verilean / Sparkle / Aria-HDL**. They can
claim "Lean 4 + FPGA synthesis + proofs"; t27 counters by extending its proof
lattice into the **configuration and boot domain** — a space they do not
currently cover.

---

## 3. Chosen variant

**Variant B — Extend the Lean 4 model to bitstream configuration predicates** is
the highest-leverage path because:
- it requires no hardware,
- it directly closes the formal gap identified in W402,
- it strengthens t27's differentiation against Sparkle/Aria-HDL,
- it documents why the smoke-gate assertions are exactly the right set.

Variant A tooling is already complete; if hardware becomes available, the
physical capture can be slotted in without code changes. Variant C is deferred
to a stretch goal.

---

## 4. Decomposed work

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 4.1 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | `BitstreamConfig` structure: `idcode`, `spi_buswidth`, `startupclk`, `oscfsel` |
| 4.2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Canonical config constant / predicate matching smoke-gate assertions |
| 4.3 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Lemmas: `canonical_config_implies_spi_x1_cclk`, `config_and_mode_implies_boot_pred`, `boot_success_decision_tree` |
| 4.4 | `fpga/HARDWARE_SSOT.md` | Link the bitstream config audit checklist to the Lean 4 predicates |
| 4.5 | `docs/reports/WAVE_LOOP_403_REPORT.md` | Close-out report |
| 4.6 | `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md` | Evidence of build and lemma coverage |
| 4.7 | `docs/reports/FPGA_LOOP_COOPERATION_2026-07-05.md` | W404 cooperation variants |
| 4.8 | `.trinity/experience.md` | W403 learnings |
| 4.9 | git/PR | Commit, push `trinity-rust-rings`, create PR, close #1307 |

---

## 5. Acceptance criteria

- AC-B1: `BitstreamConfig` defined with the four fields matching the smoke-gate
  assertions.
- AC-B2: `canonical_config` predicate matches the canonical Wukong V1 / XC7A200T
  setup.
- AC-B3: At least three lemmas link config / mode / boot predicates.
- AC-B4: `fpga/HARDWARE_SSOT.md` references the formal config predicates.
- AC-B5: `lake build Trinity.TernaryFPGABoot` passes.
- AC-B6: `./scripts/tri test` passes (576/576 baseline maintained).
- AC-B7: Close-out report + evidence + W404 cooperation variants committed.

---

## 6. Risks

- **Lean 4 proof complexity:** keep the model shallow — encode configuration as
  a record and prove implication lemmas, not a full bitstream parser.
- **Backend hash drift:** after editing Lean files only, no spec hashes should
  change; but run `tri test` to confirm.
- **CI time:** the new module is small and should not affect the suite budget.

---

*φ² + φ⁻² = 3 | TRINITY*
