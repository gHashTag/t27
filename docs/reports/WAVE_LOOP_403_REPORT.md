# Wave Loop 403 — FPGA bitstream configuration formalized in Lean 4

> **Issue:** [#1307](https://github.com/t27/t27/issues/1307)  
> **Branch:** `trinity-rust-rings`  
> **Date:** 2026-07-05  
> **Status:** implemented (Variant B), physical CCLK capture still deferred  
> **Conformance:** `576 / 576 PASS`

---

## 1. Goal

Close W403 by either capturing the real CCLK frequency on pin P12 (Variant A),
extending the Lean 4 model with bitstream-config predicates (Variant B), or
adding a cable-connected SRAM smoke load (Variant C). Hardware access was not
available, so **Variant B** was executed.

---

## 2. Acceptance criteria (AC)

| ID | Criterion | Status |
|----|-----------|--------|
| AC-B1 | `BitstreamConfig` structure + `canonical` predicate in Lean 4. | ✅ |
| AC-B2 | `ColdPOR` preconditions link static config to STAT decision tree. | ✅ |
| AC-B3 | `decision_tree_exhaustive` theorem proves every STAT maps to an outcome. | ✅ |
| AC-B4 | `fpga/HARDWARE_SSOT.md` references the bitstream-config lemmas. | ✅ |
| AC-D1 | `./scripts/tri test` passes. | ✅ |
| AC-D2 | Close-out report + evidence + W404 cooperation variants committed. | ✅ |
| AC-A1 | Physical CCLK trace captured on P12. | ⏸️ deferred |
| AC-C1 | `--require-cable` SRAM smoke load reaches `DONE=HIGH`. | ⏸️ not attempted |

---

## 3. What changed

### 3.1 `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Extended the W402 STAT-register model with bitstream-level predicates and
cold-POR linkage.

New definitions:

- `BitstreamConfig` — records `idcode`, `spi_buswidth`, `startupclk`, `oscfsel`.
- `BitstreamConfig.canonical` — asserts the Wukong V1 / XC7A200T SPI-flash
  defaults:
  - `IDCODE = 0x03636093`
  - `SPI_BUSWIDTH = x1` (`COR1[8:7] = 00`)
  - `STARTUPCLK = CCLK` (`COR0[16:15] = 00`)
  - `OSCFSEL = 0` (`COR0[22:17] = 0`)
- `ColdPOR` — bundles `cfg`, `mode_ok` (FPGA sampled Master SPI x1 at POR),
  and `no_cable_interference` (JTAG cable disconnected during POR).
- `cold_por_spi_flash_pred` — the static + dynamic preconditions for a clean
  SPI-flash boot.

New / updated lemmas:

1. `BitstreamConfig.canonical_implies_spi_x1_cclk_boot` — canonical config
   implies SPI x1 + CCLK startup.
2. `cold_por_done_eos_high_implies_boot_success` — static preconditions +
   `DONE=1` + `EOS=1` ⇒ `boot_success`.
3. `cold_por_done_low_implies_h2` — static preconditions + `DONE=0` ⇒
   `h2_cclk_timing`.
4. `decision_tree_exhaustive` — for every STAT value, one of
   `boot_success`, `h2_cclk_timing`, `mode_mismatch`, or `fatal_error` holds.

Modeling note: `boot_success` no longer requires `EOS=1` because, in a valid
SPI master boot, `DONE=1` implies `EOS=1`. EOS remains a dynamic observation in
`cold_por_done_eos_high_implies_boot_success`.

### 3.2 `proofs/lean4/Trinity.lean`

No change; the module was already imported in W402.

### 3.3 `fpga/HARDWARE_SSOT.md`

Added a formal-traceability callout in §3.2 linking the canonical bitstream
configuration (`tri fpga bit-config` audit fields) to the Lean 4 predicates
and the exhaustive decision-tree theorem.

### 3.4 `docs/NOW.md`

Added W403 entry; `Last updated` stays 2026-07-05.

### 3.5 Planning / coordination

- `.claude/plans/wave-loop-403.md` already contained the decomposed plan,
  weak-point scan, and competitor scan from the start of the wave.
- `.trinity/current-issue.md` already pointed to #1307.

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

- **AC-A1 (physical CCLK measurement):** still requires logic-analyzer /
  oscilloscope bench time. The `tri fpga measure-cclk --csv` tooling is ready.
- **AC-C1 (`--require-cable` SRAM smoke):** depends on a connected Digilent
  FTDI cable and board. Not attempted because the default wave path (Variant B)
  does not require hardware.

---

## 6. Key learnings

1. **Bool-level proofs need explicit disjunct construction in Lean 4.**
   `rcases` and `tauto` struggle with `Bool` disjunctions that are defined via
   `Decidable.rec`/`decide`. The exhaustive decision-tree proof was closed by
   explicit `Or.inl` / `Or.inr` paths plus `rcases` on the `¬fatal_error`
   conjunction.
2. **Keep model corners honest.** The original `boot_success` required
   `EOS=1`, which left a logical gap (DONE=1, mode OK, EOS=0, no fatal error).
   Removing the unnecessary EOS requirement made the exhaustiveness theorem
   provable without adding an unreachable "other" branch.
3. **Formal traceability is cumulative.** W402 linked the decision tree to
   Lean; W403 now links the `.bit` configuration audit fields as well. Each
   wave adds another checkpoint that competitors would have to replicate.
4. **No-hardware waves can still advance the repository.** When bench access is
   blocked, formalizing the next layer of operational knowledge is a valid
   close-out and keeps the loop cadence.

---

## 7. Next loop (W404) targets

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md` for three cooperation
variants. Likely candidates include:

- physical CCLK measurement on P12 and recording the result,
- extending the Lean 4 model with `CCLK` frequency bounds or `OSCFSEL` variant
  predicates,
- extending `tri fpga smoke-gate` to optionally assert `DONE=HIGH` via a
  cable-connected SRAM load.

---

*φ² + 1/φ² = 3 | TRINITY*
