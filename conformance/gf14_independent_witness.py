#!/usr/bin/env python3
"""
Независимый второй свидетель для GF14 (строгий SW-bitexact).

НЕ переиспользует generator/encoder. Декодер написан с нуля по параметрам
каталога SSOT (specs/numeric/gf14.t27 -> bits=14 s=1 e=5 m=8 bias=15 storage=u16),
с точной рациональной арифметикой (fractions.Fraction) -> abs_error == 0.

Соответствие honesty-rules:
- фиксированный детерминированный bit-layout: [sign:1][exp:5][mant:8] (LSB-aligned в u16);
- второй независимый закон decode (этот файл) против вектор-значений из пака;
- сравнение точное (Fraction), не float-приближение.

Запуск: python3 gf14_independent_witness.py conformance/vectors/gf14_conformance_v0.json
Выход: 0 = все 14 совпали abs_error=0; 1 = расхождение.
"""
import json
import sys
from fractions import Fraction

# --- параметры GF14 из каталога SSOT (НЕ из generator) ---
EXP_BITS = 5
MANT_BITS = 8
BIAS = 15
EXP_MAX = (1 << EXP_BITS) - 1          # 31 = all-ones -> Inf/NaN
MANT_MAX = (1 << MANT_BITS) - 1        # 255
TOTAL_BITS = 1 + EXP_BITS + MANT_BITS  # 14


def decode_exact(raw):
    """Независимый decode -> Fraction (или строка для Inf/NaN/zero-sign)."""
    raw &= (1 << TOTAL_BITS) - 1               # маска 14 значащих бит
    sign = (raw >> (EXP_BITS + MANT_BITS)) & 1
    exp = (raw >> MANT_BITS) & EXP_MAX
    mant = raw & MANT_MAX

    if exp == EXP_MAX:                          # спец-класс
        if mant == 0:
            return "INF(-)" if sign else "INF(+)"
        return "NAN(-)" if sign else "NAN(+)"

    if exp == 0:                                # zero / subnormal
        if mant == 0:
            return "ZERO(-)" if sign else "ZERO(+)"
        # subnormal: mant/2^m * 2^(1-bias)
        val = Fraction(mant, 1 << MANT_BITS) * Fraction(2) ** (1 - BIAS)
    else:                                       # normal: (1 + mant/2^m) * 2^(exp-bias)
        val = (1 + Fraction(mant, 1 << MANT_BITS)) * Fraction(2) ** (exp - BIAS)

    return -val if sign else val


def parse_expected(value_str):
    """Ожидаемое значение из пака -> Fraction или маркер-строка."""
    s = value_str.strip()
    if s in ("INF(+)", "INF(-)", "NAN(+)", "NAN(-)"):
        return s
    if s in ("0", "-0", "+0"):
        return Fraction(0)
    # десятичная дробь -> точная Fraction через строку
    return Fraction(s)


def main(path):
    pack = json.load(open(path))
    vecs = pack["vectors"]
    assert pack["format"].upper().startswith("GF14"), "не GF14-пак"
    ok = 0
    fails = []
    for v in vecs:
        raw = int(v["hex"], 16)
        # перекрёстная проверка bits == hex
        if v.get("bits") is not None and v["bits"] != raw:
            fails.append((v["label"], f"bits!=hex: {v['bits']} vs {raw}"))
            continue
        got = decode_exact(raw)
        exp = parse_expected(v["value"])

        if isinstance(exp, str):                # спец-класс: сравниваем по классу/знаку
            # zero: пак пишет "0" -> допускаем оба знака нуля
            if exp in ("INF(+)", "INF(-)", "NAN(+)", "NAN(-)"):
                match = (got == exp)
            else:
                match = isinstance(got, str) and got.startswith("ZERO")
        else:
            if isinstance(got, str):            # got спец, exp число
                match = (exp == 0 and got.startswith("ZERO"))
            else:
                match = (Fraction(got) == exp)  # ТОЧНОЕ равенство, abs_error=0

        if match:
            ok += 1
        else:
            fails.append((v["label"], f"got={got} exp={exp}"))

    print(f"GF14 independent witness: {ok}/{len(vecs)} bit-exact (abs_error=0)")
    if fails:
        print("FAILS:")
        for lbl, msg in fails:
            print(f"  {lbl}: {msg}")
        return 1
    print("VERDICT: gf14 promotable selfconsistent -> strict bitexact (2nd independent witness OK)")
    return 0


if __name__ == "__main__":
    p = sys.argv[1] if len(sys.argv) > 1 else "conformance/vectors/gf14_conformance_v0.json"
    sys.exit(main(p))
