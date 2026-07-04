# FPGA Loop Evidence — 2026-07-05 (W402)

> Wave Loop 402 issue: [#1305](https://github.com/t27/t27/issues/1305)  
> Branch: `trinity-rust-rings`

---

## Lean 4 build

```bash
$ cd proofs/lean4
$ lake build Trinity.TernaryFPGABoot
✔ Built Trinity.TernaryFPGABoot
```

## Conformance suite

```bash
$ ./scripts/tri test
...
576 / 576 PASS
ALL TESTS PASSED
```

## Formal coverage

`proofs/lean4/Trinity/TernaryFPGABoot.lean` contains:

| Predicate / Lemma | Role |
|-------------------|------|
| `StatRegister` | 32-bit STAT register wrapper |
| `mode`, `done`, `eos`, `crc_error`, `id_error`, `dec_error`, `bus_width` | Named bit-field decoders |
| `mode_master_spi_x1` | Master SPI x1 mode predicate |
| `boot_success` | Cold-POR success predicate |
| `h2_cclk_timing` | CCLK/SPI timing hypothesis bucket |
| `mode_mismatch` | Mode-pin strapping issue predicate |
| `fatal_error` | CRC / ID / DEC error aggregate |
| `boot_success_implies_mode_master_spi_x1` | Success → correct mode |
| `boot_success_implies_no_fatal_error` | Success → no fatal errors |
| `h2_implies_mode_ok_done_low` | H2 → mode OK, DONE LOW |
| `fatal_error_implies_not_boot_success` | Fatal error prevents success |
| `mode_mismatch_implies_not_boot_success` | Wrong mode prevents success |
| `stat_success_example_boots` | `0x401079FC` is a success instance |
| `stat_incomplete_example_is_h2` | `0x5000190C` is an H2 instance |
| `boot_success_and_h2_disjoint` | Success and H2 are mutually exclusive |

## Physical measurement

No oscilloscope / logic-analyser capture was performed. Pin P12 remains the
documented target; the measurement is deferred to W403 when hardware is
available.

---

*φ² + 1/φ² = 3 | TRINITY*
