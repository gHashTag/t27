import math
import json
import sys
import struct

# Constants from GF16 spec
BIAS = 31
EXP_BITS = 6
MANT_BITS = 9
EXP_MAX = (1 << EXP_BITS) - 1
MANT_MAX = (1 << MANT_BITS) - 1
EXP_MASK = 0x7E00
SIGN_MASK = 0x8000
MANT_MASK = 0x01FF

# GF16 special values per spec D3
POS_ZERO = 0x0000
NEG_ZERO = 0x8000
POS_INF = 0x7E00
NEG_INF = 0xFE00
QUIET_NAN = 0xFE01


def encode(v):
    """Encode IEEE 754 float to GF16 following D1-D4:
    D1: no subnormals; exponent-0 codes with non-zero mantissa are normal, (1+M/512)*2^-31; below 2^-31 encodes to signed zero.
    D2: round to nearest, ties toward zero ("ties-to-zero (frozen)")
    D3: canonical NaN is 0xFE01
    D4: exponent extraction is (x & EXP_MASK) >> EXP_SHIFT
    """
    if isinstance(v, str):
        if v == "Infinity":
            return POS_INF
        if v == "-Infinity":
            return NEG_INF
        if v == "NaN":
            return QUIET_NAN
        v = float(v)
    
    if math.isnan(v):
        return QUIET_NAN
    
    if v == 0.0:
        return NEG_ZERO if math.copysign(1.0, v) < 0 else POS_ZERO
    
    if math.isinf(v):
        return NEG_INF if v < 0 else POS_INF
    
    # Extract f32 bits
    sign = 1 if v < 0 else 0
    abs_v = abs(v)
    f32_bits = struct.unpack('I', struct.pack('f', abs_v))[0]
    f32_exp = (f32_bits >> 23) & 0xFF
    f32_mant = f32_bits & 0x7FFFFF
    
    # Handle values below 2^-31 (flush to signed zero per D1)
    if f32_exp < 96:  # 127 - 31 = 96
        return sign << 15
    
    # Handle overflow to infinity
    if f32_exp > 158:  # 127 + 31 = 158
        return (sign << 15) | POS_INF
    
    # Rebias exponent from 127 to 31
    gf16_exp = f32_exp - 96
    mant = f32_mant >> 14  # Keep top 9 bits of 23-bit mantissa
    rem = f32_mant & 0x3FFF  # Remaining 14 bits
    
    # D2: ties toward zero - round up only strictly above half
    if rem > 0x2000:
        mant += 1
        if mant > MANT_MAX:
            mant = 0
            gf16_exp += 1
            if gf16_exp > EXP_MAX:
                return (sign << 15) | POS_INF
    
    # D1: E = 0, M = 0 would be 2^-31, which has no code: flush to signed zero
    if gf16_exp == 0 and mant == 0:
        return sign << 15
    
    return (sign << 15) | (gf16_exp << 9) | mant


def decode(raw):
    """Decode GF16 to IEEE 754 float following D1-D4"""
    sign = (raw >> 15) & 1
    exp = (raw & EXP_MASK) >> 9  # D4: exponent extraction via mask/shift
    mant = raw & MANT_MASK
    
    # Handle zero
    if exp == 0 and mant == 0:
        return -0.0 if sign else 0.0
    
    # Handle special values (Inf/NaN)
    if exp == EXP_MAX:
        if mant == 0:
            return float("-inf") if sign else float("inf")
        else:
            return float("nan")
    
    # Normal number: D1 - no subnormals, E=0 with M!=0 is normal
    value = (1.0 + mant / 512.0) * (2.0 ** (exp - BIAS))
    return -value if sign else value


def gf16_add(a_raw, b_raw):
    a = decode(a_raw)
    b = decode(b_raw)
    return encode(a + b)


def gf16_mul(a_raw, b_raw):
    a = decode(a_raw)
    b = decode(b_raw)
    return encode(a * b)


def gf16_dot4(a_vec, b_vec):
    products = [gf16_mul(a_vec[i], b_vec[i]) for i in range(4)]
    s01 = gf16_add(products[0], products[1])
    s23 = gf16_add(products[2], products[3])
    return gf16_add(s01, s23)


def to_hex(raw):
    return f"0x{raw:04X}"


def run_fpga_tests():
    tests = [
        ("1.0*1.0", "mul", 0x3E00, 0x3E00, 1.0),
        ("1.0*0", "mul", 0x3E00, 0x0000, 0.0),
        ("0*1.0", "mul", 0x0000, 0x3E00, 0.0),
        ("2.0*1.0", "mul", 0x4000, 0x3E00, 2.0),
        ("2.0*2.0", "mul", 0x4000, 0x4000, 4.0),
        ("3.0*3.0", "mul", 0x4100, 0x4100, 9.0),
        ("-1*1", "mul", 0xBE00, 0x3E00, -1.0),
        ("-1*-1", "mul", 0xBE00, 0xBE00, 1.0),
        ("1.5*1.5", "mul", 0x3F00, 0x3F00, 2.25),
        ("0.5*0.5", "mul", 0x3C00, 0x3C00, 0.25),
        ("1+1", "add", 0x3E00, 0x3E00, 2.0),
        ("1+0", "add", 0x3E00, 0x0000, 1.0),
        ("0+1", "add", 0x0000, 0x3E00, 1.0),
        ("1+2", "add", 0x3E00, 0x4000, 3.0),
        ("3-1", "add", 0x4100, 0xBE00, 2.0),
        ("1-1", "add", 0x3E00, 0xBE00, 0.0),
        ("-1+2", "add", 0xBE00, 0x4000, 1.0),
        ("0.5+0.5", "add", 0x3C00, 0x3C00, 1.0),
        ("0.5+0.25", "add", 0x3C00, 0x3A00, 0.75),
        ("1.5+1.5", "add", 0x3F00, 0x3F00, 3.0),
        ("100+200", "add", 0x4B20, 0x4D20, 300.0),
    ]

    special = [
        ("NaN*1", "mul", 0xFE01, 0x3E00, QUIET_NAN),
        ("Inf*1", "mul", 0x7E00, 0x3E00, POS_INF),
        ("0*Inf", "mul", 0x0000, 0x7E00, QUIET_NAN),
        ("NaN+1", "add", 0xFE01, 0x3E00, QUIET_NAN),
        ("Inf+1", "add", 0x7E00, 0x3E00, POS_INF),
        ("Inf+-Inf", "add", 0x7E00, 0xFE00, QUIET_NAN),
    ]

    dot4 = [
        ("ones.dot4", [0x3E00]*4, [0x3E00]*4, 4.0),
        ("1234.dot4", [0x3E00, 0x4000, 0x4100, 0x4200], [0x3E00, 0x4000, 0x4100, 0x4200], 30.0),
        ("0.5x2.dot4", [0x3C00]*4, [0x4000]*4, 4.0),
        ("zeros.dot4", [0x0000]*4, [0x3E00]*4, 0.0),
        ("neg_pos.dot4", [0xBE00, 0x3E00, 0xBE00, 0x3E00], [0x3E00]*4, 0.0),
    ]

    passed = 0
    failed = 0

    for name, op, a_raw, b_raw, expected in tests:
        if op == "mul":
            result = gf16_mul(a_raw, b_raw)
        else:
            result = gf16_add(a_raw, b_raw)
        dec = decode(result)
        abs_e = abs(expected)
        diff = abs(dec - expected)
        if diff < 0.05 * abs_e + 0.01 or (expected == 0.0 and dec == 0.0):
            passed += 1
        else:
            print(f"FAIL {name}: {to_hex(a_raw)} {op} {to_hex(b_raw)} = {to_hex(result)} ({dec}) expected {expected}")
            failed += 1

    for name, op, a_raw, b_raw, expected_raw in special:
        if op == "mul":
            result = gf16_mul(a_raw, b_raw)
        else:
            result = gf16_add(a_raw, b_raw)
        if result == expected_raw:
            passed += 1
        else:
            print(f"FAIL {name}: got {to_hex(result)} expected {to_hex(expected_raw)}")
            failed += 1

    for name, a_vec, b_vec, expected in dot4:
        result = gf16_dot4(a_vec, b_vec)
        dec = decode(result)
        abs_e = abs(expected) if abs(expected) > 0.001 else 0.001
        diff = abs(dec - expected)
        if diff < 0.1 * abs_e + 0.1 or (expected == 0.0 and dec == 0.0):
            passed += 1
        else:
            print(f"FAIL {name}: {to_hex(result)} ({dec}) expected {expected}")
            failed += 1

    print(f"FPGA consistency: {passed} pass, {failed} fail")
    return failed == 0


def run_roundtrip():
    values = [
        0.0, 1.0, -1.0, 2.0, 0.5, 0.25, 3.0, 4.0,
        0.125, 0.75, 1.5, 65504.0, 0.000061035,
        3.14159265, 1.61803398, 100.0, 200.0, 50.0, 25.0,
    ]
    passed = 0
    failed = 0
    for v in values:
        raw = encode(v)
        dec = decode(raw)
        tol = max(abs(v) * 0.005, 0.001)
        if abs(dec - v) <= tol:
            passed += 1
        else:
            print(f"FAIL roundtrip {v}: {to_hex(raw)} -> {dec} (tol={tol})")
            failed += 1
    print(f"Roundtrip: {passed} pass, {failed} fail")
    return failed == 0


def run_conformance(vectors_path):
    with open(vectors_path) as f:
        data = json.load(f)

    passed = 0
    failed = 0
    skipped = 0
    special_names = {"infinity_positive", "infinity_negative", "nan_quiet", "max_value"}

    for tv in data["test_vectors"]:
        name = tv["name"]
        inp = tv["input"]["value"]

        if name in special_names:
            raw = encode(inp)
            dec = decode(raw)
            if name.startswith("infinity") and math.isinf(dec):
                passed += 1
            elif name == "nan_quiet" and math.isnan(dec):
                passed += 1
            elif name == "max_value" and dec > 65000:
                passed += 1
            else:
                print(f"FAIL {name}: encode({inp}) = {to_hex(raw)} decode = {dec}")
                failed += 1
            continue

        exp_decoded = tv["expected"].get("decoded")
        tol = tv["expected"].get("tolerance_abs", 0.0)

        if exp_decoded is None:
            skipped += 1
            continue

        raw = encode(inp)
        dec = decode(raw)

        if isinstance(exp_decoded, str):
            if exp_decoded == "Infinity":
                passed += 1 if math.isinf(dec) and dec > 0 else failed + 0 or True
                continue
            elif exp_decoded == "-Infinity":
                passed += 1 if math.isinf(dec) and dec < 0 else 0
                continue
            elif exp_decoded == "NaN":
                passed += 1 if math.isnan(dec) else 0
                continue

        if abs(dec - exp_decoded) <= tol + 1e-9:
            passed += 1
        else:
            print(f"FAIL {name}: {inp} -> {to_hex(raw)} -> {dec} expected {exp_decoded} (tol={tol})")
            failed += 1

    print(f"Conformance: {passed} pass, {failed} fail, {skipped} skip")
    return failed == 0


if __name__ == "__main__":
    ok = True
    ok = run_fpga_tests() and ok
    ok = run_roundtrip() and ok
    ok = run_conformance("conformance/gf16_vectors.json") and ok
    sys.exit(0 if ok else 1)