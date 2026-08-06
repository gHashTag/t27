# Wave Loop 404 — FPGA hardware smoke gate with `--require-cable`

> **Issue:** [#1309](https://github.com/t27/t27/issues/1309)  
> **Branch:** `trinity-rust-rings`  
> **Date:** 2026-07-06  
> **Status:** implemented (Variant C), physical CCLK capture still deferred  
> **Conformance:** `576 / 576 PASS`

---

## 1. Goal

Close W404 by either measuring CCLK on pin P12 (Variant A), extending the
Lean 4 model with CCLK bounds (Variant B), or adding a cable-connected SRAM
smoke load to `tri fpga smoke-gate` (Variant C).

At the start of the wave the Digilent FTDI cable and XC7A200T board were
found to be reachable (`openFPGALoader --detect` returned idcode `0x3636093`),
so **Variant C** was executed.

---

## 2. Acceptance criteria (AC)

| ID | Criterion | Status |
|----|-----------|--------|
| AC-C1 | `tri fpga smoke-gate --require-cable` detects cable, loads SRAM, asserts `DONE=HIGH`. | ✅ |
| AC-D1 | `./scripts/tri test` passes. | ✅ |
| AC-D2 | Close-out report + evidence + W405 cooperation variants committed. | ✅ |
| AC-A1 | Physical CCLK trace captured on P12. | ⏸️ deferred |
| AC-A2 | `fpga/HARDWARE_SSOT.md` §3.5 contains measured value. | ⏸️ deferred |
| AC-B1 | New Lean 4 lemmas link `OSCFSEL`/CCLK bounds to decision trees. | ⏸️ not executed |

---

## 3. What changed

### 3.1 `cli/tri/src/fpga.rs`

Extended the `SmokeGate` subcommand with three new optional arguments:

- `--require-cable` — opt into the cable-connected hardware check.
- `--cable` — openFPGALoader cable profile (default `digilent_hs2`).
- `--part` — FPGA part/package for openFPGALoader (default `xc7a200tfgg676`).

When `--require-cable` is set, `smoke_gate` now:

1. Detects the JTAG chain with `openFPGALoader --detect`.
2. Loads the canonical bitstream into FPGA SRAM via `load_sram`.
3. Captures STAT via `capture_stat`.
4. Asserts `boot_success` conditions (`DONE=1`, `MODE=0b001`, no CRC/ID/DEC
   errors) using the new `assert_stat_boot_success` helper.

The existing board-less checks (bit-config audit, dry-run CCLK sweep, yosys
synthesis) still run afterwards, so CI without hardware is unaffected.

### 3.2 `fpga/HARDWARE_SSOT.md`

Added a hardware smoke traceability callout in §3.2 linking the new
`tri fpga smoke-gate --require-cable` behavior to the Lean 4 `boot_success`
predicate.

### 3.3 Planning / coordination

- `.claude/plans/wave-loop-404.md` updated to show Variant C was implemented,
  Variants A/B deferred, and acceptance-criteria status.
- `docs/NOW.md` updated with the W404 entry.
- `.trinity/experience.md` updated with W404 learnings.

---

## 4. Verification

```bash
cargo run --release -p tri -- fpga smoke-gate --require-cable
```

Bench output (excerpt):

```text
[smoke-gate] require-cable: detecting FPGA via digilent_hs2...
[smoke-gate] cable OK (FPGA detected)
[smoke-gate] loading SRAM: .../ternary_mac_demo_top_200t.bit
Done
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
[smoke-gate] reading STAT after SRAM load...
[smoke-gate] hardware check OK (DONE=HIGH, mode=001, no errors)
[smoke-gate] bit-config audit: ...
[smoke-gate] dry-run sweep report OK (6 variants)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

Post-load STAT = `0x401079FC`, matching the `Trinity.StatRegister.boot_success`
example already proved in Lean 4.

Board-less path also verified:

```bash
./scripts/tri test
# 576 / 576 PASS
```

---

## 5. What was not done and why

- **AC-A1/A2 (physical CCLK measurement):** no logic analyzer or oscilloscope is
  connected; the agent can drive JTAG but cannot sample the analog CCLK
  waveform.
- **AC-B1 (formal `OSCFSEL`/CCLK bounds):** not needed because hardware was
  available for Variant C. It remains a candidate for a future no-hardware wave.

---

## 6. Key learnings

1. **Probe hardware before choosing the variant.** The W404 plan assumed
   another no-hardware formal extension, but the Digilent cable and board were
   reachable, making a hardware smoke gate the higher-leverage close-out.
2. **Optional hardware gates preserve CI.** Adding `--require-cable` kept the
   board-less path as the default; only runners with hardware opt into the SRAM
   load assertion.
3. **Reuse existing command helpers.** `load_sram` and `capture_stat` already
   handled openFPGALoader parsing and error reporting; using them avoided
   duplication.
4. **Link hardware checks to formal predicates.** Asserting the same
   `boot_success` conditions used in Lean 4 gives the hardware result the same
   audit trail as the formal model.

---

## 7. Next loop (W405) targets

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-07.md` for three cooperation
variants. Likely candidates include:

- physical CCLK measurement on P12 once a logic analyzer/oscilloscope is
  available,
- extending the hardware smoke gate to also verify flash boot from cold POR,
- formalizing the `OSCFSEL`/CCLK bounds as a no-hardware fallback.

---

*φ² + 1/φ² = 3 | TRINITY*
