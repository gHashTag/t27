# FPGA Loop Evidence — W405 (2026-07-10)

> Companion to `docs/reports/WAVE_LOOP_405_REPORT.md` (Issue [#1311](https://github.com/t27/t27/issues/1311)).  
> This file records the exact commands and artifacts that produced the W405 flash-boot result.

---

## 1. Hardware state

- **Board:** QMTech Wukong V1 / XC7A200T-FGG676-1
- **Cable:** Digilent FTDI (`digilent_hs2` profile)
- **Host:** macOS arm64
- **Date:** 2026-07-10

JTAG chain detection:

```bash
./target/debug/tri fpga smoke-gate --require-cable --flash-boot --wait-seconds 120
```

```text
[smoke-gate] require-cable: detecting FPGA via digilent_hs2...
[openfpgaloader] $ /opt/homebrew/bin/openFPGALoader -c digilent_hs2 --detect
empty
Jtag frequency : requested 6.00MHz    -> real 6.00MHz
index 0:
    idcode 0x3636093
    manufacturer xilinx
    family artix a7 200t
    model  xc7a200
    irlength 6

[smoke-gate] cable OK (FPGA detected)
```

---

## 2. Bitstream

Canonical bitstream used for the flash-boot gate:

```text
fpga/verilog/ternary_mac_demo_top_200t.bit
```

Bit-config audit asserts:

- `IDCODE=0x03636093`
- `SPI_BUSWIDTH=x1`
- `STARTUPCLK=CCLK`
- `OSCFSEL=0`
- no CRC register writes

The smoke gate internally patches an `OSCFSEL=0` variant (identical MD5 to the
source when the source already has `OSCFSEL=0`) and programs that variant to
flash.

---

## 3. Flash program + verify

`tri fpga smoke-gate --flash-boot` programs flash via openFPGALoader's
JTAG-to-SPI bridge:

```text
[program-flash] bitstream expects SPI x1; ensure the flash QE bit and board straps match
[openfpgaloader] $ /opt/homebrew/bin/openFPGALoader -c digilent_hs2 -f --freq 6000000 --fpga-part xc7a200tfgg676 --verify build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel00.bit
...
Erasing: [==================================================] 100.00%
Writing: [==================================================] 100.00%
Reading: [==================================================] 100.00%
Done

[program-flash] Write complete. Reset or power-cycle the board to load from flash.
```

Flash verify passed (read-back matched write).

---

## 4. Cold-POR protocol

The operator followed the prompt printed by `cclk_sweep` inside the smoke gate:

```text
[cclk-sweep] PHYSICAL POWER-CYCLE REQUIRED
  1. Disconnect the JTAG/programming cable from the board.
     (An attached cable can hold TMS/TCK/PROGRAM_B and corrupt cold-POR
      mode sampling. See AR66954 / XAPP1188.)
  2. Disconnect the board's USB power / barrel jack.
  3. Wait at least 10 seconds for all rails to collapse.
  4. Reconnect power.
  5. Do NOT press the FPGA's PROG_B or RESET button.
  6. Wait at least 2 seconds, then reconnect the JTAG cable.
  7. Auto-continuing after 120 seconds (press ENTER to continue early).
```

---

## 5. Cold-POR STAT capture

After the power-cycle, the gate captured `STAT` without JTAG reset:

```text
[stat] reading STAT without JTAG reset/PROGRAM_B pulse
[openfpgaloader] $ /opt/homebrew/bin/openFPGALoader -c digilent_hs2 --read-register STAT --skip-reset
...
Register raw value: 0x401079fc
...
[stat] sample 1/3: raw=0x401079FC
[stat] sample 2/3: raw=0x401079FC
[stat] sample 3/3: raw=0x401079FC
```

Decoded fields:

| Field | Value | Meaning |
|---|---|---|
| DONE | 1 | FPGA configured successfully |
| MODE | `0b001` | Master SPI x1 |
| EOS | 1 | End-of-Startup reached |
| INIT_COMPLETE | 1 | Configuration initialization complete |
| INIT_B | 1 | INIT_B released high |
| Release Done | 1 | Done pin released |
| GTS_CFG_B | 1 | I/O released from configuration |
| GWE | 1 | Global write enable asserted |
| GHIGH_B | 1 | Global high impedance released |
| CRC Error | No CRC error | Bitstream integrity OK |
| ID Error | No ID error | IDCODE check OK |
| DEC Error | 0 | No decrypt error |

This matches the Lean 4 `boot_success` predicate:

```lean
def boot_success (stat : StatRegister) : Prop :=
  stat.done = true ∧
  stat.mode = 0b001 ∧
  stat.eos = true ∧
  stat.crc_error = false ∧
  stat.id_error = false ∧
  stat.dec_error = false
```

---

## 6. Smoke gate conclusion

```text
=> First working variant: OSCFSEL=0 (build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel00.bit)
   Next: measure actual CCLK with `tri fpga measure-cclk` and commit this variant as the default.
[smoke-gate] flash-boot check OK (DONE=HIGH, mode=001, no errors)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

---

## 7. Conformance suite

Board-less verification still passes:

```bash
./scripts/tri test
```

```text
Gen Verilog Yosys Smoke: 56 passed, 0 failed
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
TOTAL FAILURES: 0
ALL TESTS PASSED
phi^2 + phi^-2 = 3 | TRINITY
```

---

## 8. Notes

- The direct `program_flash()` + `capture_stat()` sequence returned
  `H2_CCLK_TIMING` (`STAT=0x5000190C`) repeatedly during W405 development,
  even with identical operator actions. Reusing the `cclk_sweep` cold-POR path
  resolved the issue.
- The `OSCFSEL=0` variant is bit-identical (same MD5) to the source bitstream,
  confirming that the canonical config already uses the default/internal CCLK
  oscillator.
- No PROG_B or RESET button was pressed during the cold-POR.

---

*phi^2 + phi^-2 = 3 | TRINITY*
