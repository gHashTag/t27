# FPGA Loop Evidence — W403 (2026-07-06)

> Companion to `docs/reports/WAVE_LOOP_403_REPORT.md` (Issue [#1307](https://github.com/t27/t27/issues/1307)).  
> This file records the artifacts and commands that produced the W403 result.

---

## 1. Formal artifact

**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Key declarations added in W403:

```lean
structure BitstreamConfig where
  idcode : UInt32
  spi_buswidth : UInt8
  startupclk : UInt8
  oscfsel : UInt8

def BitstreamConfig.canonical (cfg : BitstreamConfig) : Bool :=
  cfg.idcode = IDCODE_XC7A200T
  ∧ cfg.spi_buswidth = SPI_BUSWIDTH_X1
  ∧ cfg.startupclk = STARTUPCLK_CCLK
  ∧ cfg.oscfsel = OSCFSEL_DEFAULT

structure ColdPOR where
  cfg : BitstreamConfig
  mode_ok : Bool
  no_cable_interference : Bool

def cold_por_spi_flash_pred (p : ColdPOR) (s : StatRegister) : Bool :=
  p.cfg.canonical ∧ p.mode_ok ∧ p.no_cable_interference
  ∧ s.mode_master_spi_x1 ∧ ¬s.fatal_error

theorem cold_por_done_eos_high_implies_boot_success ...
theorem cold_por_done_low_implies_h2 ...
theorem decision_tree_exhaustive ...
```

---

## 2. Build commands

```bash
cd proofs/lean4
lake build Trinity.TernaryFPGABoot
```

**Result:** `✔ [2/2] Built Trinity.TernaryFPGABoot`

Full-library build (`lake build`) still fails in unrelated modules
(`Trinity.H4Lagrangian`, `Trinity.NeutrinoMasses`) that were failing before
W403.

---

## 3. Conformance suite

```bash
cd .
./scripts/tri test
```

**Result:**

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

## 4. Hardware status

- **Bench hardware:** not accessed in W403.
- **CCLK measurement on P12:** deferred to W404 (Variant A).
- **Cable-connected SRAM smoke:** not attempted (Variant C).

---

## 5. Traceability

- Formal predicates ↔ prose decision tree: `fpga/HARDWARE_SSOT.md` §3.2
- Issue: [#1307](https://github.com/t27/t27/issues/1307)
- Branch: `trinity-rust-rings`

---

*φ² + 1/φ² = 3 | TRINITY*
