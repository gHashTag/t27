# FPGA Loop Evidence — 2026-07-09 (W401)

> Wave Loop 401 issue: [#1303](https://github.com/t27/t27/issues/1303)  
> Branch: `trinity-rust-rings`

---

## Board-less verification

```bash
$ ./scripts/tri test
...
575 / 575 PASS
```

This confirms that every acceptance criterion that can be tested without the
physical board passes:

- `tri fpga smoke-gate` asserts the canonical bitstream configuration.
- `tri fpga boot-protocol --checklist` prints the cold-POR checklist.
- `tri fpga measure-cclk --csv` parses DSView, PulseView, and Saleae exports.
- The smoke-gate dry-run path produces a six-row `sweep-report`.

## Canonical bitstream assertions

`tri fpga smoke-gate` now runs the bitstream through:

```
--assert-idcode 0x03636093
--assert-spi-x1
--assert-cclk-startup
--assert-oscfsel 0
--assert-no-crc-writes
```

This pins the proven W400 result: the default `OSCFSEL=0` bitstream is the
working default, and any CRC-register writes in the bitstream are flagged before
a `patch-cor0` step can invalidate them.

## CSV parser coverage

Unit tests in `cli/tri/src/fpga.rs` cover:

| Format | Header | Result |
|--------|--------|--------|
| DSView | `Time,Voltage` | frequency ~1 MHz, duty cycle matched |
| PulseView | `Time,Channel 0,Channel 1` | frequency ~1 MHz, duty cycle matched |
| Saleae | `time, channel 0, channel 1` | frequency ~1 MHz, duty cycle matched |
| Too few transitions | — | returns error |

## Physical measurement

No physical CCLK capture was performed. Pin P12 (CFGCLK / CCLK_0) remains the
documented target; the measurement is deferred to W402 when hardware is
available.

---

*φ² + 1/φ² = 3 | TRINITY*
