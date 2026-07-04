# Wave Loop 402 — FPGA cold-POR decision tree formalized in Lean 4

> **Issue:** [#1305](https://github.com/t27/t27/issues/1305)  
> **Branch:** `trinity-rust-rings`  
> **Date:** 2026-07-05  
> **Status:** implemented (Variant B), physical CCLK capture still deferred  
> **Conformance:** `576 / 576 PASS`

---

## 1. Goal

Close the deferred W401 AC5 by either measuring CCLK on pin P12 (Variant A) or
formalizing the cold-POR / CCLK decision tree (Variant B). Hardware access was
not available, so **Variant B** was executed.

---

## 2. Acceptance criteria (AC)

| ID | Criterion | Status |
|----|-----------|--------|
| AC-B1 | New Lean 4 module compiles with `lake build`. | ✅ |
| AC-B2 | At least three decision-tree lemmas are stated and proved. | ✅ |
| AC-B3 | `fpga/HARDWARE_SSOT.md` references the formal lemmas. | ✅ |
| AC-B4 | `./scripts/tri test` passes. | ✅ |
| AC-B5 | Close-out report + evidence + W403 cooperation variants committed. | ✅ |
| AC-A1 | Physical CCLK trace captured on P12. | ⏸️ deferred |

---

## 3. What changed

### 3.1 `proofs/lean4/Trinity/TernaryFPGABoot.lean` (new)

A formal model of the 7-series FPGA STAT register and the cold-POR decision
tree. It provides:

- `StatRegister` with named field decoders (`mode`, `done`, `eos`, `crc_error`,
  `id_error`, `dec_error`, `bus_width`, `init_complete`) matching the bit
  layout used by `cli/tri/src/fpga.rs` and `cli/dlc10/src/lib.rs`.
- Named constants `MODE_MASTER_SPI_X1` and `BUS_WIDTH_X1`.
- Decision predicates:
  - `boot_success` — DONE=HIGH, EOS=HIGH, mode=Master SPI x1, no CRC/ID/DEC
    errors.
  - `h2_cclk_timing` — DONE=LOW, mode correct, no CRC/ID errors (CCLK/SPI
    timing hypothesis).
  - `mode_mismatch` — mode not Master SPI x1.
  - `fatal_error` — CRC, ID, or DEC error set.
- Proved lemmas:
  1. `boot_success_implies_mode_master_spi_x1`
  2. `boot_success_implies_no_fatal_error`
  3. `h2_implies_mode_ok_done_low`
  4. `fatal_error_implies_not_boot_success`
  5. `mode_mismatch_implies_not_boot_success`
  6. `stat_success_example_boots` — `STAT=0x401079FC` satisfies `boot_success`.
  7. `stat_incomplete_example_is_h2` — `STAT=0x5000190C` satisfies
     `h2_cclk_timing`.
  8. `boot_success_and_h2_disjoint` — the two predicates are mutually
     exclusive.

### 3.2 `proofs/lean4/Trinity.lean`

Added `import Trinity.TernaryFPGABoot` so the module is included in the
library build.

### 3.3 `fpga/HARDWARE_SSOT.md`

Updated §3.2 to reference the Lean 4 predicates and the verified W400 / W401
STAT examples.

### 3.4 Resealed specs

The W401 squash-land brought the master gen-verilog backend (#1250) onto
`trinity-rust-rings`, changing generated hashes for three specs:

- `specs/fpga/bpsk.t27`
- `specs/ml/transformer/feed_forward_network.t27`
- `specs/numeric/formats_catalog.t27`

All three were resealed and now verify cleanly.

### 3.5 Planning / coordination

- `.trinity/current-issue.md` updated to point to #1305.
- `.claude/plans/wave-loop-402.md` created with weak-point and competitor
  analysis.

---

## 4. Verification

```bash
lake build Trinity.TernaryFPGABoot   # ✅
./scripts/tri test                   # 576 / 576 PASS
```

The full `lake build Trinity` target still has pre-existing failures in
`NeutrinoMasses.lean` and `H4Lagrangian.lean`; those modules are independent of
the FPGA boot formalization and were already failing before this wave.

---

## 5. What was not done and why

AC-A1 (physical CCLK measurement on P12) requires a logic analyser or
oscilloscope capture. The tooling (`tri fpga measure-cclk --csv`) is ready, but
the capture must be performed at the bench. It is deferred to W403 if hardware
becomes available.

---

## 6. Key learnings

1. **Long-lived branches need squash-landing.** `trinity-rust-rings` had
   accumulated many commits without per-commit issue references. Squashing the
   wave sequence into a single mergeable commit was the only path through the L1
   TRACEABILITY gate.
2. **Reseating generated artifacts is part of landing.** When a backend change
   (gen-verilog #1250) reaches a working branch, the seal ledger must be
   updated before the conformance gate passes.
3. **Formalizing operational knowledge is high leverage.** The cold-POR
   decision tree was previously prose-only; encoding it in Lean 4 gives it the
   same audit trail as the ternary MAC proof lattice.
4. **Competitor defense needs explicit claims.** Verilean and Sparkle HDL are
   the closest formal-HDL competitors; a small, concrete formal module is a
   stronger rebuttal than a roadmap slide.

---

## 7. Next loop (W403) targets

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-05.md` for three cooperation
variants. Likely candidates include:

- physical CCLK measurement on P12 and recording the result,
- extending the Lean 4 model with `STARTUPCLK` / `OSCFSEL` predicates,
- extending `tri fpga smoke-gate` to optionally assert `DONE=HIGH` via a
  cable-connected SRAM load.

---

*φ² + 1/φ² = 3 | TRINITY*
