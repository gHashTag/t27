#!/usr/bin/env python3
"""Native macOS DLC10 (Xilinx Platform Cable USB II) JTAG driver.

Reverse-engineered from xc3sprog/ioxpc.cpp. Uses pyusb for direct USB access.
Bit-reverses bitstream data (xc3sprog convention: Xilinx MSB-first -> JTAG LSB-first).
Parses .bit format to extract raw bitstream data (skips header metadata).

Usage:
    python3 dlc10_jtag.py <file.bit>

Tested: QMTECH XC7A100T Wukong V1, macOS ARM64, Python 3.14, pyusb.
"""

import sys, struct, time, usb.core, usb.util

FW_PATH = "/Users/playom/trinity-fpga/fpga/tools/xusb_xp2.hex"

VID_XILINX = 0x03FD
PID_UNINIT = 0x0013
PID_READY = 0x0008

BIT_REV_TABLE = bytes(int(f"{b:08b}"[::-1], 2) for b in range(256))

XC7_IR = {
    "BYPASS": 0x3F, "IDCODE": 0x09, "CFG_IN": 0x05, "CFG_OUT": 0x04,
    "JPROGRAM": 0x0B, "JSTART": 0x0C, "JSHUTDOWN": 0x0D,
    "ISC_ENABLE": 0x10, "ISC_DISABLE": 0x16,
}


def bitrev(data):
    return bytes(BIT_REV_TABLE[b] for b in data)


def parse_bitfile(path):
    """Parse Xilinx .bit format, return bit-reversed bitstream data."""
    with open(path, "rb") as f:
        data = f.read()
    for i in range(min(512, len(data) - 5)):
        if data[i] == 0x65:
            bs_len = struct.unpack(">I", data[i + 1 : i + 5])[0]
            if abs(bs_len - (len(data) - i - 5)) < 256:
                return bitrev(data[i + 5 : i + 5 + bs_len])
    raise ValueError("No 'e' field found in .bit file")


class DLC10:
    CHUNK_BITS = 16379  # NOT multiple of 4 to avoid padding corruption

    def __init__(self):
        self.dev = None
        self.intf = None

    def open(self):
        dev = usb.core.find(idVendor=VID_XILINX, idProduct=PID_UNINIT)
        if dev:
            self._load_firmware(dev)
            dev = None
            for _ in range(20):
                time.sleep(1)
                dev = usb.core.find(idVendor=VID_XILINX, idProduct=PID_READY)
                if dev:
                    break
            assert dev, "PID 0x0008 not found after firmware load"
        else:
            dev = usb.core.find(idVendor=VID_XILINX, idProduct=PID_READY)
            assert dev, "DLC10 not found (needs USB replug)"

        time.sleep(2)
        dev.set_configuration()
        cfg = dev.get_active_configuration()
        self.intf = cfg[(0, 0)]
        try:
            if dev.is_kernel_driver_active(0):
                dev.detach_kernel_driver(0)
        except Exception:
            pass
        usb.util.claim_interface(dev, self.intf)

        dev.ctrl_transfer(0xC0, 0xB0, 0x0050, 0, 2, 10000)
        dev.ctrl_transfer(0xC0, 0xB0, 0x0050, 1, 2, 10000)
        dev.ctrl_transfer(0x40, 0xB0, 0x0028, 0x11, b"", 10000)
        dev.ctrl_transfer(0x40, 0xB0, 0x0030, 1 << 3, b"", 10000)
        dev.ctrl_transfer(0x40, 0xB0, 0x0028, 0x11, b"", 10000)
        dev.ctrl_transfer(0x40, 0xB0, 0x0018, 0, b"", 10000)
        dev.ctrl_transfer(0x40, 0xB0, 0xA6, 2, b"", 10000)
        dev.write(0x02, b"\x00\x00", timeout=10000)
        dev.ctrl_transfer(0x40, 0xB0, 0x0028, 0x12, b"", 10000)
        self.dev = dev

    def close(self):
        if self.dev:
            try:
                self.dev.ctrl_transfer(0x40, 0xB0, 0x0010, 0, b"", 5000)
            except Exception:
                pass
            try:
                usb.util.release_interface(self.dev, self.intf)
            except Exception:
                pass
            usb.util.dispose_resources(self.dev)
            self.dev = None

    def _load_firmware(self, dev):
        dev.set_configuration()
        with open(FW_PATH) as f:
            for line in f:
                line = line.strip()
                if not line or line[0] != ":":
                    continue
                b = bytes.fromhex(line[1:])
                addr = (b[1] << 8) | b[2]
                rlen, typ = b[0], b[3]
                if typ == 0 and rlen > 0:
                    dev.ctrl_transfer(0x40, 0xA0, addr, 0, b[4 : 4 + rlen], 5000)
            dev.ctrl_transfer(0x40, 0xA0, 0xE600, 0, b"\x00", 5000)
        time.sleep(5)
        try:
            usb.util.dispose_resources(dev)
        except Exception:
            pass

    def _do_shift(self, tdi, tms):
        n = len(tdi)
        if n % 4 == 0:
            tdi.append(False)
            tms.append(False)
            n += 1
        nw = (n + 3) // 4
        buf = bytearray(nw * 2)
        for i in range(n):
            bi = i & 3
            wi = (i - bi) >> 1
            if bi == 0:
                buf[wi] = 0
                buf[wi + 1] = 0
            if tdi[i]:
                buf[wi] |= 0x01 << bi
            if tms[i]:
                buf[wi] |= 0x10 << bi
            buf[wi + 1] |= 0x01 << bi
        self.dev.ctrl_transfer(0x40, 0xB0, 0xA6, n, b"", 10000)
        self.dev.write(0x02, bytes(buf), timeout=30000)

    def shift_ir(self, ir_val):
        tdi, tms = [], []
        for _ in range(5):
            tdi.append(True)
            tms.append(True)
        tdi += [True, False, True, True, False, False]
        tms += [False, True, True, False, False, False]
        for i in range(6):
            tdi.append(bool(ir_val & (1 << i)))
            tms.append(i == 5)
        tdi += [True, True]
        tms += [True, False]
        self._do_shift(tdi, tms)

    def shift_dr(self, data, nb):
        sent, first = 0, True
        while sent < nb:
            chunk = min(nb - sent, self.CHUNK_BITS)
            tdi, tms = [], []
            if first:
                tdi += [True, True, True]
                tms += [True, False, False]
                first = False
            for i in range(chunk):
                bp = sent + i
                tdi.append(bool(data[bp >> 3] & (1 << (bp & 7))))
                tms.append((sent + i) == nb - 1)
            if sent + chunk == nb:
                tdi += [True, True]
                tms += [True, False]
            self._do_shift(tdi, tms)
            sent += chunk
            if sent == nb or sent % 4000000 < chunk:
                print(f"  {sent}/{nb} ({100 * sent // nb}%)")

    def shift_dr_small(self, data, nb):
        tdi, tms = [True, True, True], [True, False, False]
        for i in range(nb):
            tdi.append(bool(data[i >> 3] & (1 << (i & 7))))
            tms.append(i == nb - 1)
        tdi += [True, True]
        tms += [True, False]
        self._do_shift(tdi, tms)

    def cycle_tck(self, n):
        self._do_shift([True] * n, [False] * n)

    def read_dr_32(self):
        tdi, tms = [True, True, True], [True, False, False]
        rdo_start = len(tdi)
        for i in range(32):
            tdi.append(False)
            tms.append(i == 31)
        tdi += [True, True]
        tms += [True, False]
        n = len(tdi)
        if n % 4 == 0:
            tdi.append(False)
            tms.append(False)
            n += 1
        nw = (n + 3) // 4
        buf = bytearray(nw * 2)
        for i in range(n):
            bi = i & 3
            wi = (i - bi) >> 1
            if bi == 0:
                buf[wi] = 0
                buf[wi + 1] = 0
            if tdi[i]:
                buf[wi] |= 0x01 << bi
            if tms[i]:
                buf[wi] |= 0x10 << bi
            if rdo_start <= i < rdo_start + 32:
                buf[wi + 1] |= 0x11 << bi
            else:
                buf[wi + 1] |= 0x01 << bi
        self.dev.ctrl_transfer(0x40, 0xB0, 0xA6, n, b"", 10000)
        self.dev.write(0x02, bytes(buf), timeout=30000)
        ol = 2 * ((32 + 15) // 16)
        resp = bytes(self.dev.read(0x86, ol, timeout=10000))
        words = [
            struct.unpack_from("<H", resp, j)[0] for j in range(0, len(resp), 2)
        ]
        val = 0
        for i in range(32):
            wi, bi = i // 16, i % 16
            if words[wi] & (1 << bi):
                val |= 1 << i
        return val

    def read_idcode(self):
        """Read IDCODE via dedicated instruction. Returns 32-bit value."""
        self.shift_ir(XC7_IR["IDCODE"])
        tdi, tms = [True, True, True], [True, False, False]
        rdo_start = len(tdi)
        for i in range(32):
            tdi.append(False)
            tms.append(i == 31)
        tdi += [True, True]
        tms += [True, False]

        n = len(tdi)
        if n % 4 == 0:
            tdi.append(False)
            tms.append(False)
            n += 1
        nw = (n + 3) // 4
        buf = bytearray(nw * 2)
        for i in range(n):
            bi = i & 3
            wi = (i - bi) >> 1
            if bi == 0:
                buf[wi] = 0
                buf[wi + 1] = 0
            if tdi[i]:
                buf[wi] |= 0x01 << bi
            if tms[i]:
                buf[wi] |= 0x10 << bi
            if rdo_start <= i < rdo_start + 32:
                buf[wi + 1] |= 0x11 << bi
            else:
                buf[wi + 1] |= 0x01 << bi
        self.dev.ctrl_transfer(0x40, 0xB0, 0xA6, n, b"", 10000)
        self.dev.write(0x02, bytes(buf), timeout=30000)
        ol = 2 * ((32 + 15) // 16)
        resp = bytes(self.dev.read(0x86, ol, timeout=10000))
        words = [
            struct.unpack_from("<H", resp, j)[0] for j in range(0, len(resp), 2)
        ]
        val = 0
        for i in range(32):
            wi, bi = i // 16, i % 16
            if words[wi] & (1 << bi):
                val |= 1 << i
        return val

    def program_xc7(self, bitfile_path):
        """Program XC7 FPGA with .bit file (SRAM, legacy flow)."""
        bs = parse_bitfile(bitfile_path)
        nb = len(bs) * 8
        print(f"Bitstream: {len(bs)}B ({nb} bits)")

        self.shift_ir(XC7_IR["JSHUTDOWN"])
        self.cycle_tck(12)

        print("CFG_IN + bitstream...")
        self.shift_ir(XC7_IR["CFG_IN"])
        t0 = time.time()
        self.shift_dr(bs, nb)
        elapsed = time.time() - t0
        print(f"  {elapsed:.1f}s ({nb / elapsed / 1e6:.2f} Mbps)")
        self.cycle_tck(1)

        self.shift_ir(XC7_IR["JSTART"])
        self.cycle_tck(24)

        self.shift_ir(XC7_IR["BYPASS"])
        self.shift_dr_small(bytes([0x00]), 1)
        self.cycle_tck(1)

        self.shift_ir(XC7_IR["CFG_OUT"])
        tdo = self.read_dr_32()
        print(f"STATUS: 0x{tdo:08X}")
        done = (tdo >> 2) & 1
        print(f"DONE pin: {'HIGH (configured!)' if done else 'LOW (not configured)'}")

        print("Done.")


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <file.bit>")
        sys.exit(1)

    jtag = DLC10()
    try:
        jtag.open()
        idcode = jtag.read_idcode()
        print(f"IDCODE: 0x{idcode:08X}")
        jtag.program_xc7(sys.argv[1])
    finally:
        jtag.close()


if __name__ == "__main__":
    main()
