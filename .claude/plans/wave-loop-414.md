# Wave Loop 414 Plan

**Issue:** #1342  
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
   hardens tooling but produces no new silicon evidence; the next wave still
   needs a real capture or relay cycle to close the loop to hardware.
2. **PVT envelope is an informed model, not a datasheet curve.** The N25Q128_3V
   datasheet gives nominal `t_CL`/`t_CH` = 5.5 ns but does not ship a public
   closed-form PVT derating. Our linear envelope must be conservative enough to
   stay valid if real curves are later published; otherwise the theorems become
   false.
3. **Multi-bit bus / real trace parsing is under-constrained.** A VCD bus may
   carry a clock on bit 0 or on every bit; the parser must either reject
   ambiguous buses or require an explicit bit index. Real-valued VCD nets need
   a user-supplied threshold to be treated as logic.
4. **Validation duplicates spec logic in Rust and Lean.** `--validate` must match
   the formal predicate; drift between the Rust guard and `measured_cclk_*_satisfies_flash_spec`
   would let a theorem be generated for an out-of-spec capture.
5. **CI still cannot exercise real hardware.** Mock paths and synthetic fixtures
   keep the code tested, but they do not prove that the parser tolerates real
   sigrok/DSView/VCD quirks (header variants, timestamp jumps, `$dumpoff`).
6. **Relay safety not yet modeled.** Even the formal model has no notion of
   power-cycle cadence limits; Variant B will need a safety policy before it
   can run unattended.

## Competitor scan

- **Sparkle HDL / Verilean:** formal Verilog-to-Lean/Coq verification, no public
  instrument-to-Lean bridge for FPGA boot timing or PVT-aware flash constraints.
- **SymbiYosys / Yosys formal:** bounded property checking on RTL, no link to
  logic-analyzer measurements or PVT uncertainty envelopes.
- **Koika / Kami:** processor-model verification in Coq, unrelated to 7-series
  configuration/boot timing or SPI flash constraints.
- **OpenFPGALoader / prjxray:** tooling for bitstream manipulation and JTAG, no
  formal proof pipeline for timing compliance.
- **TinyTapeout / Efabless:** silicon-shuttle flow, not a formal verification
  tool; their timing closure relies on PDK characterization, not user-captured
  logic-analyzer proofs.
- **t27 differentiation:** still the only open pipeline that converts sigrok /
  DSView / VCD instrument exports into machine-checked Lean 4 proofs of flash
  timing compliance, with an explicit, falsifiable PVT uncertainty envelope and
  an early-rejection validation gate.

## Verification checklist

- [x] `cargo test -p tri fpga::tests` passes (new + existing tests).
- [x] `lake build Trinity.TernaryFPGABoot` passes from `proofs/lean4/`.
- [x] `./scripts/tri test` passes (parse/typecheck/gen/seal-verify); 16 pre-existing gen-verilog-yosys-smoke failures unchanged.
- [ ] For Variant A: at least one generated `.lean` file builds standalone (deferred to W415 if bench becomes available).
- [ ] For Variant B: `tri fpga cold-por ... --relay-port <real>` produces a
      real log when hardware is connected (deferred to W415 if relay becomes available).
- [x] For Variant C: `tri fpga measured-to-lean --raw-ns --validate` rejects
      an out-of-spec fixture and accepts an in-spec fixture.

## Acceptance criteria

- [x] Variant C is fully implemented and verified.
- [x] Invariant checks pass (cargo test, lake build, tri test).
- [x] Report + evidence + W415 cooperation variants are produced.
- [ ] PR closes #1342 (final step after report).

---

*φ² + φ⁻² = 3 | TRINITY*
