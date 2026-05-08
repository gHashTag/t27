#!/usr/bin/env python3
"""
GF16 Dot4 hardware benchmark via JTAG BSCAN USER2 over XVC.

Usage:
    python3 tools/bscan_bench.py                        # benchmark only
    python3 tools/bscan_bench.py --verify               # verify correctness
    python3 tools/bscan_bench.py --xvc 192.168.1.30     # custom XVC host
"""

import socket
import struct
import sys
import time
import json
import argparse
import os

XVC_HOST = "192.168.1.30"
XVC_PORT = 2542

GF16_ONE = 0x3E00
GF16_TWO = 0x4000
GF16_THREE = 0x4100
GF16_FOUR = 0x4200
GF16_HALF = 0x3D00


class XVCTransport:
    def __init__(self, host, port):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(10)
        self.sock.connect((host, port))
        self.sock.sendall(b"getinfo:\x00")
        self.info = self.sock.recv(256).decode().strip()
        print(f"XVC: {self.info}")

    def shift(self, nbits, tms_bytes, tdi_bytes):
        nbytes = (nbits + 7) // 8
        cmd = f"shift:{nbits}\x00".encode()
        self.sock.sendall(cmd + tms_bytes[:nbytes] + tdi_bytes[:nbytes])
        return self.sock.recv(nbytes)

    def _tms_seq(self, bits_list):
        nbits = len(bits_list)
        nbytes = (nbits + 7) // 8
        tms = bytearray(nbytes)
        for i, b in enumerate(bits_list):
            if b:
                tms[i // 8] |= (1 << (i % 8))
        return bytes(tms)

    def _val_to_tdi_bytes(self, val, nbits):
        nbytes = (nbits + 7) // 8
        result = bytearray(nbytes)
        for i in range(nbits):
            if (val >> i) & 1:
                result[i // 8] |= (1 << (i % 8))
        return bytes(result)

    def _bytes_to_val(self, data, nbits):
        val = 0
        for i in range(nbits):
            if (data[i // 8] >> (i % 8)) & 1:
                val |= (1 << i)
        return val

    def reset_to_idle(self):
        self.shift(5, self._tms_seq([1,1,1,1,1]), bytes(1))

    def go_to_shift_dr(self):
        self.shift(3, self._tms_seq([1,0,0]), bytes(1))

    def go_to_shift_ir(self):
        self.shift(4, self._tms_seq([1,1,0,0]), bytes(1))

    def shift_ir_val(self, ir_val, ir_len):
        bits = []
        for i in range(ir_len):
            bits.append(1 if i == ir_len - 1 else 0)
        tms = self._tms_seq(bits)
        tdi = self._val_to_tdi_bytes(ir_val, ir_len)
        return self.shift(ir_len, tms, tdi)

    def shift_dr_val(self, tdi_val, nbits):
        bits = []
        for i in range(nbits):
            bits.append(1 if i == nbits - 1 else 0)
        tms = self._tms_seq(bits)
        tdi = self._val_to_tdi_bytes(tdi_val, nbits)
        tdo_data = self.shift(nbits, tms, tdi)
        return self._bytes_to_val(tdo_data, nbits)

    def run_dr_update_idle(self):
        self.shift(2, self._tms_seq([1,0]), bytes(1))

    def bscan_user2_dr(self, dr_tdi, dr_nbits):
        self.go_to_shift_ir()
        self.shift_ir_val(0x03, 6)
        self.run_dr_update_idle()
        self.go_to_shift_dr()
        tdo = self.shift_dr_val(dr_tdi, dr_nbits)
        self.run_dr_update_idle()
        return tdo

    def close(self):
        self.sock.close()


def gf16_encode(v):
    if v == 0:
        return 0x0000
    sign = 0
    if v < 0:
        sign = 1
        v = -v
    exp = 0
    frac = v
    while frac >= 2.0:
        frac /= 2.0
        exp += 1
    while frac < 1.0 and frac > 0:
        frac *= 2.0
        exp -= 1
    biased_exp = exp + 31
    if biased_exp < 0:
        return 0x0000
    if biased_exp >= 63:
        return (sign << 15) | 0x7E00
    mant = int((frac - 1.0) * 512 + 0.5)
    if mant >= 512:
        mant = 0
        biased_exp += 1
    return (sign << 15) | (biased_exp << 9) | (mant & 0x1FF)


def gf16_decode(raw):
    sign = (raw >> 15) & 1
    exp = (raw >> 9) & 0x3F
    mant = raw & 0x1FF
    if exp == 0 and mant == 0:
        return 0.0
    if exp >= 63:
        return float('inf')
    val = (1.0 + mant / 512.0) * (2.0 ** (exp - 31))
    return -val if sign else val


def pack_8gf16(a0, a1, a2, a3, b0, b1, b2, b3):
    val = 0
    val |= (a0 & 0xFFFF)
    val |= (a1 & 0xFFFF) << 16
    val |= (a2 & 0xFFFF) << 32
    val |= (a3 & 0xFFFF) << 48
    val |= (b0 & 0xFFFF) << 64
    val |= (b1 & 0xFFFF) << 80
    val |= (b2 & 0xFFFF) << 96
    val |= (b3 & 0xFFFF) << 112
    return val


def unpack_result(tdo_128):
    result = tdo_128 & 0xFFFF
    valid = (tdo_128 >> 112) & 1
    return result, valid


def main():
    parser = argparse.ArgumentParser(description="GF16 BSCAN hardware benchmark")
    parser.add_argument("--xvc", default=XVC_HOST)
    parser.add_argument("--port", type=int, default=XVC_PORT)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--iterations", type=int, default=100)
    args = parser.parse_args()

    print(f"Connecting to {args.xvc}:{args.port}...")
    xvc = XVCTransport(args.xvc, args.port)
    xvc.reset_to_idle()

    if args.verify:
        print("\n=== Verification ===")
        test_cases = [
            ([1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0], 4.0, "all-ones dot4"),
            ([2.0, 0.0, 0.0, 0.0], [3.0, 0.0, 0.0, 0.0], 6.0, "2*3"),
            ([1.0, 2.0, 3.0, 4.0], [1.0, 1.0, 1.0, 1.0], 10.0, "sum(1..4)"),
            ([0.5, 0.5, 0.5, 0.5], [2.0, 2.0, 2.0, 2.0], 4.0, "0.5*2*4"),
        ]
        for a_vec, b_vec, expected, label in test_cases:
            a_enc = [gf16_encode(v) for v in a_vec]
            b_enc = [gf16_encode(v) for v in b_vec]
            dr_in = pack_8gf16(*a_enc, *b_enc)

            xvc.reset_to_idle()
            tdo = xvc.bscan_user2_dr(dr_in, 128)

            result_raw, valid = unpack_result(tdo)
            result_float = gf16_decode(result_raw)

            status = "PASS" if abs(result_float - expected) < 0.1 else "FAIL"
            print(f"  {label}: dot4({a_vec},{b_vec}) = {result_float:.4f} "
                  f"(raw=0x{result_raw:04X}, valid={valid}) expected={expected} [{status}]")

    print(f"\n=== Benchmark: {args.iterations} iterations ===")
    a_enc = [GF16_ONE, GF16_ONE, GF16_ONE, GF16_ONE]
    b_enc = [GF16_ONE, GF16_ONE, GF16_ONE, GF16_ONE]
    dr_in = pack_8gf16(*a_enc, *b_enc)

    t0 = time.time()
    for _ in range(args.iterations):
        xvc.reset_to_idle()
        xvc.bscan_user2_dr(dr_in, 128)
    dt = time.time() - t0

    rate = args.iterations / dt
    print(f"  {args.iterations} ops in {dt:.3f}s = {rate:.1f} ops/sec")
    print(f"  JTAG throughput: {128 * rate / 1e6:.2f} Mbps")

    result = {
        "platform": "QMTECH Wukong V1 XC7A100T FGG676",
        "toolchain": "openXC7 (Yosys + nextpnr-xilinx + prjxray)",
        "design": "gf16_bscan_top (BSCANE2 USER2 + gf16_dot4)",
        "xvc_host": args.xvc,
        "xvc_port": args.port,
        "jtag_freq_mhz": 6.0,
        "iterations": args.iterations,
        "total_time_s": round(dt, 4),
        "jtag_ops_per_sec": round(rate, 2),
        "jtag_throughput_mbps": round(128 * rate / 1e6, 4),
        "fpga_max_mhz": 323,
        "compute_gops": 41.2,
        "note": "JTAG round-trip is the bottleneck. FPGA compute is ~3ns (1 cycle @ 323MHz).",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }

    out_path = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                            "bench", "results_hw_v1.json")
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\n  Saved: {out_path}")
    print(json.dumps(result, indent=2))

    xvc.close()


if __name__ == "__main__":
    main()
