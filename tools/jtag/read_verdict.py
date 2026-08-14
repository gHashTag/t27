#!/usr/bin/env python3
"""Read the MVP's verdict out of the FPGA through JTAG USER3.

WHAT THIS IS FOR.  Everything this project has put on silicon has rested on a
lamp. The equivalence miter proves the arithmetic for all 256 inputs (T110) and
the on-chip sweep re-checks them ~250,000 times a second -- but `Done 0x1` says
only that the fabric was configured, and a blinking LED says nothing a machine
can read. This reads the answer.

THE WORD.  `mvp_ternary_classifier_jtag_noport` loads a 32-bit register on
CAPTURE and shifts it out LSB first:

    [31:4]  28-bit magic 0xA5A5A5A
    [3]     constant 0
    [2]     constant 1
    [1]     beat   -- heartbeat, so a stuck chain is distinguishable from a live one
    [0]     ok     -- 1 = every input classified correctly since power-up

WHY THE MAGIC IS 28 BITS.  W675 read a four-bit version ten times from a
bitstream containing NO BSCANE2 and got the same two values in the same
proportions as the real design (T139). Two constant bits are not enough entropy
to prove provenance. 0xA5A5A5A cannot be produced by a TAP that is not shifting
this register: if it comes back, the bits below it came from here; if it does
not, `ok` means nothing whatever its value.

ADDRESSING.  All three cables in this project share serial 210512180081, so
libftdi index is the only handle this transport has, and it is not the same
handle as openFPGALoader's --busdev-num. Read every index and report each --
identifying the board by WHAT IT ANSWERS is more reliable than assuming a
mapping between two different enumerations.

Refs #1959
"""
import sys

sys.path.insert(0, "tools/jtag")
from mpsse_jtag import Mpsse, CLK_BITS_OUT_NEG, CLK_TMS_OUT_NEG  # noqa: E402

# 7-series user instruction opcodes, indexed by JTAG_CHAIN number.
#
# W693: this used to be a constant `USER3 = 0x22`, because W690's build happened
# to place BSCANE2 at site 3. A compiler change in W692 moved the placement to
# site 2 and the constant became wrong -- silently, because a wrong chain reads
# all-zero, which is indistinguishable from a design that is not there.
#
# The chain is a property of THIS BUILD, not of the project. `t27c silicon`
# derives it from the FASM and passes it in.
USER_OPCODE = {1: 0x02, 2: 0x03, 3: 0x22, 4: 0x23}
MAGIC = 0xA5A5A5A


def shift_ir(m, val):
    """Load a 6-bit instruction. The last bit exits with TMS=1 into Exit1-IR."""
    m.tms([1, 1, 0, 0])                                        # Select-DR, Select-IR, Capture-IR, Shift-IR
    m.w(bytes([CLK_BITS_OUT_NEG, 4, val & 0x1F]))              # 5 bits, LSB first
    m.w(bytes([CLK_TMS_OUT_NEG, 0, (((val >> 5) & 1) << 7) | 0x01]))
    m.tms([1, 0])                                              # Update-IR, Run-Test/Idle


def read_word(idx, chain=3):
    """One 32-bit read of USER<chain> from the cable at libftdi index `idx`."""
    m = Mpsse(idx)
    try:
        m.tms([1, 1, 1, 1, 1])          # Test-Logic-Reset
        m.tms([0])                      # Run-Test/Idle
        shift_ir(m, USER_OPCODE[chain])
        m.tms([1, 0, 0])                # Select-DR, Capture-DR, Shift-DR
        raw = m.shift_dr_read(32)
        m.tms([1, 1, 0])                # Exit1-DR, Update-DR, RTI
        if len(raw) < 4:
            return None
        return int.from_bytes(raw[:4], "little")
    finally:
        m.close()


def report(idx, reads=5, chain=3):
    words = []
    for _ in range(reads):
        try:
            words.append(read_word(idx, chain))
        except SystemExit as e:
            print(f"  index {idx}: cannot open -- {e}")
            return None
        except Exception as e:  # noqa: BLE001
            print(f"  index {idx}: {type(e).__name__}: {e}")
            return None

    print(f"  index {idx}: " + " ".join("None" if w is None else f"{w:08x}" for w in words))
    good = [w for w in words if w is not None]
    if not good:
        print("           no data")
        return None

    # A chain that is dead returns all-zero or all-one; neither can carry the magic.
    if all(w == 0 for w in good):
        print("           ALL ZERO -- dead chain or no BSCANE2 in this bitstream")
        return None
    if all(w == 0xFFFFFFFF for w in good):
        print("           ALL ONES -- dead chain")
        return None

    for w in good:
        if (w >> 4) == MAGIC:
            ok = w & 1
            beat = (w >> 1) & 1
            c1 = (w >> 2) & 1
            c0 = (w >> 3) & 1
            print(f"           MAGIC PRESENT 0x{MAGIC:07X}  const={c1}{c0}  beat={beat}  ok={ok}")
            print("           VERDICT: " + ("PASS -- silicon classified every input correctly"
                                            if ok else "FAIL -- a wrong class was latched"))
            return w
    print("           magic absent -- this is not our design, or the chain is not carrying it")
    return None


if __name__ == "__main__":
    # usage: read_verdict.py [--chain N] [idx ...]
    argv = sys.argv[1:]
    chain = 3
    if "--chain" in argv:
        k = argv.index("--chain")
        chain = int(argv[k + 1])
        del argv[k:k + 2]
    idxs = [int(a) for a in argv] or [0, 1, 2]
    print(f"reading USER{chain} (IR=0x{USER_OPCODE[chain]:02x}), magic 0x{MAGIC:07X}, {len(idxs)} cable(s)")
    hits = [i for i in idxs if report(i, chain=chain) is not None]
    print()
    print(f"cables carrying the magic: {len(hits)} of {len(idxs)}  {hits}")
    sys.exit(0 if hits else 1)
