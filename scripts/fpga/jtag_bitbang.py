#!/usr/bin/env python3
"""JTAG bitbang programmer for Xilinx Platform Cable / FTDI-based adapters.
Supports XC7A100T (Artix-7) bitstream loading via JTAG SVF/XSVF or direct bitbang.

Usage:
    python3 jtag_bitbang.py --cable xpc --bitstream design.bit
    python3 jtag_bitbang.py --cable ftdi --bitstream design.bit
"""
import argparse
import struct
import sys
import time

try:
    import usb.core
except ImportError:
    usb.core = None

XILINX_VID = 0x03FD
XPC_PID_BOOT = 0x0013
XPC_PID_READY = 0x0008

JTAG_IRLEN = 6
XC7A100T_IDCODE = 0x0362D093

JTAG_CMD_WRITE_GPIO = 0x0030
JTAG_CMD_READ_GPIO = 0x0038
JTAG_CMD_INIT = 0x0028
JTAG_CMD_SELECT_GPIO = 0x0052

XPC_PROG = 0x01
XPC_TCK = 0x02
XPC_TMS = 0x04
XPC_TDI = 0x08
XPC_TDO = 0x10


class XilinxPlatformCable:
    def __init__(self):
        self.dev = None

    def open(self):
        if usb.core is None:
            raise RuntimeError("pyusb not installed: pip install pyusb")
        dev = usb.core.find(idVendor=XILINX_VID, idProduct=XPC_PID_READY)
        if dev is None:
            dev = usb.core.find(idVendor=XILINX_VID, idProduct=XPC_PID_BOOT)
            if dev:
                raise RuntimeError(
                    "Cable in bootloader mode (PID 0x0013). "
                    "Load firmware first: fxload -t fx2 -d 03fd:0013 -I xusb_xp2.hex"
                )
            raise RuntimeError("Xilinx Platform Cable not found")
        self.dev = dev
        for cfg in dev:
            for iface in cfg:
                try:
                    if dev.is_kernel_driver_active(iface.bInterfaceNumber):
                        dev.detach_kernel_driver(iface.bInterfaceNumber)
                except Exception:
                    pass
        dev.set_configuration()
        self._init_cable()

    def _vendor_out(self, wValue, wIndex=0):
        self.dev.ctrl_transfer(0x40, 0xB0, wValue, wIndex, b"", timeout=5000)

    def _vendor_in(self, wValue, wIndex=0, length=1):
        return self.dev.ctrl_transfer(0xC0, 0xB0, wValue, wIndex, length, timeout=5000)

    def _init_cable(self):
        self._vendor_out(0x0028, 0x11)
        self._vendor_out(0x0028, 0x12)
        self._write_gpio(XPC_PROG)

    def _write_gpio(self, bits):
        self._vendor_out(JTAG_CMD_WRITE_GPIO, bits)

    def _read_gpio(self):
        data = self._vendor_in(JTAG_CMD_READ_GPIO)
        return data[0] if data else 0

    def jtag_clock(self, tms, tdi):
        self._write_gpio(XPC_PROG | XPC_TCK | (tms * XPC_TMS) | (tdi * XPC_TDI))
        self._write_gpio(XPC_PROG | (tms * XPC_TMS) | (tdi * XPC_TDI))
        tdo = (self._read_gpio() & XPC_TDO) == XPC_TDO
        return tdo

    def jtag_reset(self):
        for _ in range(5):
            self.jtag_clock(1, 1)

    def jtag_goto_idle(self):
        for _ in range(1):
            self.jtag_clock(0, 1)

    def jtag_shift_ir(self, instruction):
        self.jtag_clock(1, 1)
        self.jtag_clock(0, 1)
        self.jtag_clock(0, 1)
        for i in range(JTAG_IRLEN - 1):
            self.jtag_clock(0, (instruction >> i) & 1)
        self.jtag_clock(1, (instruction >> (JTAG_IRLEN - 1)) & 1)
        self.jtag_clock(1, 1)

    def jtag_shift_dr(self, data, num_bits):
        result = 0
        self.jtag_clock(1, 1)
        self.jtag_clock(0, 1)
        self.jtag_clock(0, 1)
        for i in range(num_bits - 1):
            tdo = self.jtag_clock(0, (data >> i) & 1)
            result |= tdo << i
        tdo = self.jtag_clock(1, (data >> (num_bits - 1)) & 1)
        result |= tdo << (num_bits - 1)
        self.jtag_clock(1, 1)
        return result

    def read_idcode(self):
        self.jtag_reset()
        self.jtag_goto_idle()
        self.jtag_shift_ir(0x09)
        idcode = self.jtag_shift_dr(0, 32)
        return idcode


def parse_bitstream(filepath):
    with open(filepath, "rb") as f:
        magic = f.read(2)
        if magic != b"\x00\x09":
            raise ValueError(f"Not a Xilinx bitstream: {filepath}")
        fields = []
        for _ in range(4):
            header = struct.unpack(">H", f.read(2))[0]
            if header == ord("e"):
                key = chr(f.read(1)[0])
                length = struct.unpack(">I", f.read(4))[0]
                fields.append((key, f.read(length)))
            elif header == ord("a"):
                key = "a"
                length = struct.unpack(">H", f.read(2))[0]
                fields.append((key, f.read(length)))
            elif header == ord("c"):
                key = "c"
                length = struct.unpack(">H", f.read(2))[0]
                fields.append((key, f.read(length)))
            elif header == ord("b"):
                key = "b"
                length = struct.unpack(">I", f.read(4))[0]
                fields.append((key, f.read(length)))
            else:
                raise ValueError(f"Unknown bitstream field: 0x{header:04X}")
        for key, data in fields:
            if key == "b":
                return data
    raise ValueError("No bitstream data found")


def jtag_program_xc7(cable, bitstream_data):
    print(f"Bitstream: {len(bitstream_data)} bytes")

    print("Resetting JTAG...")
    cable.jtag_reset()

    print("Reading IDCODE...")
    idcode = cable.read_idcode()
    print(f"IDCODE: 0x{idcode:08X}")
    if idcode != XC7A100T_IDCODE:
        print(f"WARNING: Expected 0x{XC7A100T_IDCODE:08X} (XC7A100T)")

    print("Loading bitstream via JTAG...")
    cable.jtag_shift_ir(0x3B)
    cable.jtag_shift_dr(0, 32)
    cable.jtag_shift_ir(0x05)

    print("Shifting DR with bitstream data...")
    cable.jtag_clock(1, 1)
    cable.jtag_clock(0, 1)
    cable.jtag_clock(0, 1)
    total_bits = len(bitstream_data) * 8
    for byte_idx, byte_val in enumerate(bitstream_data):
        for bit_idx in range(8):
            tdi = (byte_val >> (7 - bit_idx)) & 1
            is_last = (byte_idx == len(bitstream_data) - 1) and (bit_idx == 7)
            if is_last:
                cable.jtag_clock(1, tdi)
            else:
                cable.jtag_clock(0, tdi)
        if byte_idx % 100000 == 0 and byte_idx > 0:
            pct = byte_idx * 100 // len(bitstream_data)
            print(f"  {pct}% ({byte_idx}/{len(bitstream_data)} bytes)")

    cable.jtag_clock(1, 1)
    cable.jtag_clock(1, 1)

    print("Starting FPGA...")
    cable.jtag_shift_ir(0x0F)
    cable.jtag_shift_dr(0, 32)

    print("Checking DONE...")
    cable.jtag_shift_ir(0x3C)
    status = cable.jtag_shift_dr(0, 32)
    done = (status >> 3) & 1
    print(f"STATUS: 0x{status:08X}, DONE={done}")
    if done:
        print("SUCCESS! FPGA configured.")
    else:
        print("WARNING: DONE pin not asserted. Check bitstream and pins.")
    return done


def main():
    parser = argparse.ArgumentParser(description="JTAG bitbang programmer")
    parser.add_argument("--cable", choices=["xpc", "ftdi"], default="xpc")
    parser.add_argument("--bitstream", required=True)
    parser.add_argument("--idcode-only", action="store_true")
    args = parser.parse_args()

    cable = XilinxPlatformCable()
    print("Opening cable...")
    cable.open()
    print("Cable opened.")

    if args.idcode_only:
        cable.jtag_reset()
        idcode = cable.read_idcode()
        print(f"IDCODE: 0x{idcode:08X}")
        return

    data = parse_bitstream(args.bitstream)
    jtag_program_xc7(cable, data)


if __name__ == "__main__":
    main()
