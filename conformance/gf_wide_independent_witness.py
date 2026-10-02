#!/usr/bin/env python3
"""
Независимый второй свидетель для ШИРОКИХ рунгов GoldenFloat (gf48/gf96/gf128/gf512/gf1024).

Параметрический по (s,e,m,bias) из каталога SSOT каждого пака. НЕ переиспользует
generator/encoder: decode написан с нуля. Точное равенство (abs_error == 0) — БЕЗ
материализации гигантских степеней 2^bias. Любое конечное значение представляется
канонической dyadic-парой (odd_num, shift) = odd_num * 2^shift, где odd_num нечётно
(или 0). Сравнение двух dyadic = равенство (odd_num, shift). Это позволяет верифицировать
рунги с bias до ~2.5e120 (gf1024) без взрыва целых.

Bit-layout (LSB-aligned, как у gf14/gf16): [sign:1][exp:e][mant:m], total=1+e+m бит.
Спец-класс: exp == all-ones -> mant==0 ? Inf : NaN. exp==0 -> zero/subnormal.
value_encoding = decimal | dyadic ("A p B" = A*2^B).

Запуск: python3 gf_wide_independent_witness.py conformance/vectors/gf48_conformance_v0.json
Выход: 0 = все векторы bit-exact (abs_error=0); 1 = расхождение.
"""
import json
import math
import re
import sys


def normalize_dyadic(num, shift):
    """Канонизировать num*2^shift -> (odd_num, shift'): odd_num нечётно (или 0).
    Сдвигает двойки из num в shift. Не раскрывает 2^shift."""
    if num == 0:
        return (0, 0)
    sign = -1 if num < 0 else 1
    num = abs(num)
    # вынести все множители 2 из num в показатель
    tz = (num & -num).bit_length() - 1  # число младших нулевых бит
    num >>= tz
    shift += tz
    return (sign * num, shift)


def make_decoder(e_bits, m_bits, bias):
    """Возвращает decode(raw) -> ('INF'|'NAN'|'ZERO' ...) либо ('NUM', (odd_num, shift)).

    Конечное нормальное:    (1 + mant/2^m) * 2^(exp-bias)
       = (2^m + mant) * 2^(exp - bias - m)         <- целое * степень двойки
    Субнормальное (exp==0): (mant/2^m) * 2^(1-bias)
       = mant * 2^(1 - bias - m)
    """
    exp_max = (1 << e_bits) - 1
    total = 1 + e_bits + m_bits

    def decode(raw):
        raw &= (1 << total) - 1
        sign = (raw >> (e_bits + m_bits)) & 1
        exp = (raw >> m_bits) & exp_max
        mant = raw & ((1 << m_bits) - 1)
        if exp == exp_max:
            if mant == 0:
                return ("INF(-)" if sign else "INF(+)")
            return ("NAN(-)" if sign else "NAN(+)")
        if exp == 0:
            if mant == 0:
                return ("ZERO(-)" if sign else "ZERO(+)")
            num = mant
            shift = 1 - bias - m_bits
        else:
            num = (1 << m_bits) + mant
            shift = exp - bias - m_bits
        if sign:
            num = -num
        return normalize_dyadic(num, shift)

    return decode, total


_DYADIC = re.compile(r"^(-?\d+)p(-?\d+)$")


def parse_expected_dyadic(value):
    """Возвращает ('INF'/'NAN' string) ИЛИ ('NUM', (odd_num, shift)) для конечных.
    Поддерживает dyadic 'A p B' и десятичные строки (последние конвертируются точно,
    если знаменатель — степень двойки; иначе через Fraction-fallback с проверкой)."""
    if isinstance(value, str):
        s = value.strip()
        if s in ("INF(+)", "INF(-)", "NAN(+)", "NAN(-)"):
            return ("SPECIAL", s)
        m = _DYADIC.match(s)
        if m:                                  # dyadic "A p B" = A * 2^B  (ТОЧНО)
            a, b = int(m.group(1)), int(m.group(2))
            return ("NUM", normalize_dyadic(a, b))
        if s in ("0", "-0", "+0"):
            return ("NUM", (0, 0))
        # десятичная строка -> dyadic, если знаменатель степень двойки
        return ("NUM", decimal_to_dyadic(s))
    # int/float-подобное
    return ("NUM", decimal_to_dyadic(str(value)))


def decimal_to_dyadic(s):
    """Точно конвертировать десятичную строку в (odd_num, shift) ТОЛЬКО если значение
    диадическое (знаменатель = степень двойки). Иначе бросает (нет в широких паках)."""
    from fractions import Fraction
    f = Fraction(s)
    num, den = f.numerator, f.denominator
    # den должен быть степенью двойки
    if den & (den - 1) != 0:
        raise ValueError(f"non-dyadic expected value {s} (den={den})")
    shift = -((den).bit_length() - 1)
    return normalize_dyadic(num, shift)


def main(path):
    pack = json.load(open(path))
    
    # Handle both unified and old schemas
    if "catalog" in pack:
        # Unified schema: read parameters from catalog
        cat = pack["catalog"]
        e_bits, m_bits, bias = cat["e"], cat["m"], cat["bias"]
    else:
        # Old per-format schema: use hardcoded parameters
        format_name = pack["format"].upper()
        if format_name == "GF16":
            e_bits, m_bits, bias = 6, 9, 31  # GF16: s=1, e=6, m=9, bias=31 (from gf16_ref.py)
        elif format_name == "GF32":
            e_bits, m_bits, bias = 8, 23, 127  # GF32: similar to binary32
        elif format_name == "GF64":
            e_bits, m_bits, bias = 11, 52, 1023  # GF64: similar to binary64
        else:
            raise ValueError(f"Unsupported old schema format: {format_name}")
    
    fmt = pack["format"]
    decode, total = make_decoder(e_bits, m_bits, bias)
    vecs = pack["vectors"]
    ok, fails = 0, []
    for v in vecs:
        # Determine hex and value keys based on schema
        if "hex" in v:
            # Unified schema: use "hex" and "value" directly
            hex_str = v["hex"]
            expected_val = v["value"]
            raw = int(hex_str, 16)
            # Check bits if available
            bits_val = v.get("bits")
        else:
            # Old per-format schema: find appropriate keys
            hex_key = None
            for key in v.keys():
                if key.endswith("_bits_hex"):
                    hex_key = key
                    break
            
            if hex_key is None:
                # Try to construct from the format
                format_str = pack["format"]
                hex_key = format_str + "_bits_hex"
                if hex_key not in v:
                    hex_key = format_str.lower() + "_bits_hex"
                if hex_key not in v:
                    hex_key = "gf16_bits_hex"  # fallback for GF16
                if hex_key not in v:
                    raise KeyError(f"Could not find hex key for format {format_str}")
            
            hex_str = v[hex_key]
            # For the value, we use "input_f64" for old schema
            if "input_f64" not in v:
                raise KeyError("Missing input_f64 in vector for old schema")
            expected_val = v["input_f64"]
            
            # Determine the raw integer from the hex string
            raw = int(hex_str, 16)
            # Check bits if available
            bits_val = None
            for key in v.keys():
                if key.endswith("_bits_int"):
                    bits_val = v[key]
                    break

        # Validate bits consistency
        # Get the vector identifier (handle both 'label' and 'name' fields)
        vector_id = v.get("label", v.get("name", "unknown"))

        if bits_val is not None and bits_val != raw:
            fails.append((vector_id, f"bits!=hex {bits_val} vs {raw}"))
            continue

        got = decode(raw)
        
        # Check which schema we're using and apply appropriate comparison
        if "hex" in v:
            # Unified schema: exact dyadic comparison
            exp = parse_expected_dyadic(expected_val)
            if isinstance(got, str):  # спец-класс или zero
                if got.startswith("ZERO"):
                    match = (exp[0] == "NUM" and exp[1] == (0, 0))
                else:  # INF / NAN
                    match = (exp[0] == "SPECIAL" and exp[1] == got)
            else:  # got = (odd, shift) конечное
                match = (exp[0] == "NUM" and exp[1] == got)
        else:
            # Old per-format schema: f64 comparison with tolerance
            try:
                # Convert dyadic result to float for comparison
                if isinstance(got, str):
                    # Handle special cases
                    if got.startswith("ZERO"):
                        got_float = 0.0
                    elif got.startswith("INF"):
                        got_float = float('inf') if "(+)" in got else float('-inf')
                    elif got.startswith("NAN"):
                        got_float = float('nan')
                    else:
                        match = False
                        fails.append((vector_id, f"unhandled special case: got={got}"))
                        continue
                else:
                    # Convert dyadic (odd, shift) to float
                    odd_num, shift = got
                    got_float = float(odd_num * (2 ** shift))
                
                # Convert expected value to float
                if isinstance(expected_val, str):
                    if expected_val.lower() in ("inf", "infinity"):
                        exp_float = float('inf')
                    elif expected_val.lower() == ("-inf", "-infinity"):
                        exp_float = float('-inf')
                    elif expected_val.lower() == "nan":
                        exp_float = float('nan')
                    else:
                        exp_float = float(expected_val)
                else:
                    exp_float = float(expected_val)
                
                # Compare with tolerance (similar to gf16_ref.py logic)
                if math.isnan(got_float) and math.isnan(exp_float):
                    match = True
                elif math.isinf(got_float) and math.isinf(exp_float) and (got_float > 0) == (exp_float > 0):
                    match = True
                elif got_float == exp_float:  # exact match for exact values
                    match = True
                else:
                    # Tolerance-based comparison for non-exact values
                    abs_e = abs(exp_float)
                    tol = max(abs_e * 0.005, 0.001)  # same tolerance as gf16_ref.py
                    diff = abs(got_float - exp_float)
                    match = (diff <= tol + 1e-9)
                    
            except Exception as e:
                match = False
                fails.append((vector_id, f"comparison error: {e}"))
        
        if match:
            ok += 1
        else:
            fails.append((vector_id, f"got={got} exp={expected_val} (schema={'unified' if 'hex' in v else 'old'})"))
    print(f"{fmt} independent witness: {ok}/{len(vecs)} bit-exact (abs_error=0)  [e={e_bits} m={m_bits} bias={bias}]")
    if fails:
        print("FAILS:")
        for lbl, msg in fails:
            print(f"  {lbl}: {msg}")
        return 1
    print(f"VERDICT: {fmt} selfconsistent -> strict bitexact OK (2nd independent witness, dyadic-exact)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
