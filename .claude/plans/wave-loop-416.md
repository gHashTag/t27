# Wave Loop 416 Plan

**Issue:** #1347 (to be created after W415 land)  
**Branch:** `wave-loop-416`

---

## Decision gate

| Bench available? | Pick |
|--------------------|------|
| P12 + analyzer + DLC10 cable | Variant A |
| Relay / power switch + DLC10 cable | Variant B |
| Nothing | Variant C |

Current state (2026-07-01): **nothing** — P12 unwired, DLC10 cable missing, no USB relay detected. Confirmed by `dlc10 idcode` (`VID=0x03FD` not found). Default to **Variant C** for W416.

---

## Goals

1. If the bench becomes available, capture real CCLK for OSCFSEL=6/7 and commit instrument-to-Lean theorems (Variant A).
2. If relay hardware becomes available, automate cold-POR power-cycling and capture real STAT (Variant B).
3. Otherwise, continue hardening the formal model: add a PVT-envelope CLI helper, prove monotonicity of the derating functions, extend VCD parser coverage for escaped identifiers / x/z / hex bus literals, and link the OSCFSEL nominal theorems to `transaction_satisfies_flash_spec` proofs (Variant C).

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

1. **PVT-envelope CLI helper**: add `tri fpga pvt-envelope --pvt-context <ctx.json>` that prints the derated `t_CL`/`t_CH` bounds, the margin over the nominal 6 ns bound, and the operating-envelope validity status.
2. **PVT monotonicity lemmas** in Lean 4: prove the temperature and voltage derating functions are monotone inside the operating envelope, and that the process-corner derating increases from `ff` to `tt` to `ss`.
3. **VCD parser coverage**: handle escaped identifiers with embedded spaces (e.g. `\my sig `), scalar `x`/`z` transitions (skip them), and hex bus value changes (`hFF !`) with `x`/`z` bit skipping.
4. **OSCFSEL transaction theorems**: for each OSCFSEL 0..7, prove `transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz n) 50 bits) = true` using the existing implication theorems.
5. **Docs**: update `fpga/HARDWARE_SSOT.md` with the new `pvt-envelope` command and VCD parser capabilities.

---

## Weak points

1. **Physical evidence gap remains** under Variant C.
2. **PVT envelope coefficients are still informed estimates**, not Micron datasheet curves.
3. **Escaped VCD identifiers** are non-trivial to parse correctly with line-oriented tokenization; our handling will be best-effort for common cases.
4. **Hex bus literals** are not part of the IEEE VCD standard but are emitted by some tools; supporting them is pragmatic, not canonical.
5. **Relay automation** remains unimplemented until hardware is selected.

---

## Competitor scan

- **Sparkle HDL / Verilean:** formal Verilog-to-Lean/Coq verification; no public instrument-to-Lean bridge or PVT-aware timing envelope for flash boot.
- **SymbiYosys / Yosys formal:** bounded property checking on RTL; no link to logic-analyzer measurements.
- **Koika / Kami:** processor-model verification in Coq; unrelated to 7-series boot timing.
- **OpenFPGALoader / prjxray:** bitstream/JTAG tooling; no formal proof pipeline.
- **TinyTapeout / Efaless:** silicon-shuttle flow; relies on PDK characterization, not user-captured proofs.
- **t27 differentiation:** still the only open pipeline converting instrument exports (CSV/VCD) into Lean 4 proofs of flash timing compliance, now with a falsifiable PVT envelope, envelope monotonicity proofs, and per-OSCFSEL transaction theorems.

---

## Verification checklist

- [x] `cargo test -p tri fpga::tests` passes (new + existing tests).
- [x] `lake build Trinity.TernaryFPGABoot` passes from `proofs/lean4/`.
- [x] `./scripts/tri test` passes parse/typecheck/gen/seal-verify.
- [ ] For Variant A: at least one generated `.lean` file builds standalone.
- [ ] For Variant B: real relay port produces a non-mock boot log.
- [x] For Variant C: `tri fpga pvt-envelope` works, VCD tests cover escaped identifiers / x/z / hex, OSCFSEL transaction theorems build.

---

## Acceptance criteria

- The chosen variant is fully implemented and verified.
- All invariant checks pass.
- Report + evidence + W417 cooperation variants are produced.
- PR closes #1347.

---

*φ² + φ⁻² = 3 | TRINITY*
