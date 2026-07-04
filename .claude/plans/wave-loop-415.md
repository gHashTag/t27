# Wave Loop 415 Plan

**Issue:** #1343  
**Branch:** `wave-loop-415`

## Decision gate

| Bench available? | Pick |
|--------------------|------|
| P12 + analyzer | Variant A |
| Relay / power switch + cable | Variant B |
| Nothing | Variant C |

Current state (2026-07-01): **nothing** — P12 unwired, DLC10 cable missing, no USB relay detected. Confirmed by `dlc10 idcode` (`VID=0x03FD` not found) and `system_profiler SPUSBDataType` (no FTDI/relay/serial signatures). Default to **Variant C** for W415.

Execution status: **Variant C completed**.

## Goals

1. If the bench is available, turn real instrument captures into committed Lean theorems (Variant A).
2. If relay hardware is available, automate cold-POR power-cycling and capture real STAT (Variant B).
3. Otherwise, continue hardening the formal model so it is ready for real data: PVT-aware validation, richer VCD parser tests, and a library of OSCFSEL-specific theorems (Variant C).

## Work breakdown

### Variant A — Physical CCLK capture

Files: `fpga/HARDWARE_SSOT.md`, `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `docs/reports/*`

- Wire P12 (CFGCLK / CCLK_0) to the DSLogic Plus or sigrok-compatible analyzer.
- Generate `OSCFSEL=6` and `OSCFSEL=7` bitstream variants with `tri fpga cclk-variants`.
- Program each variant to flash and run `tri fpga boot-log` cold-POR.
- Capture CCLK during configuration via `tri fpga measure-cclk --live` or DSView export.
- Import each capture with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --out W415_OSCFSEL6.lean`.
- Add the generated theorems to `proofs/lean4/Trinity/TernaryFPGABoot.lean` or commit them as standalone files.
- Update `fpga/HARDWARE_SSOT.md` with measured frequencies and duty cycles.

### Variant B — Real relay-controlled cold-POR

Files: `cli/tri/src/fpga.rs`, `fpga/HARDWARE_SSOT.md`

- Select a relay interface (USB-serial relay module, smart power strip with local API, or microcontroller GPIO bridge).
- Implement a small `RelayControl` trait with `power_cycle(delay_ms: u64)`.
- Extend `FpgaCmd::ColdPor` so `--relay-port` accepts real port strings (`/dev/cu.usbserial-*`, `tcp://...`) in addition to `MOCK`.
- After relay power-cycle, run `capture_stat` and write a real boot log with `relay_mock: false`.
- Add Rust tests for the relay protocol framing (without touching hardware).
- Document relay wiring, port syntax, and safety rules in `fpga/HARDWARE_SSOT.md`.

### Variant C — Further formal tooling (default fallback)

Files: `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `cli/tri/src/fpga.rs`

- Integrate the PVT envelope into `tri fpga measure-cclk --validate` so live/CSV/VCD paths can check against PVT-margin bounds.
- Add `--pvt-context <json>` option to `measured-to-lean` so a user-supplied `PvtContext` is attached to generated theorems.
- Extend VCD parser unit tests for real-world quirks: multi-line `$var` declarations, mixed scalar/bus dumps, timestamp jumps, `$dumpoff`/`$dumpon` regions.
- Build a library of measured-CCLK theorems for every documented Artix-7 OSCFSEL value (0..7) under both nominal and worst-case PVT contexts.

## Weak points

1. **Physical evidence gap remains** if the bench is still blocked. Variant C hardens tooling but produces no new silicon evidence.
2. **PVT envelope coefficients are informed estimates**, not Micron datasheet curves. A real characterization could force us to raise them.
3. **Real relay control is safety-sensitive** — automating board power requires clear safeguards against rapid cycling or unintended reconnections.
4. **VCD parser quirks** from real instruments (header variants, escaped identifiers, `$dumpvars` initial values) may not all be covered by synthetic fixtures.
5. **Generated `.lean` files** for Variant A may be large; decide whether to commit them as standalone files under `proofs/lean4/Trinity/Measured/`.

## Competitor scan

- **Sparkle HDL / Verilean:** formal Verilog-to-Lean/Coq verification, no public instrument-to-Lean bridge for FPGA boot timing or PVT-aware flash constraints.
- **SymbiYosys / Yosys formal:** bounded property checking on RTL, no link to logic-analyzer measurements or PVT uncertainty envelopes.
- **Koika / Kami:** processor-model verification in Coq, unrelated to 7-series configuration/boot timing or SPI flash constraints.
- **OpenFPGALoader / prjxray:** tooling for bitstream manipulation and JTAG, no formal proof pipeline for timing compliance.
- **TinyTapeout / Efabless:** silicon-shuttle flow, not a formal verification tool; their timing closure relies on PDK characterization, not user-captured logic-analyzer proofs.
- **t27 differentiation:** still the only open pipeline that converts sigrok / DSView / VCD instrument exports into machine-checked Lean 4 proofs of flash timing compliance, with an explicit, falsifiable PVT uncertainty envelope and an early-rejection validation gate.

## Verification checklist

- [x] `cargo test -p tri fpga::tests` passes (32/32).
- [x] `lake build Trinity.TernaryFPGABoot` passes from `proofs/lean4/`.
- [x] `./scripts/tri test` passes seal-verify and all code-generation phases.
- [ ] For Variant A: deferred until P12 / analyzer available.
- [ ] For Variant B: deferred until relay hardware available.
- [x] For Variant C: PVT-aware validation, VCD parser tests (multi-line var, mixed scalar/bus, dumpoff), and OSCFSEL 0..7 nominal/worst-case theorem library landed.

## Acceptance criteria

- The chosen variant is fully implemented and verified.
- All invariant checks pass.
- Report + evidence + W416 cooperation variants are produced.
- PR #1346 closes #1343.

---

*φ² + φ⁻² = 3 | TRINITY*
