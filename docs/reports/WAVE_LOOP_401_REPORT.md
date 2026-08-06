# Wave Loop 401 — Cold-POR protocol hardening & board-less CI guards

> **Issue:** [#1303](https://github.com/t27/t27/issues/1303)  
> **Branch:** `trinity-rust-rings`  
> **Date:** 2026-07-09  
> **Status:** implemented, board-less verification passed, physical CCLK capture deferred  
> **Conformance:** `575 / 575 PASS`

---

## 1. Goal

Make the cold-POR SPI-boot workflow reproducible and CI-friendly even when the
FPGA board is not connected. W400 proved that the canonical
`ternary_mac_demo_top_200t.bit` boots from flash once the cable-disconnect
protocol is followed; W401 hardens the tooling around that result so future
waves do not regress the protocol or the canonical bitstream configuration.

---

## 2. Acceptance criteria (AC)

| ID | Criterion | Status |
|----|-----------|--------|
| AC1 | `tri fpga smoke-gate` asserts canonical bitstream config (IDCODE, x1 SPI, CCLK startup, `OSCFSEL=0`, no CRC writes). | ✅ |
| AC2 | `tri fpga boot-protocol` documents the cold-POR checklist interactively and in `--checklist` mode. | ✅ |
| AC3 | `tri fpga measure-cclk --csv` is robust to DSView / PulseView / Saleae CSV formats. | ✅ |
| AC4 | Board-less CI guards exist for both the smoke-gate assertions and the dry-run CCLK sweep/report path. | ✅ |
| AC5 | If a physical CCLK capture becomes available, record frequency and duty cycle on pin P12. | ⏸️ deferred |

---

## 3. What changed

### 3.1 `scripts/dump_bit_config.py`

- Added `--assert-oscfsel N` and `--assert-no-crc-writes` flags.
- Refactored `run_assertions(registers, crc_writes, args)` so the CRC-write
count is available for assertion checks.
- Output now warns when CRC register writes are present in the bitstream,
which would be invalidated by `patch-cor0`.

### 3.2 `cli/tri/src/fpga.rs`

- Added `BootProtocol { checklist: bool }` subcommand with interactive and
  checklist-only modes.
- Extended `SmokeGate` to invoke `bit_config` with a fixed assertion set:
  ```
  --assert-idcode 0x03636093
  --assert-spi-x1
  --assert-cclk-startup
  --assert-oscfsel 0
  --assert-no-crc-writes
  ```
- Added a dry-run CCLK sweep guard inside `smoke_gate`:
  - cleans `build/fpga/smoke-gate-dry-run/`,
  - runs `cclk-sweep --dry-run` over the six canonical `OSCFSEL` values,
  - runs `sweep-report`,
  - asserts exactly six variant rows are produced.
- Improved `measure_cclk` CSV parsing:
  - renamed `parse_dsview_csv` → `parse_cclk_csv_reader`,
  - auto-detects DSView (`Time,Voltage`), PulseView (`Time,Channel 0,...`),
    and Saleae (`time, channel 0,...`) headers,
  - detects numeric time/value columns heuristically,
  - reports dominant frequency and duty cycle,
  - errors if fewer than 10 transitions are found.
- Added unit tests in `#[cfg(test)] mod tests` covering all three CSV formats
  and the too-few-samples error path.
- Added optional `--log-dir` to `CclkSweep` and `BootLog` and propagated it
  through `cclk_sweep`, `boot_log`, and `write_sweep_log`.

### 3.3 `fpga/HARDWARE_SSOT.md`

- Documented `tri fpga boot-protocol` and `tri fpga boot-protocol --checklist`
  in §3.4.
- Documented the `--log-dir` option for `cclk-sweep`.
- Updated §3.5 / §9.5 to describe multi-format CSV parsing and the Saleae
  header case.
- Added §9.3 for cold-POR protocol helpers and renumbered the CCLK tooling
  subsections.

### 3.4 Generated evidence

- `build/fpga/sweep-report-smoke-gate-dry-run.md` produced by the smoke-gate
  dry-run guard (gitignored build artifact; regenerated on every
  `./scripts/tri test`).

---

## 4. Verification

```bash
./scripts/tri test
```

Result: **575 / 575 PASS**.

The board-less guards mean W401 is fully CI-ready: every future `tri test` will
fail if the canonical bitstream assertions break or if the dry-run sweep/report
path stops producing six `OSCFSEL` rows.

---

## 5. What was not done and why

AC5 (physical CCLK measurement) requires attaching an oscilloscope or logic
analyzer to pin P12 and capturing the first ~100 µs after cold-POR. That is a
physical operation that cannot be performed autonomously in a headless session.
The tooling is ready; the measurement is left for the next loop when hardware is
on the bench.

---

## 6. Key learnings

1. **Protocol checklists should be executable.** `tri fpga boot-protocol` makes
   the human steps explicit and gives a printable `--checklist` that can be
   attached to lab notebooks.
2. **Smoke-gate assertions should cover the root-cause hypothesis, not just
   load success.** Adding `OSCFSEL=0` and CRC-write checks prevents silent
   regression of the W400 finding that the default bitstream is the working
   default.
3. **CSV parsers need format detection, not format assumptions.** DSView,
   PulseView, and Saleae all export different headers; the new parser treats the
   header as metadata and detects numeric columns.
4. **Dry-run paths are cheap CI guards.** By running the sweep/report pipeline
   on synthetic logs during `tri test`, the repo catches regressions in the
   evidence-generation path before anyone touches hardware.

---

## 7. Next loop (W402) targets

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-09.md` for three cooperation
variants. Likely candidates include:

- physical CCLK measurement on P12 and recording the result,
- a Lean 4 formalization of the CCLK/SPI timing decision tree,
- extending the `tri fpga smoke-gate` to also verify a GF16 SRAM load reaches
  `DONE=HIGH` when a cable is connected.

---

*φ² + 1/φ² = 3 | TRINITY*
