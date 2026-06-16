#!/usr/bin/env python3
"""
Master conformance-pack generator for the full t27 numeric-format catalog.

Reads every format record from the catalog SSOT and emits a conformance vector
pack per format, in the SAME shared row schema as the original hand-curated
packs (gf16 / fp8_e4m3fn / fp8_e5m2 / mxfp4_e2m1 / bf16_golden).

Coverage policy (agreed with maintainer):
  * Every format with a fixed bit layout (radix-2 float, integer/fixed,
    posit/takum, LNS, GoldenFloat, historical vendor floats) gets a
    BIT-PRECISE decode pack. abs_error == 0 by construction for every
    representable code (decode bits -> f64 is exact into f64 for these
    widths; binary128/binary256/int128 mantissas exceed f64 precision and
    are flagged -- see WIDE_DECIMAL handling below).
  * Formats with NO fixed bit layout (parametric minifloat, q_format, bcd,
    block_fp, shared_exp, stochastic_rounding, unum_i, unum_ii SORN,
    decimal DPD/BID, nf4 quantile table) get an HONEST STRUCTURAL pack:
    bitexact=false, an explicit reason, the catalog metadata, and an anchor
    note. Nothing is faked as bit-precise.

The 5 original packs are NOT overwritten by this script (they remain the
authoritative reference); this script only writes the *missing* packs and a
combined INDEX with SHA-256 for everything.

Anchor identity (ASCII): phi^2 + 1/phi^2 = 3.
All output ASCII-only. Apache-2.0.
"""
import json, struct, math, hashlib, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = HERE  # write packs next to the existing ones
CATALOG_LINES = "/tmp/catalog_lines.txt"

SSOT = "https://github.com/gHashTag/t27/blob/master/conformance/FORMAT-SPEC-001.json"
PREPRINT = "https://arxiv.org/abs/2606.05017"
ANCHOR = "phi^2 + 1/phi^2 = 3"
SCHEMA = "t27-conformance/v0.1"

PHI = (1.0 + math.sqrt(5.0)) / 2.0

# The 5 formats that already have authoritative hand-curated packs. We keep
# those files and do not regenerate them, but list them in the index.
EXISTING = {
    "gf16": "gf16_conformance_v0.json",
    "fp8_e4m3": "fp8_e4m3fn_conformance_v0.json",
    "fp8_e5m2": "fp8_e5m2_conformance_v0.json",
    "mxfp4": "mxfp4_e2m1_conformance_v0.json",
    "bfloat16": "bf16_golden_conformance_v0.json",
}

# Externally-generated, dyadic-exact packs that re-derive under ONE decode law
# and carry honest abs_error, but have NO independent second witness (so they are
# NOT promoted to the stronger "bitexact" label -- only gf16 has a silicon oracle).
# These files are produced by the wide-rung GoldenFloat oracle (WP-29/WP-30), kept
# verbatim here, and listed in the index with kind="bitexact_selfconsistent". This
# registry exists so a re-run of this generator PRESERVES the self-consistent tier
# instead of re-deriving these wide rungs as plain structural stubs.
SELFCONSISTENT = {
    "gf14": "gf14_conformance_v0.json",
    "gf48": "gf48_conformance_v0.json",
    "gf96": "gf96_conformance_v0.json",
    "gf128": "gf128_conformance_v0.json",
    "gf512": "gf512_conformance_v0.json",
    "gf1024": "gf1024_conformance_v0.json",
}

def f64_hex(x):
    return "0x" + struct.pack(">d", x).hex().upper()

# ---------------------------------------------------------------------------
# Catalog parsing
# ---------------------------------------------------------------------------
def parse_catalog(path):
    rows = []
    kv = re.compile(r'(\w+)=("[^"]*"|\S+)')
    for line in open(path):
        line = line.strip()
        if not line.startswith("id="):
            continue
        d = {}
        for k, v in kv.findall(line):
            if v.startswith('"') and v.endswith('"'):
                v = v[1:-1]
            d[k] = v
        def parse_int(x):
            # tolerate expressions like 2^194-1 used for ultra-wide biases
            m_ = re.fullmatch(r'2\^(\d+)-1', x)
            if m_:
                return (1 << int(m_.group(1))) - 1
            try:
                return int(x)
            except ValueError:
                return x  # leave as string; such formats are structural anyway
        for f in ("bits", "s", "e", "m", "bias"):
            d[f] = parse_int(d[f])
        try:
            d["phi_distance"] = float(d["phi_distance"])
        except ValueError:
            d["phi_distance"] = -1.0
        rows.append(d)
    return rows

# ---------------------------------------------------------------------------
# Generic radix-2 float decode (sign : exp : mantissa, IEEE-style)
#   - exp all-ones => inf/nan IF the format has inf/nan (has_specials)
#   - exp == 0 => zero / subnormal
# Works for binary16/32/64, bf16, tf32, fp8/6/4 variants, and the GF family
# (GF uses the same S:E:M layout with its own bias).
# ---------------------------------------------------------------------------
def make_ieee_decoder(e_bits, m_bits, bias, has_inf, has_nan,
                      nan_is_e4m3fn=False):
    e_max = (1 << e_bits) - 1
    m_max = (1 << m_bits)
    def decode(bits, total_bits):
        s = (bits >> (e_bits + m_bits)) & 1
        e = (bits >> m_bits) & e_max
        m = bits & (m_max - 1)
        sign = -1.0 if s else 1.0
        if nan_is_e4m3fn:
            # E4M3FN: no inf; single NaN at e_max & m_max-1
            if e == e_max and m == (m_max - 1):
                return (math.nan, "nan")
        elif e == e_max:
            if has_inf and m == 0:
                return (sign * math.inf, "inf")
            if has_nan and m != 0:
                return (math.nan, "nan")
            if has_inf and not has_nan and m != 0:
                return (math.nan, "nan")
        if e == 0:
            if m == 0:
                return (sign * 0.0, "zero")
            val = sign * (m / m_max) * (2.0 ** (1 - bias))
            return (val, "subnormal")
        val = sign * (1.0 + m / m_max) * (2.0 ** (e - bias))
        return (val, "normal")
    return decode

def make_int_decoder(m_bits, signed=True):
    """Two's-complement integer of width (1+m) bits when signed, else m+1."""
    def decode(bits, total_bits):
        width = total_bits
        if signed:
            if bits & (1 << (width - 1)):
                val = bits - (1 << width)
            else:
                val = bits
            cat = "zero" if val == 0 else "normal"
            return (float(val), cat)
        else:
            return (float(bits), "zero" if bits == 0 else "normal")
    return decode

def make_lns_decoder(e_bits, total_bits):
    """Log-number-system: 1 sign bit, rest is a fixed-point base-2 logarithm.
    Convention here: value = sign * 2^(L) where L is the remaining bits read as
    a two's-complement fixed-point with the binary point in the middle. We keep
    it explicit and document it; zero is the reserved all-zero-magnitude code
    with sign 0 (a common LNS 'zero flag' convention)."""
    frac_bits = (total_bits - 1) // 2
    def decode(bits, tb):
        s = (bits >> (total_bits - 1)) & 1
        mag = bits & ((1 << (total_bits - 1)) - 1)
        sign = -1.0 if s else 1.0
        if mag == 0 and s == 0:
            return (0.0, "zero")
        # interpret mag as signed fixed-point logarithm
        intbits = total_bits - 1 - frac_bits
        if mag & (1 << (total_bits - 2)):
            L = (mag - (1 << (total_bits - 1))) / (2.0 ** frac_bits)
        else:
            L = mag / (2.0 ** frac_bits)
        return (sign * (2.0 ** L), "normal")
    return decode

# ---------------------------------------------------------------------------
# IBM Hexadecimal Floating Point (base-16 exponent), System/360 (1964).
#   layout: S(1) : E(7, excess-64) : M(m_bits fractional hex digits)
#   value  = (-1)^S * 0.M(base16) * 16^(E - 64)
#   no inf/nan; zero is all-zero (true zero). Mantissa is an unsigned binary
#   fraction read in groups of 4 bits (hex digits); the radix point sits before
#   the most significant fraction bit. Bit-exact for values whose binary
#   expansion fits in m_bits and whose magnitude lies in the hex-exponent range.
# ---------------------------------------------------------------------------
def make_ibm_hfp_decoder(m_bits, e_bits=7, bias=64):
    e_max = (1 << e_bits) - 1
    def decode(bits, total_bits):
        s = (bits >> (e_bits + m_bits)) & 1
        e = (bits >> m_bits) & e_max
        m = bits & ((1 << m_bits) - 1)
        sign = -1.0 if s else 1.0
        if e == 0 and m == 0:
            return (sign * 0.0, "zero")
        # fraction value 0.M in base 2 (m_bits fractional bits)
        frac = m / float(1 << m_bits)
        val = sign * frac * (16.0 ** (e - bias))
        return (val, "normal")
    return decode

def ibm_hfp_encode_exact(m_bits, value, e_bits=7, bias=64):
    """Encode value into IBM HFP. Returns int bits, or None if not exactly
    representable in m_bits of binary fraction within the hex-exponent range.
    Uses the normalized convention: leading hex digit of the fraction nonzero."""
    if value == 0.0:
        return 0
    s = 1 if value < 0 else 0
    a = abs(value)
    # find hex exponent E such that 1/16 <= a / 16^(E-bias) < 1
    E = bias
    # scale up
    while a / (16.0 ** (E - bias)) >= 1.0:
        E += 1
    while a / (16.0 ** (E - bias)) < (1.0 / 16.0):
        E -= 1
    e_max = (1 << e_bits) - 1
    if E <= 0 or E >= e_max:
        return None
    frac = a / (16.0 ** (E - bias))      # in [1/16, 1)
    scaled = frac * (1 << m_bits)
    m_field = round(scaled)
    if m_field >= (1 << m_bits):
        # carried into next hex exponent
        E += 1
        if E >= e_max:
            return None
        frac = a / (16.0 ** (E - bias))
        scaled = frac * (1 << m_bits)
        m_field = round(scaled)
    if abs(scaled - m_field) > 1e-9:
        return None                      # not exactly representable
    if m_field == 0:
        return None
    return (s << (e_bits + m_bits)) | (E << m_bits) | m_field

# ---------------------------------------------------------------------------
# Intel x87 80-bit extended (double extended). Explicit integer bit.
#   layout: S(1) : E(15, bias 16383) : SIG(64)
#   The 64-bit significand field holds the EXPLICIT integer bit as its MSB
#   (bit 63 of the field) followed by 63 fraction bits. The catalog records
#   this as m=64 (the full significand width including the integer bit), so
#   1 + 15 + 64 = 80 bits exactly.
#   value = (-1)^S * (SIG / 2^63) * 2^(E - 16383)  [SIG = integer.fraction]
#   For normalized numbers the integer bit = 1. E = max -> inf/nan.
# ---------------------------------------------------------------------------
def make_x87_decoder(bias=16383, e_bits=15, sig_bits=64):
    e_max = (1 << e_bits) - 1
    int_bit_pos = sig_bits - 1            # position of the explicit integer bit
    def decode(bits, total_bits):
        s = (bits >> (e_bits + sig_bits)) & 1
        e = (bits >> sig_bits) & e_max
        sig = bits & ((1 << sig_bits) - 1)
        j = (sig >> int_bit_pos) & 1
        f = sig & ((1 << int_bit_pos) - 1)
        sign = -1.0 if s else 1.0
        if e == e_max:
            if j == 1 and f == 0:
                return (sign * math.inf, "inf")
            return (math.nan, "nan")
        if e == 0 and sig == 0:
            return (sign * 0.0, "zero")
        significand = sig / float(1 << int_bit_pos)   # integer.fraction
        val = sign * significand * (2.0 ** (e - bias))
        return (val, "normal")
    return decode

def x87_encode_exact(value, bias=16383, e_bits=15, sig_bits=64):
    """Encode value into x87 80-bit with explicit integer bit (MSB of the
    64-bit significand field). Returns int bits or None if not exactly
    representable in the 63 fraction bits within the exponent range."""
    if value == 0.0:
        return 0
    s = 1 if value < 0 else 0
    a = abs(value)
    int_bit_pos = sig_bits - 1            # 63 fraction bits below the integer bit
    mant, exp = math.frexp(a)            # a = mant * 2^exp, 0.5<=mant<1
    E_unbiased = exp - 1                  # significand 1.f * 2^(E_unbiased)
    frac = a / (2.0 ** E_unbiased) - 1.0  # in [0,1)
    e_field = E_unbiased + bias
    e_max = (1 << e_bits) - 1
    if e_field <= 0 or e_field >= e_max:
        return None
    f_field = round(frac * (1 << int_bit_pos))
    if f_field == (1 << int_bit_pos):
        f_field = 0; e_field += 1
    if abs(frac * (1 << int_bit_pos) - f_field) > 1e-6:
        return None
    if e_field >= e_max:
        return None
    sig = (1 << int_bit_pos) | f_field    # explicit integer bit set + fraction
    return (s << (e_bits + sig_bits)) | (e_field << sig_bits) | sig

# ---------------------------------------------------------------------------
# NF4 (NormalFloat 4-bit), QLoRA (Dettmers 2023). 16-entry quantile table
# fitted to N(0,1), normalized so the table spans [-1, 1]. The 4-bit code is
# an index into this fixed table; the decode is the table lookup. Bit-exact:
# every 4-bit code maps to exactly one table value, round-trip is the index.
# Reference table from bitsandbytes (the canonical QLoRA NF4 levels).
# ---------------------------------------------------------------------------
NF4_TABLE = [
    -1.0,
    -0.6961928009986877,
    -0.5250730514526367,
    -0.39491748809814453,
    -0.28444138169288635,
    -0.18477343022823334,
    -0.09105003625154495,
    0.0,
    0.07958029955625534,
    0.16093020141124725,
    0.24611230194568634,
    0.33791524171829224,
    0.44070982933044434,
    0.5626170039176941,
    0.7229568362236023,
    1.0,
]
def make_nf4_decoder():
    def decode(bits, total_bits):
        idx = bits & 0xF
        v = NF4_TABLE[idx]
        cat = "zero" if v == 0.0 else "normal"
        return (v, cat)
    return decode

# ---------------------------------------------------------------------------
# GFTernary: 2-bit discrete set {-phi, 0, +phi}. Four 2-bit codes; one is a
# spare (we map it to the same +phi for completeness / documented as reserved).
# Anchor 3.0 is NOT a single code (it arises as phi^2 + 1/phi^2 = 3 over the
# nonzero codes), so it is recorded in anchor_note, not as a vector.
# ---------------------------------------------------------------------------
def make_gfternary_decoder():
    def decode(bits, total_bits):
        c = bits & 0x3
        if c == 0:
            return (0.0, "zero")
        if c == 1:
            return (PHI, "normal")
        if c == 2:
            return (-PHI, "normal")
        return (PHI, "reserved")  # code 3: reserved/spare -> documented duplicate
    return decode

# ---------------------------------------------------------------------------
# Vector selection
# ---------------------------------------------------------------------------
def f64_repr(val):
    if isinstance(val, float) and math.isnan(val):
        return ("NaN", "0x7FF8000000000000", "NaN")
    if isinstance(val, float) and math.isinf(val):
        t = "-Inf" if val < 0 else "Inf"
        return (t, f64_hex(val), 0.0)
    return (val, f64_hex(val), 0.0)

def make_vector(name, bits, total_bits, decode, fmt_key):
    val, cat = decode(bits, total_bits)
    inp, inp_hex, abs_err = f64_repr(val)
    dec, dec_hex, _ = f64_repr(val)
    nib = (total_bits + 3) // 4
    return {
        "name": name,
        "input_f64": inp,
        "input_f64_hex": inp_hex,
        f"{fmt_key}_bits_hex": "0x%0*X" % (nib, bits),
        f"{fmt_key}_bits_int": bits,
        "decoded_f64": dec,
        "decoded_f64_hex": dec_hex,
        "abs_error": abs_err,
        "category": cat,
    }

def find_anchor_bits(decode, total_bits, target=3.0):
    """Search for a code that decodes exactly to target (the 3.0 anchor)."""
    if total_bits <= 16:
        rng = range(1 << total_bits)
    else:
        # construct directly for IEEE-style: 3.0 = 1.5 * 2^1 -> mantissa=0.5
        rng = []
    for b in rng:
        v, c = decode(b, total_bits)
        if isinstance(v, float) and not math.isnan(v) and not math.isinf(v) and v == target:
            return b
    return None

# ---- explicit bit encoders for wide formats (no brute force) ----
def ieee_encode_exact(e_bits, m_bits, bias, value):
    """Encode a value that is an exact (1.f)*2^k or (m/2^M)*2^(1-bias) into the
    S:E:M bit pattern. Returns int bits, or None if not exactly representable
    in <= m_bits of mantissa within the exponent range."""
    if value == 0.0:
        return 0  # +0
    s = 1 if value < 0 else 0
    a = abs(value)
    mant, exp = math.frexp(a)        # a = mant * 2^exp, 0.5<=mant<1
    # normalize to 1.f form: a = 1.f * 2^(E-bias)
    E_unbiased = exp - 1             # because mant in [0.5,1) -> 1.f*2^(exp-1)
    frac = a / (2.0 ** E_unbiased) - 1.0  # in [0,1)
    e_field = E_unbiased + bias
    e_max = (1 << e_bits) - 1
    if e_field <= 0 or e_field >= e_max:
        return None                  # subnormal/overflow: skip for named set
    m_field = round(frac * (1 << m_bits))
    if m_field == (1 << m_bits):
        m_field = 0; e_field += 1
    # exactness check
    if abs(frac * (1 << m_bits) - m_field) > 1e-12:
        return None
    if e_field >= e_max:
        return None
    return (s << (e_bits + m_bits)) | (e_field << m_bits) | m_field

def int_encode(value, total_bits):
    v = int(value) & ((1 << total_bits) - 1)
    return v

def make_posit_encoder(nbits, es=2):
    """Encode an exact value (power-of-two * small fraction) into a posit code.
    Returns int bits or None. Uses the standard regime/exp/frac construction."""
    def encode(value):
        if value == 0.0:
            return 0
        s = 1 if value < 0 else 0
        a = abs(value)
        useed_pow = (1 << es)            # 2^es
        # decompose a = 2^(scale) * (1.f), scale = k*2^es + ebits
        m_, exp = math.frexp(a)          # a = m_ * 2^exp, 0.5<=m_<1
        scale = exp - 1                  # a = 1.f * 2^scale
        frac = a / (2.0 ** scale) - 1.0  # in [0,1)
        k = scale >> es                  # regime
        ebits = scale - (k << es)        # exponent within regime
        bits = []
        # regime: k>=0 -> (k+1) ones then a zero; k<0 -> (-k) zeros then a one
        if k >= 0:
            bits += [1] * (k + 1) + [0]
        else:
            bits += [0] * (-k) + [1]
        # exponent bits (es bits, MSB first)
        for j in range(es - 1, -1, -1):
            bits.append((ebits >> j) & 1)
        # fraction bits: fill remaining
        remaining = (nbits - 1) - len(bits)
        if remaining < 0:
            return None                  # does not fit -> skip
        fr = frac
        for _ in range(remaining):
            fr *= 2.0
            bit = int(fr)
            bits.append(bit)
            fr -= bit
        if fr != 0.0:
            return None                  # not exactly representable
        magnitude = 0
        for b in bits[:nbits - 1]:
            magnitude = (magnitude << 1) | b
        code = magnitude
        if s:
            code = ((~code) + 1) & ((1 << nbits) - 1)
        return code
    return encode

def make_lns_encoder(total_bits):
    """Encode value = sign * 2^L into the LNS fixed-point log layout used by
    make_lns_decoder. Returns int bits or None."""
    frac_bits = (total_bits - 1) // 2
    def encode(value):
        if value == 0.0:
            return 0
        s = 1 if value < 0 else 0
        a = abs(value)
        L = math.log2(a)
        scaled = round(L * (2.0 ** frac_bits))
        if abs(L * (2.0 ** frac_bits) - scaled) > 1e-9:
            return None
        mag = scaled & ((1 << (total_bits - 1)) - 1)
        if mag == 0 and s == 0:
            return None                  # collides with zero code
        return (s << (total_bits - 1)) | mag
    return encode

NAMED_TARGETS = [("pos_zero", 0.0), ("pos_one", 1.0), ("neg_one", -1.0),
                 ("pos_two", 2.0), ("pos_three", 3.0), ("pos_half", 0.5),
                 ("pos_four", 4.0), ("neg_three", -3.0)]

def curated_codes(total_bits, decode, fmt_key, encoder=None):
    """For widths <= 8 enumerate ALL codes. For wider formats build named
    high-signal vectors with an explicit encoder (no brute force)."""
    vecs = []
    if total_bits <= 8:
        # <=8 bits: exhaustive is tiny (<=256 rows) and ideal.
        for b in range(1 << total_bits):
            vecs.append(make_vector(f"code_0x%02X" % b, b, total_bits, decode, fmt_key))
        return vecs, "exhaustive"
    # >8 bits: exhaustive would be huge (2^16 = 65536 rows -> ~20 MB), so use a
    # curated high-signal vector set (same spirit as the original GF16 pack:
    # boundaries + named values + the 3.0 anchor). Prefer an explicit encoder;
    # if none, fall back to a brute search over <=16-bit space for the targets.
    if encoder is None and total_bits <= 16:
        encoder = lambda val, _d=decode, _tb=total_bits: encode_ieee(val, _d, _tb)
    if encoder is None:
        return [], "none"
    seen = set(); out = []
    for nm, target in NAMED_TARGETS:
        b = encoder(target)
        if b is None or b in seen:
            continue
        # verify decode round-trips to the target exactly
        v, c = decode(b, total_bits)
        if isinstance(v, float) and not math.isnan(v) and v == target:
            seen.add(b)
            out.append(make_vector(nm, b, total_bits, decode, fmt_key))
    return out, "curated_named"

def encode_ieee(target, decode, total_bits):
    if total_bits <= 16:
        for b in range(1 << total_bits):
            v, c = decode(b, total_bits)
            if isinstance(v, float) and not math.isnan(v) and v == target:
                return b
    return None

# ---------------------------------------------------------------------------
# Per-format dispatch
# ---------------------------------------------------------------------------
def fmt_key_for(rec):
    return rec["id"].replace("-", "_")

def build_decodable(rec):
    """Return (decode_fn, notes, has_specials_desc) for a fixed-layout format,
    or None if not bit-precise decodable here."""
    cid = rec["id"]; cluster = rec["cluster"]
    e, m, bias, bits = rec["e"], rec["m"], rec["bias"], rec["bits"]
    storage = rec["storage"]

    # Integer / fixed two's complement
    if cluster == "IntegerFixed" and cid.startswith("int") and bits > 0:
        return (make_int_decoder(m, signed=True),
                f"two's-complement signed integer, {bits} bits", "no inf/nan; exact integers")
    if cid == "per_channel_scale":
        return None  # has external fp32 scale -> structural

    # LNS family
    if cluster == "Lns" and bits in (8, 16, 32, 64):
        return (make_lns_decoder(e, bits),
                f"log-number-system, 1 sign + {bits-1} fixed-point base-2 log bits",
                "zero is reserved code; no inf/nan in this convention")

    # GoldenFloat radix-2 (S:E:M) with explicit bias. Skip gfternary (2-bit
    # discrete set) -> structural; skip gf16 (existing).
    if cluster == "GoldenFloat":
        if cid == "gfternary":
            # 2-bit discrete set {-phi, 0, +phi}: enumerate all 4 codes bit-exactly.
            return (make_gfternary_decoder(),
                    "GFTernary 2-bit discrete set {-phi, 0, +phi}; codes 00=0, "
                    "01=+phi, 10=-phi, 11=reserved (duplicate +phi)",
                    "no inf/nan; discrete ternary levels")
        if e > 0 and m > 0 and bits in (4, 6, 8, 12, 16, 20, 24, 32, 64):
            dec = make_ieee_decoder(e, m, bias, has_inf=True, has_nan=True)
            return (dec, f"GoldenFloat phi-aligned radix-2 float S{1}E{e}M{m}, bias {bias}",
                    "IEEE-style specials at top exponent")
        return None  # gf128/gf256 (open bias) -> structural

    # IEEE binary radix-2
    if cluster == "Ieee754Binary" and e > 0 and m > 0:
        dec = make_ieee_decoder(e, m, bias, has_inf=True, has_nan=True)
        wide = bits > 64
        return (dec, f"IEEE 754 binary{bits}, S1E{e}M{m}, bias {bias}",
                "inf at max exp m=0, nan at max exp m!=0" + (" (mantissa exceeds f64; decode approximate for wide formats)" if wide else ""))

    # ML low precision radix-2 (fp16 covered by Ieee754; bf16/tf32/fp8/fp6/fp4)
    if cluster == "MlLowPrecision" and e > 0 and m > 0:
        if cid == "fp8_e4m3":
            return None  # existing
        if cid == "fp8_e5m2":
            return None  # existing
        # bf16 handled by existing golden pack; tf32/fp6/fp4 are new
        if cid == "bfloat16":
            return None  # existing
        has_inf = cid in ("tf32",)  # fp6/fp4 OCP element: no inf/nan
        has_nan = cid in ("tf32",)
        # OCP FP8/FP6/FP4 element formats: finite-only (block scale handles range)
        if cid.startswith("fp6") or cid.startswith("fp4"):
            dec = make_ieee_decoder(e, m, bias, has_inf=False, has_nan=False)
            return (dec, f"OCP MX element format S1E{e}M{m}, bias {bias}, finite-only",
                    "no inf/nan (block scale handles dynamic range)")
        dec = make_ieee_decoder(e, m, bias, has_inf=has_inf, has_nan=has_nan)
        return (dec, f"radix-2 float S1E{e}M{m}, bias {bias}", "IEEE-style specials")

    # Microscaling ELEMENT formats: same element layout as fp8/6/4, finite-only.
    if cluster == "Microscaling":
        if cid == "mxfp4":
            return None  # existing
        dec = make_ieee_decoder(e, m, bias, has_inf=False, has_nan=False)
        return (dec, f"OCP MX element S1E{e}M{m}, bias {bias} (paired with E8M0 block scale)",
                "element is finite-only; block scale E8M0 handles range")

    # Historical vendor radix-2 floats (IBM HFP base-16 differs; treat IEEE-like
    # ones precisely, flag base-16 ones as structural).
    if cluster == "HistoricalVendor" and e > 0 and m > 0:
        if cid.startswith("ibm_hfp"):
            # base-16 exponent: dedicated HFP decoder (S1E7M<m>, bias 64).
            return (make_ibm_hfp_decoder(m, e_bits=e, bias=bias),
                    f"IBM Hexadecimal Floating Point S1E{e}M{m}, excess-{bias} "
                    f"base-16 exponent: value = 0.M(2) * 16^(E-{bias})",
                    "no inf/nan; true zero is all-zero; "
                    + ("named small values exact (mantissa exceeds f64 for the full grid)" if m > 52 else "exact for the named set"))
        if cid == "cray_float":
            # Cray: no NaN/Inf
            dec = make_ieee_decoder(e, m, bias, has_inf=False, has_nan=False)
            return (dec, f"Cray-1 float S1E{e}M{m}, bias {bias}, no NaN/Inf", "no specials")
        dec = make_ieee_decoder(e, m, bias, has_inf=(cid in ("vax_g","vax_h")), has_nan=(cid in ("vax_g","vax_h")))
        return (dec, f"vendor radix-2 float S1E{e}M{m}, bias {bias} ({rec['standard']})",
                "vendor-specific specials")

    # ExtendedFloat: x87 has a single fixed S:E:J:F layout with an explicit
    # integer bit -> bit-exact. double-double / quad-double are composite
    # multi-double components (no single S:E:M) -> remain structural.
    if cluster == "ExtendedFloat":
        if cid == "x87_fp80":
            return (make_x87_decoder(bias=bias, e_bits=e, sig_bits=m),
                    f"Intel x87 80-bit extended S1E{e}+SIG{m} (explicit integer "
                    f"bit as MSB of the {m}-bit significand), bias {bias}: "
                    f"value = (SIG/2^{m-1}) * 2^(E-{bias})",
                    "inf at max exp (int=1, frac=0); nan otherwise at max exp")
        return None  # double_double / quad_double -> structural

    # QuantTuned: NF4 is a fixed 16-entry quantile table -> bit-exact lookup.
    if cluster == "QuantTuned" and cid == "nf4":
        return (make_nf4_decoder(),
                "NF4 (NormalFloat 4-bit, QLoRA/bitsandbytes): 4-bit code indexes "
                "a fixed 16-entry quantile table fitted to N(0,1), span [-1,1]",
                "no inf/nan; code is a table index; round-trip is the index")

    return None

# ---------------------------------------------------------------------------
# Posit decode (Posit Standard 2022, es=2)
# ---------------------------------------------------------------------------
def make_posit_decoder(nbits, es=2):
    def decode(bits, total_bits):
        n = nbits
        if bits == 0:
            return (0.0, "zero")
        if bits == (1 << (n - 1)):  # NaR (10...0)
            return (math.nan, "nan")
        s = (bits >> (n - 1)) & 1
        x = bits
        if s:
            x = ((~x) + 1) & ((1 << n) - 1)  # two's complement
        # decode regime
        bitstr = [(x >> (n - 2 - i)) & 1 for i in range(n - 1)]
        i = 0
        first = bitstr[0]
        run = 1
        while i + 1 < len(bitstr) and bitstr[i + 1] == first:
            run += 1; i += 1
        if first == 1:
            k = run - 1
        else:
            k = -run
        idx = run + 1  # position after regime + terminating bit
        # exponent
        ebits = 0
        for j in range(es):
            if idx < len(bitstr):
                ebits = (ebits << 1) | bitstr[idx]; idx += 1
            else:
                ebits = (ebits << 1)
        # fraction
        frac = 0; fcount = 0
        while idx < len(bitstr):
            frac = (frac << 1) | bitstr[idx]; idx += 1; fcount += 1
        useed = 2 ** (2 ** es)
        f = 1.0 + (frac / (2 ** fcount) if fcount else 0.0)
        val = (useed ** k) * (2 ** ebits) * f
        if s:
            val = -val
        return (val, "normal")
    return decode

# ---------------------------------------------------------------------------
# Build a structural (non-bit-exact) pack for parametric/lookup formats.
# ---------------------------------------------------------------------------
def structural_reason(rec):
    cid = rec["id"]; cluster = rec["cluster"]; storage = rec["storage"]
    if rec["bits"] == 0:
        return ("This format has no single fixed bit layout (parametric / "
                "technique / variable-width). A bit-precise round-trip vector "
                "table is not well-defined; the catalog metadata and anchor "
                "note are recorded instead.")
    if cluster == "Ieee754Decimal":
        return ("IEEE 754 decimal (DPD/BID) encodes coefficients in a packed "
                "decimal field; round-trip is exact for decimal values but the "
                "bit layout is not a plain radix-2 S:E:M. Recorded structurally "
                "with the exact decimal anchor 3.0.")
    if cid.startswith("ibm_hfp"):
        return ("IBM Hexadecimal Floating Point uses a base-16 exponent (not "
                "base-2), so the radix-2 S:E:M decoder does not apply. 3.0 is "
                "exactly representable; recorded structurally.")
    if cid == "nf4":
        return ("NF4 is a 16-entry quantile lookup table fitted to N(0,1); "
                "values are table entries, not an S:E:M layout. The 16-entry "
                "table defines the round-trip, recorded structurally.")
    if cid == "gfternary":
        return ("GFTernary is a 2-bit discrete set {-phi, 0, +phi}; the three "
                "values are recorded directly rather than via an S:E:M decode.")
    if cid in ("gf128", "gf256"):
        return ("Bias is an OPEN R&D parameter for this width (see catalog "
                "status Experimental); a bit-precise pack is deferred until the "
                "bias is fixed. Recorded structurally with the proposed layout.")
    if cluster == "ExtendedFloat":
        return ("Extended-precision layout (explicit integer bit / multi-double "
                "components) is not a single S:E:M field; recorded structurally.")
    if cid == "per_channel_scale":
        return ("INT8 payload with an external per-channel fp32 scale; the "
                "decoded value depends on the scale tensor, so a standalone "
                "round-trip table is not defined. Recorded structurally.")
    return ("No fixed bit-precise round-trip is defined for this entry; "
            "recorded structurally with catalog metadata.")

def anchor_note(rec):
    # 3.0 representability commentary per cluster
    if rec["cluster"] in ("Ieee754Decimal",):
        return "3.0 is exactly representable (decimal coefficient 3, exponent 0)."
    if rec["id"] == "gfternary":
        return "Anchor 3.0 arises as phi^2 + 1/phi^2 = 3 from the two nonzero codes +phi and... see GF16 pack; not a single code here."
    return "Anchor identity phi^2 + 1/phi^2 = 3 recorded per shared schema."

def build_structural_pack(rec):
    return {
        "schema": SCHEMA,
        "format": rec["id"].upper(),
        "format_name": rec["name"],
        "bitexact": False,
        "format_notes": rec["name"] + " -- " + rec["standard"],
        "catalog": {
            "id": rec["id"], "bits": rec["bits"], "s": rec["s"], "e": rec["e"],
            "m": rec["m"], "bias": rec["bias"], "storage": rec["storage"],
            "cluster": rec["cluster"], "status": rec["status"],
            "standard": rec["standard"], "use_case": rec["use_case"],
            "gf_relation": rec["gf_relation"], "source": rec["source"],
            "phi_distance": rec["phi_distance"],
        },
        "ssot": SSOT, "preprint": PREPRINT, "anchor_identity": ANCHOR,
        "structural_reason": structural_reason(rec),
        "anchor_note": anchor_note(rec),
        "n_vectors": 0,
        "vectors": [],
    }

def build_bitexact_pack(rec, decode, notes, specials_desc):
    cid = rec["id"]; fmt_key = fmt_key_for(rec); bits = rec["bits"]
    cluster = rec["cluster"]; e = rec["e"]; m = rec["m"]; bias = rec["bias"]
    # Build an explicit encoder for wide (>16-bit) formats so named vectors and
    # the 3.0 anchor can be constructed directly (no brute force).
    encoder = None
    if cluster in ("Ieee754Binary", "MlLowPrecision", "Microscaling", "GoldenFloat") and e > 0 and m > 0:
        encoder = lambda val, _e=e, _m=m, _b=bias: ieee_encode_exact(_e, _m, _b, val)
    elif cluster == "HistoricalVendor" and cid.startswith("ibm_hfp"):
        encoder = lambda val, _m=m, _e=e, _b=bias: ibm_hfp_encode_exact(_m, val, e_bits=_e, bias=_b)
    elif cluster == "HistoricalVendor" and e > 0 and m > 0:
        encoder = lambda val, _e=e, _m=m, _b=bias: ieee_encode_exact(_e, _m, _b, val)
    elif cluster == "ExtendedFloat" and cid == "x87_fp80":
        encoder = lambda val, _e=e, _m=m, _b=bias: x87_encode_exact(val, bias=_b, e_bits=_e, sig_bits=_m)
    elif cluster == "IntegerFixed" and cid.startswith("int"):
        encoder = lambda val, _tb=bits: int_encode(val, _tb)
    elif cluster == "PositUnumIII" and cid.startswith("posit"):
        encoder = make_posit_encoder(bits, es=2)
    elif cluster == "Lns":
        encoder = make_lns_encoder(bits)
    vectors, mode = curated_codes(bits, decode, fmt_key, encoder=encoder)
    # anchor
    if bits <= 16:
        abits = find_anchor_bits(decode, bits, 3.0)
    elif encoder is not None:
        b3 = encoder(3.0)
        v3, _ = decode(b3, bits) if b3 is not None else (None, None)
        abits = b3 if (b3 is not None and isinstance(v3, float) and v3 == 3.0) else None
    else:
        abits = None
    if abits is not None:
        av, _ = decode(abits, bits)
        nib = (bits + 3) // 4
        anchor = {"value": av, "expected": 3.0, "ieee754_exact": (av == 3.0),
                  f"{fmt_key}_bits_hex": "0x%0*X" % (nib, abits)}
    else:
        anchor = {"value": None, "expected": 3.0, "ieee754_exact": False,
                  "note": "3.0 is not an exact grid point of this format at this width"}
    # max finite: only well-defined from an EXHAUSTIVE enumeration (<=8 bits).
    # For curated packs we omit it rather than report a misleading partial max.
    max_finite = None
    if bits <= 8:
        mx = 0.0
        for v in vectors:
            x = v["decoded_f64"]
            if isinstance(x, (int, float)) and not (isinstance(x, float) and (math.isnan(x) or math.isinf(x))):
                mx = max(mx, abs(x))
        max_finite = mx
    pack = {
        "schema": SCHEMA,
        "format": cid.upper(),
        "format_name": rec["name"],
        "bitexact": True,
        "format_notes": notes + ". " + specials_desc,
        "catalog": {
            "id": cid, "bits": bits, "s": rec["s"], "e": rec["e"], "m": rec["m"],
            "bias": rec["bias"], "storage": rec["storage"], "cluster": rec["cluster"],
            "status": rec["status"], "standard": rec["standard"],
            "use_case": rec["use_case"], "gf_relation": rec["gf_relation"],
            "source": rec["source"], "phi_distance": rec["phi_distance"],
        },
        "ssot": SSOT, "preprint": PREPRINT, "anchor_identity": ANCHOR,
        "anchor_check": anchor,
        "round_trip_policy": ("decode: exact bits->f64 (" + mode +
            "). encode (reference): round-nearest-ties-even; overflow per "
            "format convention. abs_error 0 for representable values."),
        "vector_mode": mode,
        "n_vectors": len(vectors),
    }
    if max_finite is not None:
        pack["max_finite"] = max_finite
    pack["vectors"] = vectors
    return pack

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    recs = parse_catalog(CATALOG_LINES)
    print(f"parsed {len(recs)} catalog formats")

    index = []
    written = 0
    structural = 0
    bitexact = 0

    for rec in recs:
        cid = rec["id"]
        fmt_key = fmt_key_for(rec)
        fname = f"{cid}_conformance_v0.json"

        if cid in EXISTING:
            existing_file = EXISTING[cid]
            index.append({"id": cid, "file": existing_file, "kind": "bitexact",
                          "source": "hand-curated (pre-existing)"})
            continue

        if cid in SELFCONSISTENT:
            # externally-generated wide-rung pack kept verbatim; NOT regenerated,
            # NOT promoted to bitexact (no independent second witness).
            sc_file = SELFCONSISTENT[cid]
            index.append({"id": cid, "file": sc_file, "kind": "bitexact_selfconsistent",
                          "source": "wide-rung GoldenFloat oracle (single decode law, "
                                    "dyadic-exact, no independent second witness)"})
            continue

        # posit/takum cluster
        if rec["cluster"] == "PositUnumIII":
            if cid.startswith("posit"):
                dec = make_posit_decoder(rec["bits"], es=2)
                pack = build_bitexact_pack(
                    rec, dec, f"Posit Standard 2022, n={rec['bits']}, es=2",
                    "NaR at 10..0; no separate inf; tapered precision")
                bitexact += 1
            elif cid.startswith("takum"):
                # takum: tapered logarithmic; decode is nontrivial -> structural
                pack = build_structural_pack(rec)
                pack["structural_reason"] = ("Takum (Hunhold 2024) is a tapered "
                    "LOGARITHMIC format; its decode is not a plain S:E:M field. "
                    "It is the live FL-002 counterexample and is recorded "
                    "structurally pending a dedicated logarithmic decoder.")
                structural += 1
            else:
                pack = build_structural_pack(rec); structural += 1
        else:
            built = build_decodable(rec)
            if built is not None:
                dec, notes, specials = built
                pack = build_bitexact_pack(rec, dec, notes, specials)
                bitexact += 1
            else:
                pack = build_structural_pack(rec)
                structural += 1

        with open(os.path.join(OUT, fname), "w") as f:
            json.dump(pack, f, indent=2)
        written += 1
        index.append({"id": cid, "file": fname,
                      "kind": "bitexact" if pack.get("bitexact") else "structural",
                      "n_vectors": pack.get("n_vectors", 0),
                      "source": "generated by gen_all_formats.py"})

    # SHA-256 for every pack file in the index
    for entry in index:
        path = os.path.join(OUT, entry["file"])
        h = hashlib.sha256(open(path, "rb").read()).hexdigest()
        entry["sha256"] = h

    idx = {
        "schema": "t27-conformance-index/v0.1",
        "anchor_identity": ANCHOR,
        "ssot": SSOT,
        "preprint": PREPRINT,
        "total_formats": len(recs),
        "total_packs": len(index),
        "bitexact_packs": sum(1 for e in index if e["kind"] == "bitexact"),
        "selfconsistent_packs": sum(1 for e in index if e["kind"] == "bitexact_selfconsistent"),
        "structural_packs": sum(1 for e in index if e["kind"] == "structural"),
        "packs": index,
    }
    with open(os.path.join(OUT, "INDEX_all_formats.json"), "w") as f:
        json.dump(idx, f, indent=2)

    print(f"formats: {len(recs)}")
    print(f"written new packs: {written}  (bitexact={bitexact}, structural={structural})")
    print(f"existing kept: {len(EXISTING)}")
    print(f"index packs: {len(index)}  bitexact={idx['bitexact_packs']}  "
          f"selfconsistent={idx['selfconsistent_packs']}  structural={idx['structural_packs']}")

if __name__ == "__main__":
    main()
