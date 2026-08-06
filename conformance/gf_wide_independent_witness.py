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
    cat = pack["catalog"]
    e_bits, m_bits, bias = cat["e"], cat["m"], cat["bias"]
    decode, total = make_decoder(e_bits, m_bits, bias)
    vecs = pack["vectors"]
    ok, fails = 0, []
    for v in vecs:
        raw = int(v["hex"], 16)
        if v.get("bits") is not None and v["bits"] != raw:
            fails.append((v["label"], f"bits!=hex {v['bits']} vs {raw}")); continue
        got = decode(raw)
        exp = parse_expected_dyadic(v["value"])
        # got: либо строка спец-класса ("INF(+)"/"NAN(+)"/"ZERO(+)") либо tuple (odd,shift)
        if isinstance(got, str):  # спец-класс или zero
            if got.startswith("ZERO"):
                match = (exp[0] == "NUM" and exp[1] == (0, 0))
            else:  # INF / NAN
                match = (exp[0] == "SPECIAL" and exp[1] == got)
        else:  # got = (odd, shift) конечное
            match = (exp[0] == "NUM" and exp[1] == got)
        if match:
            ok += 1
        else:
            fails.append((v["label"], f"got={got} exp={exp}"))
    fmt = pack["format"]
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
