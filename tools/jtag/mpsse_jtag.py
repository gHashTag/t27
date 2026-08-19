#!/usr/bin/env python3
"""FTDI MPSSE JTAG transport -- the adapter T137 named.

`cli/dlc10` already owns every JTAG primitive this project needs: shift_ir,
shift_dr, shift_dr_read_bytes, all built on ONE boundary,
`do_shift(tdi: &[bool], tms: &[bool])`. It cannot open our cables because it is
hardcoded to VID 0x03FD. `openFPGALoader` opens them and exposes no user
register. The missing piece is a transport under that boundary.

This is that transport, prototyped through libftdi1 via ctypes so the question
"can it be done on this hardware" is answered before any Rust is written.

MILESTONE: read IDCODE. The answer is known independently -- 0x13636093, printed
by openFPGALoader on all three boards -- so a correct read proves the transport
and a wrong one cannot be mistaken for success. That is lesson 429: compare
against a case whose right answer you already have.
"""
import ctypes, sys, time

LIB = "/opt/homebrew/lib/libftdi1.2.dylib"
VID, PID = 0x0403, 0x6014

# MPSSE opcodes
CLK_BYTES_OUT_NEG = 0x19   # -ve clock, LSB first, write only
CLK_BITS_OUT_NEG  = 0x1B
CLK_BITS_IO_NEG   = 0x3B   # write TDI / read TDO, bits, LSB first
CLK_TMS_OUT_NEG   = 0x4B   # TMS bits out, TDI held
CLK_TMS_IO_NEG    = 0x6B   # TMS out + TDO in

f = ctypes.CDLL(LIB)
f.ftdi_new.restype = ctypes.c_void_p
f.ftdi_get_error_string.restype = ctypes.c_char_p


def chk(ctx, rc, what):
    if rc < 0:
        msg = f.ftdi_get_error_string(ctypes.c_void_p(ctx))
        raise SystemExit(f"  {what}: rc={rc} {msg.decode() if msg else ''}")


class Mpsse:
    def __init__(self, index=0):
        self.ctx = f.ftdi_new()
        if not self.ctx:
            raise SystemExit("  ftdi_new failed")
        c = ctypes.c_void_p(self.ctx)
        # INTERFACE_A = 1
        chk(self.ctx, f.ftdi_set_interface(c, 1), "set_interface")
        rc = f.ftdi_usb_open_desc_index(c, VID, PID, None, None, index)
        chk(self.ctx, rc, f"usb_open(index={index})")
        chk(self.ctx, f.ftdi_usb_reset(c), "usb_reset")
        chk(self.ctx, f.ftdi_set_bitmode(c, 0x00, 0x00), "bitmode reset")
        time.sleep(0.02)
        chk(self.ctx, f.ftdi_set_bitmode(c, 0x00, 0x02), "bitmode MPSSE")
        time.sleep(0.02)
        f.ftdi_usb_purge_buffers(c)
        # TCK = 60MHz/((1+div)*2); div=29 -> 1 MHz
        self.w(bytes([0x8A, 0x97, 0x8D]))          # /5 off, adaptive off, 3-phase off
        self.w(bytes([0x86, 29, 0]))               # divisor
        # TCK=0 TDI=0 TMS=1 as outputs (bit0 TCK, bit1 TDI, bit3 TMS)
        self.w(bytes([0x80, 0x08, 0x0B]))
        time.sleep(0.02)
        f.ftdi_usb_purge_buffers(c)

    def w(self, data: bytes):
        buf = (ctypes.c_ubyte * len(data))(*data)
        rc = f.ftdi_write_data(ctypes.c_void_p(self.ctx), buf, len(data))
        chk(self.ctx, rc, "write")

    def r(self, n: int, timeout=1.0) -> bytes:
        out = bytearray()
        buf = (ctypes.c_ubyte * n)()
        t0 = time.time()
        while len(out) < n and time.time() - t0 < timeout:
            rc = f.ftdi_read_data(ctypes.c_void_p(self.ctx), buf, n - len(out))
            if rc < 0:
                chk(self.ctx, rc, "read")
            out += bytes(buf[:rc])
            if rc == 0:
                time.sleep(0.002)
        return bytes(out)

    def tms(self, bits, tdi_level=0):
        """Clock up to 7 TMS bits; TDI is held at tdi_level."""
        assert 1 <= len(bits) <= 7
        val = 0
        for i, b in enumerate(bits):
            val |= (b & 1) << i
        val |= (tdi_level & 1) << 7
        self.w(bytes([CLK_TMS_OUT_NEG, len(bits) - 1, val]))

    def shift_dr_read(self, nbits):
        """In Shift-DR, clock nbits of 0 out and read TDO back."""
        nbytes, rem = divmod(nbits, 8)
        got = bytearray()
        if nbytes:
            self.w(bytes([0x39, (nbytes - 1) & 0xFF, ((nbytes - 1) >> 8) & 0xFF]) + bytes(nbytes))
            got += self.r(nbytes)
        if rem:
            self.w(bytes([CLK_BITS_IO_NEG, rem - 1, 0x00]))
            b = self.r(1)
            got += bytes([b[0] >> (8 - rem)]) if b else b""
        return bytes(got)

    def close(self):
        c = ctypes.c_void_p(self.ctx)
        f.ftdi_set_bitmode(c, 0x00, 0x00)     # back to reset mode
        f.ftdi_usb_close(c)
        f.ftdi_free(c)


def main():
    idx = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    m = Mpsse(idx)
    try:
        # TLR (5x TMS=1) -> Run-Test/Idle -> Select-DR -> Capture-DR -> Shift-DR
        m.tms([1, 1, 1, 1, 1])
        m.tms([0])              # Run-Test/Idle
        m.tms([1, 0, 0])        # Select-DR, Capture-DR, Shift-DR
        raw = m.shift_dr_read(32)
        m.tms([1, 1, 0])        # Exit1-DR, Update-DR, RTI
        if len(raw) < 4:
            print(f"  READ SHORT: {len(raw)} bytes -- transport did not return data")
            return 1
        idcode = int.from_bytes(raw[:4], "little")
        print(f"  raw={raw[:4].hex()}  IDCODE=0x{idcode:08X}")
        print(f"  expected 0x13636093 (XC7A200T)  -> {'MATCH' if idcode & 0x0FFFFFFF == 0x03636093 else 'MISMATCH'}")
        return 0
    finally:
        m.close()


if __name__ == "__main__":
    sys.exit(main())


def read_user1(idx=0):
    """Shift USER1 (IR=0x02, 6-bit IR on 7-series) and read the 4-bit verdict.

    bit0 ok | bit1 beat | bit2 constant 1 | bit3 constant 0

    Bits 3:2 must read `01`. An all-zero or all-one chain -- the two ways a
    readback dies silently -- cannot produce that, so a plausible `ok` cannot be
    manufactured by a chain that is not there.
    """
    m = Mpsse(idx)
    try:
        m.tms([1, 1, 1, 1, 1])       # TLR
        m.tms([0])                   # RTI
        m.tms([1, 1, 0, 0])          # Select-DR, Select-IR, Capture-IR, Shift-IR
        # 6-bit IR, USER1 = 0x02, last bit exits with TMS=1
        m.w(bytes([CLK_BITS_OUT_NEG, 4, 0x02]))      # 5 bits LSB-first
        m.w(bytes([CLK_TMS_OUT_NEG, 0, (0x00 << 7) | 0x01]))  # last IR bit + Exit1-IR
        m.tms([1, 0])                # Update-IR, RTI
        m.tms([1, 0, 0])             # Select-DR, Capture-DR, Shift-DR
        raw = m.shift_dr_read(4)
        m.tms([1, 1, 0])
        if not raw:
            print("  USER1: no data")
            return 1
        v = raw[0] & 0xF
        ok, beat, c1, c0 = v & 1, (v >> 1) & 1, (v >> 2) & 1, (v >> 3) & 1
        print(f"  USER1 = 0b{v:04b}   ok={ok} beat={beat} const1={c1} const0={c0}")
        if (c1, c0) != (1, 0):
            print("  CHAIN DEAD -- the constant pattern did not come back; ok is meaningless")
            return 1
        print("  chain alive (const pattern 01 present)")
        print(f"  VERDICT: {'PASS -- silicon classified every input correctly' if ok else 'FAIL -- a wrong class was latched'}")
        return 0
    finally:
        m.close()
