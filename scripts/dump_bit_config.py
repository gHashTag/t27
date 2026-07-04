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

Field decoding follows UG470 "7 Series FPGAs Configuration" and the prjxray
Series7ConfigurationRegister enum.
"""

import sys
import struct

SYNC = b'\xaa\x99\x55\x66'

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
    lines.append(f"  cclk_freq_mhz     [22:17] : {cclk_mhz}")
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
    return f"  raw                 : 0x{v:08X}"


def decode_ctl1(v):
    return f"  raw                 : 0x{v:08X}"


def decode_bspi(v):
    lines = []
    lines.append(f"  raw                 : 0x{v:08X}")
    lines.append(f"  read_cmd          [7:0]   : 0x{field(v, 7, 0):02X}")
    lines.append(f"  dummy_clk_cycles  [11:8]   : {field(v, 11, 8)}")
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


def main():
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <bitstream.bit>", file=sys.stderr)
        sys.exit(1)

    path = sys.argv[1]
    with open(path, "rb") as f:
        data = f.read()

    print(f"File: {path} ({len(data)} bytes)")
    sync_idx = data.find(SYNC)
    print(f"Sync word 0xAA995566 at byte offset: {sync_idx} (0x{sync_idx:x})")

    words = extract_bitstream_words(data)
    print(f"Configuration words after sync: {len(words)}")
    print()

    # Collect the final write to each interesting register.
    last_writes = {}
    for pkt_type, opcode, reg, payload in parse_packets(words):
        if pkt_type == 1 and opcode == 2 and reg is not None and reg in DECODERS and payload:
            last_writes[reg] = payload[-1]

    if not last_writes:
        print("No configuration-register writes found.")
        sys.exit(1)

    for reg in sorted(last_writes):
        name, decoder = DECODERS[reg]
        print(f"== {name} register (addr 0x{reg:02X}) ==")
        print(decoder(last_writes[reg]))
        print()


if __name__ == "__main__":
    main()
