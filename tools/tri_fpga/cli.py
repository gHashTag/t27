import argparse
import subprocess
import sys
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DLC10_SCRIPT = os.path.join(SCRIPT_DIR, "..", "dlc10_jtag.py")

OPENFPGA = "openFPGALoader"
CABLE = "digilent"


def _run(cmd, check=True):
    print(f"$ {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=False)
    if check and result.returncode != 0:
        print(f"FAILED (exit {result.returncode})")
    return result.returncode


def _has_openfpga_cable():
    r = subprocess.run(
        [OPENFPGA, "--cable", CABLE, "--detect"],
        capture_output=True, timeout=10,
    )
    return r.returncode == 0


def _dlc10_program(bitstream):
    return _run(["python3", DLC10_SCRIPT, bitstream], check=False)


def _dlc10_detect():
    return _run(
        ["python3", "-c",
         "import sys; sys.path.insert(0,'.'); "
         "exec(open('tools/dlc10_jtag.py').read().split('def main')[0]); "
         "j=DLC10(); j.open(); print(f'IDCODE: 0x{j.read_idcode():08X}'); j.close()"],
        check=False,
    )


def cmd_detect(args):
    rc = _run([OPENFPGA, "--cable", CABLE, "--detect"], check=False)
    if rc != 0:
        print(f"[fallback] openFPGALoader cable '{CABLE}' not found, trying DLC-10...")
        return _dlc10_detect()
    return rc


def cmd_program(args):
    rc = _run([OPENFPGA, "--cable", CABLE, args.bitstream], check=False)
    if rc != 0:
        print("[fallback] using DLC-10 Python driver (SRAM only)...")
        return _dlc10_program(args.bitstream)
    return rc


def cmd_flash(args):
    rc = _run(
        [OPENFPGA, "--cable", CABLE, "--write-flash", args.bitstream],
        check=False,
    )
    if rc != 0:
        print("[error] SPI flash requires openFPGALoader with FTDI cable (JTAG-HS2/HS3).")
        print("        DLC-10 Python driver does not support SPI flash programming.")
        print("        Options: (1) get Digilent JTAG-HS2 cable, (2) extend dlc10_jtag.py")
        return 1
    return rc


def cmd_verify(args):
    return _run(
        [OPENFPGA, "--cable", CABLE, "--verify-flash", args.bitstream],
        check=False,
    )


def cmd_erase(args):
    return _run([OPENFPGA, "--cable", CABLE, "--unprotect-flash"], check=False)


def cmd_reset(args):
    rc = _run([OPENFPGA, "--cable", CABLE, "--reset"], check=False)
    if rc != 0:
        print("[fallback] power-cycle the FPGA board manually")
    return rc


def cmd_status(args):
    return _run(
        ["python3", DLC10_SCRIPT, args.bitstream],
        check=False,
    )


def main():
    p = argparse.ArgumentParser(
        prog="tri-fpga",
        description="Trinity FPGA lifecycle CLI (detect/program/flash/bench)",
    )
    sub = p.add_subparsers(dest="cmd")

    sub.add_parser("detect", help="Detect FPGA via JTAG (IDCODE)")

    sp = sub.add_parser("program", help="Program SRAM (volatile)")
    sp.add_argument("bitstream")

    sp = sub.add_parser("flash", help="Program SPI flash (permanent)")
    sp.add_argument("bitstream")

    sp = sub.add_parser("verify", help="Verify SPI flash contents")
    sp.add_argument("bitstream")

    sub.add_parser("erase", help="Unprotect + erase SPI flash")
    sub.add_parser("reset", help="Reset FPGA")

    sp = sub.add_parser("status", help="Read STATUS register after program")
    sp.add_argument("bitstream")

    args = p.parse_args()

    dispatch = {
        "detect": cmd_detect,
        "program": cmd_program,
        "flash": cmd_flash,
        "verify": cmd_verify,
        "erase": cmd_erase,
        "reset": cmd_reset,
        "status": cmd_status,
    }

    if args.cmd in dispatch:
        sys.exit(dispatch[args.cmd](args))
    else:
        p.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
