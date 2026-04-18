#!/usr/bin/env python3
"""UART loopback smoke test for QMTECH XC7A100T minimal profile.

Verifies that the FPGA board echoes back sent data via UART loopback.
The minimal design connects uart_tx = uart_rx (hardware loopback).

Usage:
    python3 tools/uart_smoke.py --port /dev/ttyUSB0 --baud 115200
    python3 tools/uart_smoke.py --port /dev/cu.usbserial-140
"""

import argparse
import sys
import time

try:
    import serial
except ImportError:
    print("pyserial not installed. Install with: pip install pyserial")
    sys.exit(1)

TEST_MESSAGES = [
    b"PING\n",
    b"HELLO\n",
    b"\x00\xFF\x55\xAA",
    b"TRINITY",
]

TIMEOUT_SECONDS = 2.0


def test_loopback(port: str, baud: int) -> bool:
    try:
        ser = serial.Serial(port, baudrate=baud, timeout=TIMEOUT_SECONDS)
    except serial.SerialException as e:
        print(f"FAIL: Cannot open {port}: {e}")
        return False

    print(f"UART smoke test: {port} @ {baud} baud")
    print(f"{'Test':<25} {'Sent':>8} {'Recv':>8} {'Status'}")
    print("-" * 60)

    all_pass = True
    for msg in TEST_MESSAGES:
        ser.reset_input_buffer()
        ser.write(msg)
        time.sleep(0.05)
        received = ser.read(len(msg))

        label = repr(msg[:16])
        if len(msg) > 16:
            label = label[:-1] + "...)"
        passed = received == msg
        status = "PASS" if passed else "FAIL"
        if not passed:
            all_pass = False

        print(f"{label:<25} {len(msg):>8} {len(received):>8} {status}")
        if not passed:
            expected = repr(msg[:32])
            got = repr(received[:32])
            print(f"  Expected: {expected}")
            print(f"  Got:      {got}")

    ser.close()
    return all_pass


def main():
    parser = argparse.ArgumentParser(description="UART loopback smoke test")
    parser.add_argument("--port", required=True, help="Serial port (e.g., /dev/ttyUSB0)")
    parser.add_argument("--baud", type=int, default=115200, help="Baud rate (default: 115200)")
    args = parser.parse_args()

    if test_loopback(args.port, args.baud):
        print("\nAll tests PASSED")
        sys.exit(0)
    else:
        print("\nSome tests FAILED")
        sys.exit(1)


if __name__ == "__main__":
    main()
