# FPGA Loop Evidence — W404 (2026-07-07)

> Companion to `docs/reports/WAVE_LOOP_404_REPORT.md` (Issue [#1309](https://github.com/t27/t27/issues/1309)).  
> This file records the artifacts and commands that produced the W404 result.

---

## 1. Hardware state

Connected devices (from `ioreg -rc IOUSBHostDevice`):

```text
USB Product Name = Digilent USB Device
USB Vendor Name  = Digilent
```

JTAG chain detection:

```bash
openFPGALoader --detect -c digilent_hs2
```

```text
idcode 0x3636093
manufacturer xilinx
family artix a7 200t
model  xc7a200
irlength 6
```

---

## 2. Command artifacts

### 2.1 Board-less smoke gate (regression path)

```bash
cargo run --release -p tri -- fpga smoke-gate
```

Result:

```text
[smoke-gate] bit-config audit: ...
ASSERTION OK: IDCODE=0x03636093
ASSERTION OK: SPI_BUSWIDTH=x1
ASSERTION OK: STARTUPCLK=CCLK
ASSERTION OK: OSCFSEL=0
ASSERTION OK: no CRC register writes
[smoke-gate] dry-run sweep report OK (6 variants)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

### 2.2 Cable-connected smoke gate (new W404 path)

```bash
cargo run --release -p tri -- fpga smoke-gate --require-cable
```

Result:

```text
[smoke-gate] require-cable: detecting FPGA via digilent_hs2...
[smoke-gate] cable OK (FPGA detected)
[smoke-gate] loading SRAM: .../ternary_mac_demo_top_200t.bit
Done
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
[smoke-gate] reading STAT after SRAM load...
[smoke-gate] hardware check OK (DONE=HIGH, mode=001, no errors)
[smoke-gate] bit-config audit OK
[smoke-gate] dry-run sweep report OK (6 variants)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

Post-load STAT = `0x401079FC`, matching the Lean 4 `boot_success` example in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`.

---

## 3. Conformance suite

```bash
./scripts/tri test
```

Result:

```text
Parse: 576 passed, 0 failed
Typecheck: 576 passed, 0 failed
GF16: conformance OK
Gen Zig: 576 passed, 0 failed
Gen Rust: 576 passed, 0 failed
Gen Verilog: 576 passed, 0 failed
Gen Verilog Yosys Smoke: 56 passed, 0 failed
FPGA Board-Less Smoke Gate: OK
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
Fixed Point: 0 divergences
TOTAL FAILURES: 0
ALL TESTS PASSED
```

---

## 4. Source diff summary

- `cli/tri/src/fpga.rs`
  - `FpgaCmd::SmokeGate`: added `--require-cable`, `--cable`, `--part`.
  - Added `cable_detected`, `assert_stat_boot_success` helpers.
  - `smoke_gate` optionally performs cable detection, SRAM load, and STAT
    assertion before the existing board-less checks.
- `fpga/HARDWARE_SSOT.md`: hardware smoke traceability callout in §3.2.

---

## 5. Traceability

- Lean 4 predicate: `Trinity.StatRegister.boot_success`
- Prose decision tree: `fpga/HARDWARE_SSOT.md` §3.2
- Issue: [#1309](https://github.com/t27/t27/issues/1309)
- Branch: `trinity-rust-rings`

---

*φ² + 1/φ² = 3 | TRINITY*
