# Wave Loop 401 — Decomposed Plan

**Issue:** #1303  
**Branch:** `trinity-rust-rings`  
**Goal:** Measure/record CCLK frequency, harden cold-POR protocol, extend FPGA
smoke gate, and close W401.

---

## 1. Weak points

### 1.1 Physical blocker
- **Actual CCLK measurement requires a logic analyser / oscilloscope on pin P12.**
  This cannot be done autonomously. Variant A is blocked until the operator has
  hardware access.

### 1.2 Missing board-less guards (AC2/AC3)
- `tri fpga smoke-gate` does **not** assert `OSCFSEL=0`. A patched COR0 variant
  could accidentally be committed as the default.
- `tri fpga smoke-gate` does **not** assert the absence of CRC register writes,
  which would be invalidated by a future `patch-cor0` sweep.
- There is **no standalone `tri fpga boot-protocol` command** that prints the
  exact cold-POR checklist; the protocol is currently embedded inside `boot-log`
  and `cclk-sweep`.

### 1.3 Measurement-tool gaps
- `tri fpga measure-cclk --csv` only supports the DSView two-column format.
  PulseView/Saleae exports have different headers and may fail silently.
- No unit tests cover `parse_dsview_csv`, so CSV-format regressions are only
  caught when a real trace is parsed.

### 1.4 Process risks
- The cold-POR protocol is user-assisted; a skipped step (cable still attached,
  insufficient power-off time) produces false `DONE=0` results and revives the
  H2 hypothesis.
- Physical iterations are slow (~2 min per OSCFSEL value). Automating the
  checklist reduces operator error.

---

## 2. Competitor scan

| Competitor | Relevant capability | Gap vs. t27 W401 |
|------------|---------------------|------------------|
| **openFPGALoader** | JTAG/SPI programming, `--freq` sets JTAG TCK, `--read-register STAT` | No CCLK measurement, no cold-POR protocol automation |
| **openXC7 / nextpnr-xilinx** | Open-source xc7 bitstream generation | No documented `ConfigRate`/`OSCFSEL` knob; COR0 patching is external |
| **LiteX / LiteSPI** | SPI flash boot, runtime SPI divisor calibration | SoC-centric; no bare-metal cold-POR/STAT capture tooling |
| **Xilinx Vivado** | `BITSTREAM.CONFIG.CONFIGRATE`, full register control | Closed-source, no native macOS, requires Docker entitlement |
| **F4PGA (ex-SymbiFlow)** | Deprecated xc7 flow | Replaced by openXC7 ecosystem for xc7 |
| **Sparkle HDL / Verilean** | Formal verification competitors | No physical board boot evidence |

**Key defense:** t27 is the only open-source flow that combines openXC7 bitstream
 generation, openFPGALoader flash programming, a disciplined cold-POR protocol,
 JSON evidence logs, and a board-less smoke gate. W401 closes the remaining
 board-less gaps so the physical evidence is reproducible and protected from
 regression.

---

## 3. Work breakdown

### 3.1 `scripts/dump_bit_config.py`
- Add `--assert-oscfsel N` flag; fail if `COR0[22:17] != N`.
- Add `--assert-no-crc-writes` flag; fail if any CRC register (0x00) write is
  present in the bitstream.
- Keep existing `--assert-idcode`, `--assert-spi-x1`, `--assert-cclk-startup`.

### 3.2 `cli/tri/src/fpga.rs`
- Extend `SmokeGate` to call `bit_config` with the new assertions for the
  canonical 200T bitstream:
  - `IDCODE=0x03636093`
  - `SPI_BUSWIDTH=x1`
  - `STARTUPCLK=CCLK`
  - `OSCFSEL=0`
  - no CRC writes.
- Add `BootProtocol` subcommand with:
  - `checklist` — print the cold-POR protocol.
  - `confirm` — interactive confirmation of each step (for manual sessions).
- Improve `measure_cclk` CSV parser:
  - Accept DSView (`Time,Voltage`), PulseView (`samplerate,...` or `Time,Channel 0`),
    and Saleae (`time, channel 0`) headers.
  - Auto-detect the first numeric time/voltage column pair.
- Add Rust unit tests for `parse_dsview_csv` with synthetic CSV fixtures.

### 3.3 Board-less CI guard
- Add a lightweight CI step that runs:
  ```bash
  tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit --dry-run
  tri fpga sweep-report --out build/fpga/sweep-report-dry-run.md
  ```
  and asserts the report contains the expected number of synthetic variants.
  This is added to the existing FPGA smoke gate in `bootstrap/src/suite.rs`.

### 3.4 Documentation
- Update `fpga/HARDWARE_SSOT.md` with the new `boot-protocol` and extended
  `smoke-gate` behavior.
- Add a W401 result section once CCLK is measured (placeholder until physical
  capture).
- Write `docs/reports/WAVE_LOOP_401_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-09.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_2026-07-09.md` with W402 variants.

### 3.5 Conformance & land
- Run `./scripts/tri test` → 575/575 PASS.
- Run `cargo test -p tri` for new unit tests.
- Commit, PR, squash-merge to `trinity-rust-rings`, close #1303.
- Create W402 issue.

---

## 4. Acceptance criteria

- [ ] AC1: `tri fpga smoke-gate` asserts `OSCFSEL=0` and no CRC writes.
- [ ] AC2: `tri fpga boot-protocol --checklist` prints the cold-POR steps.
- [ ] AC3: `tri fpga measure-cclk --csv` parses DSView, PulseView, and Saleae CSV
      exports.
- [ ] AC4: Unit tests for CSV parsing pass.
- [ ] AC5: Board-less dry-run sweep + report path is exercised in CI/smoke gate.
- [ ] AC6: `./scripts/tri test` passes (575/575).
- [ ] AC7: W401 report + evidence + W402 cooperation variants committed.
- [ ] AC8: If a physical CCLK capture is available, pin P12 frequency is recorded
      in `fpga/HARDWARE_SSOT.md`.

---

## 5. Default execution order

1. Extend `dump_bit_config.py` assertions.
2. Extend `smoke_gate` and add `boot-protocol` command.
3. Improve `measure_cclk` CSV parsing + tests.
4. Add dry-run sweep guard to FPGA smoke gate.
5. Update docs and reports.
6. Run conformance, commit, PR, merge.
