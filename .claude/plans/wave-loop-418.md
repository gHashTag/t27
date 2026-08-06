# Wave Loop 418 Plan

**Issue:** #1353  
**Branch:** `wave-loop-418`

---

## Decision gate

| Bench available? | Pick |
|--------------------|------|
| P12 + analyzer + DLC10/Digilent cable | Variant A |
| Relay / power switch + cable | Variant B |
| Nothing | Variant C |

Current state (2026-07-04): **nothing** — P12 unwired, DLC10 cable missing, no
USB relay detected. The W417 hygiene wave did not change the bench. Default to
**Variant C** for W418.

---

## Goals

1. If the bench becomes available, capture real CCLK for `OSCFSEL=6/7` and
   commit instrument-to-Lean theorems (Variant A).
2. If relay hardware becomes available, automate cold-POR power-cycling and
   capture real STAT (Variant B).
3. Otherwise, keep hardening the formal model and the instrument-import path:
   add a PVT-envelope lower-bound regression test, extend VCD parser coverage for
   `$date`/`$version`/`$comment` headers and analog CSV voltage columns, build a
   standalone Lean integration test from a synthetic CSV, and document the
   first-real-capture checklist (Variant C).

---

## Work breakdown

### Variant A — Physical CCLK capture

Files: `fpga/HARDWARE_SSOT.md`, `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `docs/reports/*`

- Wire P12 to the logic analyzer.
- Generate `OSCFSEL=6` and `OSCFSEL=7` variants with `tri fpga cclk-variants`.
- Program each variant to flash, cold-POR boot, and capture CCLK.
- Import captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --pvt-context`.
- Commit generated theorems and update `fpga/HARDWARE_SSOT.md`.

### Variant B — Real relay-controlled cold-POR

Files: `cli/tri/src/fpga.rs`, `fpga/HARDWARE_SSOT.md`

- Select a relay interface (USB-serial relay module, smart power strip, or MCU GPIO bridge).
- Implement a `RelayControl` trait with `power_cycle(delay_ms: u64)`.
- Extend `FpgaCmd::ColdPor` to accept real `--relay-port` values.
- Capture STAT after relay power-cycle and write a non-mock log.
- Document wiring and safety rules.

### Variant C — Further formal tooling (default fallback)

Files: `cli/tri/src/fpga.rs`, `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `fpga/HARDWARE_SSOT.md`, `docs/reports/*`

1. **PVT-envelope regression test**
   - Add a Rust unit test that exhaustively samples the operating rectangle
     (temp, vccint, corner) and asserts
     `n25q128_min_sck_half_ns_pvt ≥ NOMINAL_HALF_NS`.
   - Add a Lean 4 lemma `n25q128_min_sck_half_ns_pvt_nonneg` proving the same
     bound symbolically.
   - Document in `fpga/HARDWARE_SSOT.md` how to replace the linear coefficients
     once real N25Q128_3V PVT curves are available.

2. **Instrument import coverage**
   - VCD parser: skip `$date`, `$version`, `$comment` multi-line header sections
     without confusing them with `$var` declarations.
   - CSV parser: support analog CSV exports that contain a voltage column
     directly, so `--vcd-threshold-v` is not the only analog path.
   - Add unit tests for both.

3. **Standalone Lean integration test**
   - Extend `measured-to-lean --standalone` output so the generated file is a
     complete, lake-buildable package snippet (imports + theorem + main proof).
   - Add a Rust integration test that writes a synthetic CSV, runs
     `tri fpga measured-to-lean --csv --raw-ns --standalone --pvt-context`, and
     type-checks the result in a temporary `lake` package.

4. **First-real-capture checklist**
   - Update `fpga/HARDWARE_SSOT.md` with a step-by-step checklist for the first
     real CCLK capture (wiring, samplerate, PVT context, validation,
     theorem generation).

---

## Weak points

1. **Physical evidence gap remains** under Variant C. The formal pipeline keeps
   improving, but no silicon evidence is produced until P12 is wired and the
   cable/relay hardware is available.
2. **PVT envelope coefficients are still informed estimates**, not Micron
   datasheet curves. The regression test only guards the lower bound; real
   characterization may require larger margins.
3. **VCD header sections** vary across tools (some emit `$date` on one line,
   others across multiple lines). Header handling will be best-effort for the
   most common variants.
4. **Analog CSV voltage columns** have no standard format; the parser will
   support the Saleae/DSView single-channel layout and document the expected
   column names.
5. **Standalone Lean integration test** depends on `lake` being available in
   the test environment. CI must have Lean 4 installed.

---

## Competitor scan

- **Sparkle HDL / Verilean:** formal Verilog-to-Lean/Coq verification; no public
  instrument-to-Lean bridge or PVT-aware timing envelope for flash boot.
- **SymbiYosys / Yosys formal:** bounded property checking on RTL; no link to
  logic-analyzer measurements.
- **Koika / Kami:** processor-model verification in Coq; unrelated to 7-series
  boot timing.
- **OpenFPGALoader / prjxray:** bitstream/JTAG tooling; no formal proof pipeline.
- **TinyTapeout / Efabless:** silicon-shuttle flow; relies on PDK
  characterization, not user-captured proofs.
- **OpenSTA / vyges-sta-si:** gate-level STA with PVT/OCV derating, but no
  instrument capture and no theorem-prover output.
- **t27 differentiation:** still the only open pipeline converting instrument
  exports (CSV/VCD) into Lean 4 proofs of flash timing compliance, now with an
  explicit, falsifiable PVT uncertainty envelope, envelope monotonicity proofs,
  per-OSCFSEL transaction theorems, and an integration-test path.

---

## Verification checklist

- [ ] `cargo test -p tri fpga::tests` passes (new + existing tests).
- [ ] `lake build Trinity.TernaryFPGABoot` passes from `proofs/lean4/`.
- [ ] `./scripts/tri test` passes parse/typecheck/gen/seal-verify.
- [ ] For Variant A: at least one generated `.lean` file builds standalone.
- [ ] For Variant B: real relay port produces a non-mock boot log.
- [x] For Variant C: PVT-envelope regression test, VCD header/analog CSV tests,
      and standalone Lean integration test land.

---

## Acceptance criteria

- The chosen variant is fully implemented and verified.
- All invariant checks pass.
- Report + evidence + W419 cooperation variants are produced.
- PR closes #1353.

---

*φ² + φ⁻² = 3 | TRINITY*
