# Persistent SPI Flash Workflow — XC7A100T (QMTech Wukong V1)

> **The grain agents missed for 3 months:** `--write-flash`, NOT `--program`.
>
> **Now via pure-Rust `dlc10`** — `flash-spi` no longer shells out to
> `openFPGALoader`. It calls `dlc10::Dlc10::program_flash` directly, so the
> only host-side requirement is `libusb`.

## TL;DR

```bash
# ONE TIME (until you change the bitstream):
cargo run --release -p flash-spi -- fpga/vsa/gf16_heartbeat_top.bit
# physically unplug DLC10 → power-cycle FPGA → D5/D6 keep blinking forever
```

That's it. The bitstream lives in the on-board M25P/N25Q SPI flash and is
re-loaded by the FPGA itself within ~100 ms after every power-up. The JTAG
cable is **not** needed during normal operation.

## Why two modes exist

| Mode                | Where the bitstream lives    | Survives power-off | Time    |
|---------------------|------------------------------|--------------------|---------|
| **SRAM (volatile)** | inside the FPGA fabric       | ❌ no              | seconds |
| **SPI flash (NV)**  | M25P/N25Q chip on the board  | ✅ yes, forever    | ~60 s   |

`tools/dlc10_jtag.py --program file.bit` writes to **SRAM** — fine for
development, but the bitstream dies the moment power is cut. The new
`flash-spi` Rust binary writes to **flash** — survives forever.

## Prerequisites

- **Hardware**: DLC10 plugged into Mac USB AND into the JTAG Header on
  Wukong V1. FPGA powered (5V switch ON).
- **Mode pins M[2:0] = 0b001** (Master SPI). Default on Wukong V1.
- **Constraints in `.xdc`** (already present in
  `fpga/vsa/gf16_heartbeat_top.xdc`):
  ```tcl
  set_property CFGBVS VCCO [current_design]
  set_property CONFIG_VOLTAGE 3.3 [current_design]
  set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 4 [current_design]
  set_property BITSTREAM.CONFIG.CONFIGRATE 33 [current_design]
  set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
  ```

## What the binary does

1. Pre-flight: validates `.bit` exists and is readable.
2. Opens the DLC10 cable (loading FX2 firmware on first attach), reads
   `IDCODE` and aborts if it does not match `0x13631093` (XC7A100T).
3. Loads the embedded `bscan_spi_xc7a100t.bit` JTAG-to-SPI bridge into
   FPGA SRAM (UG470 §6 sequence with `JPROGRAM`), then drives the
   M25P/N25Q SPI flash via `USER1`:
   sector-erase → page-program → optional read-back verify → `JPROGRAM`.
4. On success prints next-steps for the operator.

Total wall-clock: ~60 s on a 3.6 MiB compressed Artix-7 bitstream.

## Verification after flash

1. Unplug DLC10 from JTAG Header.
2. Toggle 5V switch off, wait 2 s, switch on.
3. Within ~100 ms LEDs **D5 (R23)** and **D6 (T23)** must show the
   3-phase phi heartbeat (slow → steady → fast) — same pattern as SRAM mode,
   but now **without any cable connected**. That proves flash is alive.

If D5/D6 stay dark:
- Check M[2:0] strap resistors are 001.
- Re-run `flash-spi` and watch for verify errors.
- Read back: `openFPGALoader --cable dlc10 --read-flash dump.bin --read-len 4194304`.

## Reflashing

Plug DLC10 back in, run `cargo run --release -p flash-spi -- new.bit`,
unplug. Done.

## Flags

```text
flash-spi [BIT]                    # default: fpga/vsa/gf16_heartbeat_top.bit
  --expected-idcode <hex>          # default: 13631093 (XC7A100T)
  --skip-detect                    # skip cable detection
  --no-verify                      # skip read-back verification
  --dry-run                        # describe intent and exit
```

## Files

- `cli/flash-spi/` — Rust binary, this is what you run.
- `fpga/vsa/gf16_heartbeat_top.bit` — current golden bitstream
  (3-phase phi heartbeat on D5/D6, R23/T23 active-low).
- `fpga/vsa/gf16_heartbeat_top.xdc` — constraints incl. flash settings.
- `tools/dlc10_jtag.py` — reverse-engineered DLC10 USB driver (SRAM only,
  legacy; kept for SRAM-only workflows).
