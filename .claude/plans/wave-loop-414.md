# Wave Loop 414 Plan

**Issue:** #1339  
**Branch:** `wave-loop-414`

## Decision gate

| Bench available? | Pick |
|--------------------|------|
| P12 + analyzer | Variant A |
| Relay / power switch + cable | Variant B |
| Nothing | Variant C |

Current state (2026-07-04): **nothing** — P12 unwired, DLC10 cable missing. Default to **Variant C** unless the bench state changes at the start of the wave.

## Goals

1. If the bench is available, turn real instrument captures into committed Lean theorems (Variant A).
2. If relay hardware is available, automate cold-POR power-cycling and capture real STAT (Variant B).
3. Otherwise, harden the formal model so it is ready for real data: PVT envelope, richer VCD parsing, and early validation (Variant C).

## Work breakdown

### Variant A — Physical CCLK capture

Files: `fpga/HARDWARE_SSOT.md`, `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `docs/reports/*`

- Wire P12 (CFGCLK / CCLK_0) to the DSLogic Plus or sigrok-compatible analyzer.
- Generate `OSCFSEL=6` and `OSCFSEL=7` bitstream variants with `tri fpga cclk-variants`.
- Program each variant to SPI flash and run `tri fpga boot-log` cold-POR.
- Capture CCLK during configuration via `tri fpga measure-cclk --live` or DSView export.
- Import each capture with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --out W414_OSCFSEL6.lean`.
- Add the generated theorems to `proofs/lean4/Trinity/TernaryFPGABoot.lean` or commit them as standalone files.
- Update `fpga/HARDWARE_SSOT.md` with measured frequencies and duty cycles.

### Variant B — Real relay-controlled cold-POR

Files: `cli/tri/src/fpga.rs`, `fpga/HARDWARE_SSOT.md`

- Select a relay interface (USB-serial relay module, smart power strip with
  local API, or a microcontroller GPIO bridge).
- Implement a small `RelayControl` trait with `power_cycle(delay_ms: u64)`.
- Extend `FpgaCmd::ColdPor` so `--relay-port` accepts real port strings
  (`/dev/cu.usbserial-*`, `tcp://...`) in addition to `MOCK`.
- After relay power-cycle, run `capture_stat` through openFPGALoader and
  write a real boot log with `relay_mock: false`.
- Add Rust tests for the relay protocol framing (without touching hardware).
- Document relay wiring, port syntax, and safety rules in
  `fpga/HARDWARE_SSOT.md`.

### Variant C — Further formal tooling (default fallback)

Files: `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `cli/tri/src/fpga.rs`

- Replace `n25q128_min_sck_*_ns_pvt` constants with a function of
  `temp_c`, `vccint_mv`, and `process_corner`.
  - Define an operating envelope (e.g. temp -40..+85 °C, VCCINT 0.95..1.05 V).
  - Derive conservative `t_CL`/`t_CH` bounds from the N25Q128_3V datasheet
    graphs or from a stated linear/spline envelope.
  - Prove the envelope is ≥ nominal 6 ns under the envelope.
- Extend `parse_vcd_to_raw_ns`:
  - Handle multi-bit bus identifiers (`b0001_!`).
  - Skip real-valued analog nets unless a threshold is supplied.
  - Support `$dumpoff` / `$dumpon` gracefully.
- Add `--validate` to `measured-to-lean --raw-ns`:
  - Before generating the theorem, check the raw-ns triple against the flash
    spec (or PVT-margin spec if `--margin`).
  - Fail with a clear message if the capture is out of spec.
- Add example theorems for nominal OSCFSEL=6 (40 ns period) and OSCFSEL=7
  (~30 ns period) raw-ns captures so the lattice is ready.

## Weak points

1. **Physical evidence gap remains** if the bench is still blocked. Variant C
   hardens tooling but produces no new silicon evidence.
2. **N25Q128 PVT data may be graph-only** in the datasheet, requiring manual
   digitization or conservative interpolation.
3. **Real relay control is safety-sensitive** — automating board power requires
   clear safeguards against rapid cycling or unintended reconnections.
4. **VCD multi-bit bus parsing** can be ambiguous (vector identifiers, radix
   prefixes); the parser must stay minimal enough to be trustworthy.
5. **Variant A generated `.lean` files** may not belong in the main
   `TernaryFPGABoot.lean` if they are large; decide whether to commit them as
   standalone files under `proofs/lean4/Trinity/Measured/`.

## Competitor scan

- **Sparkle HDL / Verilean:** formal HDL verification, no public
  instrument-to-Lean bridge for FPGA boot timing or PVT-aware flash constraints.
- **SymbiYosys / Yosys formal:** bounded property checking on Verilog, no
  link to logic-analyzer measurements.
- **Koika / Kami:** processor-model verification in Coq, unrelated to 7-series
  configuration/boot timing.
- **t27 differentiation:** still the only open pipeline that converts sigrok /
  DSView / VCD instrument exports into machine-checked Lean 4 proofs of flash
  timing compliance, with explicit PVT uncertainty.

## Verification checklist

- [ ] `cargo test -p tri fpga::tests` passes (new + existing tests).
- [ ] `lake build Trinity.TernaryFPGABoot` passes from `proofs/lean4/`.
- [ ] `./scripts/tri test` passes (parse/typecheck/gen/seal-verify).
- [ ] For Variant A: at least one generated `.lean` file builds standalone.
- [ ] For Variant B: `tri fpga cold-por ... --relay-port <real>` produces a
      real log when hardware is connected.
- [ ] For Variant C: `tri fpga measured-to-lean --raw-ns --validate` rejects
      an out-of-spec fixture and accepts an in-spec fixture.

## Acceptance criteria

- The chosen variant is fully implemented and verified.
- All invariant checks pass.
- Report + evidence + W415 cooperation variants are produced.
- PR closes #1339.

---

*φ² + φ⁻² = 3 | TRINITY*
