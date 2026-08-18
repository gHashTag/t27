"""An independent Python model of the ternary primitives, transcribed from the SPEC.

Independence is the whole point, so it matters how this was written: each function
below was read from `specs/ternary/ternary_ripple_adder.t27` and re-expressed here.
None of it was derived from the generated C or Rust. If it had been, it would agree
with them by construction and prove nothing.

Transcribed faithfully, including the parts that look wrong. `pack2` does not mask its
arguments to two bits, so a value above 3 spills into the neighbouring trit position.
That is what the spec says; a model that "fixed" it would report a divergence that is
really the model disagreeing with the specification it is supposed to encode.

Widths are emulated where the spec names them: `dot27` accumulates in i16 and `tmul`
returns i8. With 27 terms of magnitude 1 no wrap occurs, but the wrap is applied anyway
rather than assumed away.
"""


def _i16(v):
    return ((v + (1 << 15)) & 0xFFFF) - (1 << 15)


def _i8(v):
    return ((v + (1 << 7)) & 0xFF) - (1 << 7)


def tmul(ta, tb):
    """if ta == 1 -> 0; if tb == 1 -> 0; if ta == tb -> 1; else -1."""
    if ta == 1:
        return 0
    if tb == 1:
        return 0
    if ta == tb:
        return 1
    return _i8(-1)


def dot27(a, b):
    """Sum of tmul over 27 trit lanes, two bits each, accumulated in i16."""
    acc = 0
    for i in range(27):
        ta = (a >> (i << 1)) & 3
        tb = (b >> (i << 1)) & 3
        acc = _i16(acc + tmul(ta, tb))
    return acc


def sign0(v):
    """v > 0 -> 2; v < 0 -> 0; else 1."""
    return 2 if v > 0 else (0 if v < 0 else 1)


def negate(t):
    """2 -> 0; 0 -> 2; anything else -> 1."""
    return 0 if t == 2 else (2 if t == 0 else 1)


_Z = 6004799503160661


def pack2(t0, t1):
    """Note: t0 and t1 are NOT masked to two bits. Spec behaviour, reproduced."""
    cleared = _Z & 18446744073709551600
    return (cleared | (t0 & 0xFFFFFFFFFFFFFFFF) | ((t1 & 0xFFFFFFFFFFFFFFFF) << 2)) \
        & 0xFFFFFFFFFFFFFFFF


def pack3(t0, t1, t2):
    cleared = _Z & 18446744073709551552
    return (cleared | t0 | (t1 << 2) | (t2 << 4)) & 0xFFFFFFFFFFFFFFFF


def bneuron(x, w, bias):
    return sign0(_i16(dot27(x, w) + bias))


def xor2(a, b):
    x = pack2(a, b)
    w = pack2(2, 2)
    h1 = bneuron(x, w, -1)
    h2 = bneuron(x, w, 1)
    return bneuron(pack2(h2, negate(h1)), w, -1)


def maj3(a, b, c):
    return sign0(dot27(pack3(a, b, c), 12009599006321322))


def full_adder(a, b, cin):
    """Sum trit in bits[1:0], carry trit in bits[3:2]."""
    s = xor2(xor2(a, b), cin)
    carry = maj3(a, b, cin)
    return ((carry << 2) | s) & 0xFF
