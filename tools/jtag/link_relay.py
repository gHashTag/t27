#!/usr/bin/env python3
"""Carry a ternary symbol from one die to another, with the host as transport.

WHAT THIS IS AND IS NOT. There is no wire between the boards; the three cables
reach the host and nothing else. So this is not a link in the electrical sense.
What it demonstrates is the thing the project had never done: a value produced
by the fabric of ONE die, read out, delivered into the fabric of ANOTHER, and
decoded there -- with neither host arithmetic on the payload nor a second
implementation of the code anywhere in the path.

PROTOCOL. `link_node` exposes a bidirectional 32-bit BSCANE2 register:
CAPTURE loads `{28'hA5A5A5A, reply}`, SHIFT walks it out while the next command
walks in, UPDATE latches the low nibble as the new command. One DR pass is
therefore one request AND the previous response, so a request-and-read is two
passes -- the second one's command is discarded.

THE DELIMITER IS THE POINT. The decoder inverts by sweeping its OWN encoder, so
it cannot disagree with the encoder about the code -- only about whether a
preimage exists. Codeword 4'b0101 (+1,+1) has none, by the theorem in
specs/fpga/ternary_link.t27, and a searching decoder must fail to match it.
Sending the delimiter across is a silicon test of that claim, not a round trip.

Refs #1959
"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mpsse_jtag import Mpsse, CLK_BITS_IO_NEG  # noqa: E402

USER_OPCODE = {1: 0x02, 2: 0x03, 3: 0x22, 4: 0x23}
MAGIC = 0xA5A5A5A
CLK_TMS_OUT_NEG = 0x4B
CLK_BITS_OUT_NEG = 0x1B


def shift_ir(m, val):
    m.tms([1, 1, 0, 0])
    m.w(bytes([CLK_BITS_OUT_NEG, 4, val & 0x1F]))
    m.w(bytes([CLK_TMS_OUT_NEG, 0, (((val >> 5) & 1) << 7) | 0x01]))
    m.tms([1, 0])


def shift_dr_rw(m, out_word, nbits=32):
    """Clock `out_word` in on TDI while reading TDO. The existing transport
    already drives TDI -- it just always drove zeros."""
    data = out_word.to_bytes(4, "little")
    m.w(bytes([0x39, 3, 0]) + data)          # 4 bytes, write TDI + read TDO
    got = m.r(4)
    return int.from_bytes(got[:4], "little") if len(got) >= 4 else None


def xfer(idx, cmd, chain=3):
    """One DR pass: deliver `cmd`, return the word captured before it."""
    m = Mpsse(idx)
    try:
        m.tms([1, 1, 1, 1, 1])
        m.tms([0])
        shift_ir(m, USER_OPCODE[chain])
        m.tms([1, 0, 0])                     # Select-DR, Capture-DR, Shift-DR
        # THE EXIT1-DR TRANSITION CLOCKS ONE MORE BIT. `m.tms([1,1,0])` moves
        # Shift-DR -> Exit1-DR -> Update-DR, and the first of those TMS clocks
        # shifts the register a 33rd time, so what UPDATE latches is the word
        # shifted RIGHT by one. Measured, not guessed: with the naive word the
        # encoder answered ENC[cmd >> 1] for all eight commands. Pre-shift LEFT
        # by one and the die latches exactly `cmd`. The READ path is unaffected
        # -- TDO presents sr[0] before each clock, so the captured word comes
        # back aligned.
        w = shift_dr_rw(m, ((MAGIC << 4 | (cmd & 0xF)) << 1) & 0xFFFFFFFF)
        m.tms([1, 1, 0])                     # Exit1-DR, Update-DR, RTI
        return w
    finally:
        m.close()


def request(idx, cmd, chain=3):
    """Deliver `cmd`, then read the reply it produced."""
    xfer(idx, cmd, chain)
    w = xfer(idx, cmd, chain)
    if w is None or (w >> 4) != MAGIC:
        return None
    return w & 0xF


def role_of(idx, chain=3):
    """Encoder and decoder answer differently to codeword 10.
    An encoder reads it as v = 10 & 7 = 2 and returns on_comb(2) = 9.
    A decoder searches for the v encoding to 10 and returns 0."""
    r = request(idx, 10, chain)
    if r is None:
        return None, None
    return ({9: "enc", 0: "dec"}.get(r, "?"), r)  # dec: codeword 10 -> v=0, nomatch clear


ENC = {0: 10, 1: 8, 2: 9, 3: 2, 4: 0, 5: 1, 6: 6, 7: 4}   # from the spec
DELIM = 5


def main():
    chain = 3
    idxs = [int(a) for a in sys.argv[1:]] or [0, 1, 2]
    print(f"probing {len(idxs)} cable(s) on USER{chain}")
    roles = {}
    for i in idxs:
        r, raw = role_of(i, chain)
        print(f"  index {i}: role={r}  (probe returned {raw})")
        if r in ("enc", "dec"):
            roles.setdefault(r, i)
    if "enc" not in roles or "dec" not in roles:
        print("need one encoder and one decoder on the bus")
        return 1

    a, b = roles["enc"], roles["dec"]
    print(f"\nencoder on index {a}, decoder on index {b}\n")
    print(f"  {'v':>2} {'A: codeword':>12} {'expected':>9} {'B: recovered':>13}  verdict")
    ok = 0
    for v in range(8):
        code = request(a, v, chain)
        back = request(b, code, chain) if code is not None else None
        # reply is {nomatch, v}: bit 3 set means the decoder found no preimage.
        good = (code == ENC[v]) and (back == v)
        ok += good
        print(f"  {v:>2} {str(code):>12} {ENC[v]:>9} {str(back):>13}  {'OK' if good else 'MISMATCH'}")
    print(f"\n  {ok} of 8 words crossed intact")

    print(f"\n  delimiter test -- codeword {DELIM} = (+1,+1) has no preimage")
    d = request(b, DELIM, chain)
    # W720: reply bit 3 is `nomatch`. Before it reached the wire, "no preimage"
    # and "recovered v = 0" were the SAME reply and this test proved nothing.
    print(f"  decoder answered {d} (nomatch={'1' if d is not None and d & 8 else '0'}, v={d & 7 if d is not None else '-'})")
    print("  ->  " + ("NO MATCH, as the theorem requires" if d is not None and (d & 8)
                      else f"MATCHED v={d} -- the theorem is violated on silicon"))
    return 0 if ok == 8 else 1


if __name__ == "__main__":
    sys.exit(main())
