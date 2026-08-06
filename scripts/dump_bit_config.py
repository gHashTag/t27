#!/usr/bin/env python3
"""Parse a Xilinx 7-series .bit file and dump configuration registers.

Reads the ASCII header, locates the 0xAA995566 sync word, then walks the
configuration packets looking for Type-1 writes to the registers that govern
Master SPI boot:

  COR0  (0x09)  CCLK frequency, startup clock source, DONE/GTS/GWE release
  COR1  (0x0e)  SPI_BUSWIDTH
  WBSTAR(0x10)  warm-boot start address
  TIMER (0x11)  watchdog / configuration timer
  CTL0  (0x05)  control register 0
  CTL1  (0x18)  control register 1
  IDCODE(0x0c)  device IDCODE
  BSPI  (0x1f)  SPI configuration register
  CRC   (0x00)  expected CRC (diagnostic only)

Field decoding follows UG470 "7 Series FPGAs Configuration" and the prjxray
Series7ConfigurationRegister enum.
"""

import argparse
import struct
import sys

SYNC = b'\xaa\x99\x55\x66'

# Register address to human-readable name.
REG_NAMES = {
    0x00: "CRC",
    0x01: "FAR",
    0x02: "FDRI",
    0x03: "FDRO",
    0x04: "CMD",
    0x05: "CTL0",
    0x06: "MASK",
    0x07: "STAT",
    0x08: "LOUT",
    0x09: "COR0",
    0x0a: "MFWR",
    0x0b: "CBC",
    0x0c: "IDCODE",
    0x0d: "AXSS",
    0x0e: "COR1",
    0x10: "WBSTAR",
    0x11: "TIMER",
    0x13: "UNKNOWN",
    0x16: "BOOTSTS",
    0x18: "CTL1",
    0x1f: "BSPI",
}

# Registers we decode and possibly assert on.
INTERESTING = {0x05, 0x09, 0x0c, 0x0e, 0x10, 0x11, 0x18, 0x1f}


def extract_bitstream_words(data: bytes):
    """Return the 32-bit big-endian words after the sync word."""
    sync_idx = data.find(SYNC)
    if sync_idx < 0:
        raise ValueError("sync word 0xAA995566 not found")
    tail = data[sync_idx + len(SYNC):]
    # Trim to a multiple of 4 bytes.
    if len(tail) % 4:
        tail = tail[:-(len(tail) % 4)]
    return list(struct.unpack(">%dI" % (len(tail) // 4), tail))


def parse_packets(words):
    """Yield Type-1 register writes from the word stream.

    prjxray (and therefore the openXC7 flow) packs Series-7 Type-1 headers as:
        bits [31:29] = header type (001 for Type-1)
        bits [28:27] = opcode
        bits [26:13] = register address (enum value)
        bits [10:0]  = word count
    Type-2 header:
        bits [31:29] = 010
        bits [28:27] = opcode
        bits [26:0]  = word count
    Type-2 is only used for FDRI data writes and is ignored here.
    """
    i = 0
    while i < len(words):
        w = words[i]
        pkt_type = (w >> 29) & 0x7
        opcode = (w >> 27) & 0x3
        if pkt_type == 1:
            reg = (w >> 13) & 0x3fff
            count = w & 0x07ff
            i += 1
            payload = words[i:i + count]
            i += count
            yield (pkt_type, opcode, reg, payload)
        elif pkt_type == 2:
            # type-2 data packet; skip its declared word count
            count = w & 0x07ffffff
            i += 1
            i += count
            yield (pkt_type, opcode, None, words[i - count:i])
        else:
            # NOP / reserved header
            i += 1
            yield (pkt_type, opcode, None, [])


def field(value, hi, lo):
    return (value >> lo) & ((1 << (hi - lo + 1)) - 1)


def decode_cor0(v):
    lines = []
    lines.append(f"  raw                 : 0x{v:08X}")
    # prjxray/xc7series/configuration_options_0_value.h field layout
    lines.append(f"  release_gwe_cycle [2:0]   : {field(v, 2, 0)} ({field(v, 2, 0) + 1})")
    lines.append(f"  release_gts_cycle [5:3]   : {field(v, 5, 3)} ({field(v, 5, 3) + 1})")
    lines.append(f"  stall_mmcm        [8:6]   : {field(v, 8, 6)} {'NoWait' if field(v, 8, 6) == 7 else 'wait'}")
    lines.append(f"  stall_dci         [11:9]  : {field(v, 11, 9)} {'NoWait' if field(v, 11, 9) == 7 else 'wait'}")
    lines.append(f"  release_done_cycle[14:12] : {field(v, 14, 12)} ({field(v, 14, 12) + 1})")
    startup = field(v, 16, 15)
    startup_names = {0: "CCLK", 1: "UserClk", 2: "JTAGClk"}
    lines.append(f"  startup_clk       [16:15] : {startup} ({startup_names.get(startup, 'reserved')})")
    cclk_mhz = field(v, 22, 17)
    lines.append(f"  oscfsel           [22:17] : {cclk_mhz}")
    lines.append(f"  readback_single   [23]    : {field(v, 23, 23)}")
    lines.append(f"  drive_done_high   [24]    : {field(v, 24, 24)}")
    lines.append(f"  done_pipeline     [25]    : {field(v, 25, 25)}")
    return "\n".join(lines)


def decode_cor1(v):
    lines = []
    lines.append(f"  raw                 : 0x{v:08X}")
    spi_width = field(v, 8, 7)
    width_names = {0: "x1", 1: "x2", 2: "x4", 3: "reserved"}
    lines.append(f"  spi_buswidth      [8:7]   : {spi_width} ({width_names.get(spi_width, 'reserved')})")
    lines.append(f"  bpi_page_size     [3:0]   : {field(v, 3, 0)}")
    return "\n".join(lines)


def decode_idcode(v):
    return f"  raw                 : 0x{v:08X}"


def decode_wbstar(v):
    return f"  raw                 : 0x{v:08X}  start_address={v & 0x7fffff}"


def decode_timer(v):
    return f"  raw                 : 0x{v:08X}"


def decode_ctl0(v):
    lines = []
    lines.append(f"  raw                 : 0x{v:08X}")
    # UG470 Table 5-27 / Table 5-28
    gts_usr_b = field(v, 0, 0)
    lines.append(f"  gts_usr_b         [0]     : {gts_usr_b} ({'I/Os active' if gts_usr_b else 'I/Os 3-stated'})")
    sbits = field(v, 5, 4)
    sbits_names = {0: "Read/Write OK", 1: "Readback disabled", 2: "Both disabled", 3: "Both disabled"}
    lines.append(f"  sbits             [5:4]   : {sbits} ({sbits_names.get(sbits, 'reserved')})")
    dec = field(v, 6, 6)
    lines.append(f"  aes_decryptor     [6]     : {dec} ({'enabled' if dec else 'disabled'})")
    farsrc = field(v, 7, 7)
    lines.append(f"  farsrc            [7]     : {farsrc} ({'FAR' if farsrc else 'EFAR'})")
    glutmask = field(v, 8, 8)
    lines.append(f"  glutmask_b        [8]     : {glutmask} ({'do not mask' if glutmask else 'mask'})")
    fallback = field(v, 10, 10)
    lines.append(f"  config_fallback   [10]    : {fallback} ({'disabled' if fallback else 'enabled'})")
    overtemp = field(v, 12, 12)
    lines.append(f"  overtemp_pwrdwn   [12]    : {overtemp} ({'enabled' if overtemp else 'disabled'})")
    icap = field(v, 30, 30)
    lines.append(f"  icap_select       [30]    : {icap} ({'bottom' if icap else 'top'})")
    efuse = field(v, 31, 31)
    lines.append(f"  efuse_key         [31]    : {efuse} ({'eFUSE' if efuse else 'BBRAM'})")
    return "\n".join(lines)


def decode_ctl1(v):
    lines = []
    lines.append(f"  raw                 : 0x{v:08X}")
    # UG470: CTL1 is mostly reserved; bit 0 is ICAP_ENCRYPTION?
    lines.append(f"  reserved fields present; consult UG470 for design-specific bits")
    return "\n".join(lines)


def decode_bspi(v):
    lines = []
    lines.append(f"  raw                 : 0x{v:08X}")
    lines.append(f"  read_cmd          [7:0]   : 0x{field(v, 7, 0):02X}")
    lines.append(f"  dummy_clk_cycles  [11:8]    : {field(v, 11, 8)}")
    # BSPI bus-width field interpretation differs from COR1; keep it visible
    lines.append(f"  extended_addr     [16]      : {field(v, 16, 16)}")
    return "\n".join(lines)


DECODERS = {
    0x09: ("COR0", decode_cor0),
    0x0e: ("COR1", decode_cor1),
    0x0c: ("IDCODE", decode_idcode),
    0x10: ("WBSTAR", decode_wbstar),
    0x11: ("TIMER", decode_timer),
    0x05: ("CTL0", decode_ctl0),
    0x18: ("CTL1", decode_ctl1),
    0x1f: ("BSPI", decode_bspi),
}


def run_assertions(registers, crc_writes, args):
    """Exit non-zero if any requested assertion fails."""
    failed = False

    if args.assert_idcode is not None:
        idcode = registers.get(0x0c)
        if idcode != args.assert_idcode:
            print(
                f"ASSERTION FAILED: IDCODE=0x{idcode:08X}, expected 0x{args.assert_idcode:08X}",
                file=sys.stderr,
            )
            failed = True
        else:
            print(f"ASSERTION OK: IDCODE=0x{idcode:08X}")

    if args.assert_spi_x1:
        cor1 = registers.get(0x0e)
        if cor1 is None:
            print("ASSERTION FAILED: COR1 not present in bitstream", file=sys.stderr)
            failed = True
        else:
            spi_width = field(cor1, 8, 7)
            if spi_width != 0:
                print(
                    f"ASSERTION FAILED: SPI_BUSWIDTH={spi_width}, expected 0 (x1)",
                    file=sys.stderr,
                )
                failed = True
            else:
                print("ASSERTION OK: SPI_BUSWIDTH=x1")

    if args.assert_cclk_startup:
        cor0 = registers.get(0x09)
        if cor0 is None:
            print("ASSERTION FAILED: COR0 not present in bitstream", file=sys.stderr)
            failed = True
        else:
            startup = field(cor0, 16, 15)
            if startup != 0:
                print(
                    f"ASSERTION FAILED: STARTUPCLK={startup}, expected 0 (CCLK)",
                    file=sys.stderr,
                )
                failed = True
            else:
                print("ASSERTION OK: STARTUPCLK=CCLK")

    if args.assert_oscfsel is not None:
        cor0 = registers.get(0x09)
        if cor0 is None:
            print("ASSERTION FAILED: COR0 not present in bitstream", file=sys.stderr)
            failed = True
        else:
            oscfsel = field(cor0, 22, 17)
            if oscfsel != args.assert_oscfsel:
                print(
                    f"ASSERTION FAILED: OSCFSEL={oscfsel}, expected {args.assert_oscfsel}",
                    file=sys.stderr,
                )
                failed = True
            else:
                print(f"ASSERTION OK: OSCFSEL={oscfsel}")

    if args.assert_no_crc_writes:
        if crc_writes > 0:
            print(
                f"ASSERTION FAILED: {crc_writes} CRC register (0x00) write(s) present",
                file=sys.stderr,
            )
            failed = True
        else:
            print("ASSERTION OK: no CRC register writes")

    return not failed


def main():
    parser = argparse.ArgumentParser(
        description="Parse a Xilinx 7-series .bit file and dump config registers.",
    )
    parser.add_argument("bit", help="Path to the Xilinx .bit file")
    parser.add_argument(
        "--assert-idcode",
        type=lambda s: int(s, 0),
        default=None,
        help="Fail if IDCODE does not match this value (e.g. 0x03636093).",
    )
    parser.add_argument(
        "--assert-spi-x1",
        action="store_true",
        help="Fail if COR1 SPI_BUSWIDTH is not x1.",
    )
    parser.add_argument(
        "--assert-cclk-startup",
        action="store_true",
        help="Fail if COR0 STARTUPCLK is not CCLK.",
    )
    parser.add_argument(
        "--assert-oscfsel",
        type=lambda s: int(s, 0),
        default=None,
        help="Fail if COR0 OSCFSEL[22:17] does not match this value (0..63).",
    )
    parser.add_argument(
        "--assert-no-crc-writes",
        action="store_true",
        help="Fail if the bitstream contains CRC register (0x00) writes.",
    )
    args = parser.parse_args()

    with open(args.bit, "rb") as f:
        data = f.read()

    print(f"File: {args.bit} ({len(data)} bytes)")
    sync_idx = data.find(SYNC)
    print(f"Sync word 0xAA995566 at byte offset: {sync_idx} (0x{sync_idx:x})")

    words = extract_bitstream_words(data)
    print(f"Configuration words after sync: {len(words)}")
    print()

    # Collect the final write to each interesting register and all CRC writes.
    last_writes = {}
    crc_writes = 0
    for pkt_type, opcode, reg, payload in parse_packets(words):
        if pkt_type == 1 and opcode == 2 and reg is not None and payload:
            if reg in INTERESTING:
                last_writes[reg] = payload[-1]
            if reg == 0x00:
                crc_writes += len(payload)

    if not last_writes:
        print("No configuration-register writes found.")
        sys.exit(1)

    # Diagnostics: warn about common boot-from-flash gotchas.
    cor0 = last_writes.get(0x09)
    if cor0 is not None:
        oscfsel = field(cor0, 22, 17)
        if oscfsel == 0:
            print("WARNING: OSCFSEL[22:17] = 0 (default/internal CCLK). "
                  "A non-default value may be needed for reliable SPI flash wake-up.")
    if crc_writes > 0:
        print(f"WARNING: {crc_writes} CRC register (0x00) write(s) present. "
              "Manual COR0 patching without CRC recomputation may cause CRC_ERROR.")

    for reg in sorted(last_writes):
        name, decoder = DECODERS[reg]
        print(f"== {name} register (addr 0x{reg:02X}) ==")
        print(decoder(last_writes[reg]))
        print()

    if not run_assertions(last_writes, crc_writes, args):
        sys.exit(2)


if __name__ == "__main__":
    main()
