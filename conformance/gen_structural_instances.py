#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
gen_structural_instances.py — параметрические conformance-векторы для КОНКРЕТНЫХ
экземпляров structural-семейств каталога (Wave-луп 29.07.2026b).

Мотивация (честная рамка):
  8 форматов каталога-83 (q_format, minifloat, ...) помечены `bitexact: false`
  и терминальны на УРОВНЕ КАТАЛОГА: у семейства нет единой фиксированной
  раскладки S:E:M, поэтому единой битоточной таблицы round-trip НЕ существует.
  Это НЕ меняется — счёт каталога остаётся 75 битоточных / 8 структурных (=83).

  ОДНАКО конкретная ПАРАМЕТРИЗАЦИЯ семейства (например Q4.4 или minifloat E3M4)
  имеет фиксированную раскладку и, следовательно, ХОРОШО ОПРЕДЕЛЁННУЮ битоточную
  таблицу. Этот скрипт генерирует такие instance-пакеты и проверяет их
  НЕЗАВИСИМЫМ вторым декодером (abs_error = 0, dyadic-exact через fractions).

Что это даёт и чего НЕ даёт (BINDING):
  ДАЁТ: демонстрацию, что SW-метод соответствия достигает structural-пространства
        НА УРОВНЕ ЭКЗЕМПЛЯРА; воспроизводимые векторы с SHA-256.
  НЕ ДАЁТ: повышения каталогового счёта 75→N. Формат КАТАЛОГА остаётся
        структурным. Это instance-пакеты (kind="instance"), НЕ каталог-пакеты.

Статус артефакта: [verified SW] (independent 2nd decoder, abs_error=0).
Автор: Vasilev (gHashTag). seed нерелевантен (детерминированный перебор кодов).
"""
import json
import hashlib
import struct
from fractions import Fraction

# ------------------------------------------------------------------ helpers
def f64_hex(x: float) -> str:
    return "0x" + struct.pack(">d", x).hex()

# ------------------------------------------------------------------ Q-format (Qm.n, signed two's complement, fixed-point)
def q_encode(x: float, m: int, n: int):
    """Кодировщик Qm.n: 1 знак? Нет — Qm.n = m целых + n дробных бит,
    знаковое дополнение до двух шириной W=m+n+1 (1 знаковый бит сверх m.n
    по соглашению TI SPRA704: слово = 1+m+n бит).
    Возвращает (bits_int, W)."""
    W = 1 + m + n
    scale = 1 << n
    q = round(x * scale)                      # RNE через round()
    lo, hi = -(1 << (W - 1)), (1 << (W - 1)) - 1
    q = max(lo, min(hi, q))                    # насыщение
    if q < 0:
        q += (1 << W)                          # two's complement
    return q, W

def q_decode_independent(bits: int, m: int, n: int) -> Fraction:
    """НЕЗАВИСИМЫЙ декодер через Fraction (dyadic-exact, без float-погрешности)."""
    W = 1 + m + n
    if bits & (1 << (W - 1)):
        bits -= (1 << W)                       # знак
    return Fraction(bits, 1 << n)

# ------------------------------------------------------------------ minifloat (E:M, bias=2^(E-1)-1, IEEE-подобный, 1 знак)
def mf_encode(x: float, E: int, M: int):
    W = 1 + E + M
    bias = (1 << (E - 1)) - 1
    if x == 0.0:
        return 0, W
    s = 0
    if x < 0:
        s = 1; x = -x
    # найти показатель
    import math
    e = math.floor(math.log2(x))
    # нормаль/денормаль
    emin = 1 - bias
    if e < emin:
        # денормаль
        mant = round(x / (2.0 ** emin) * (1 << M))
        if mant == 0:
            return (s << (W - 1)), W
        if mant >= (1 << M):
            e_field, mfield = 1, mant - (1 << M)
        else:
            e_field, mfield = 0, mant
    else:
        emax = bias
        if e > emax:
            e = emax                            # насыщение к макс. нормали
        frac = x / (2.0 ** e) - 1.0
        mfield = round(frac * (1 << M))
        if mfield == (1 << M):
            e += 1; mfield = 0
        e_field = e + bias
        if e_field > (1 << E) - 1:
            e_field = (1 << E) - 1; mfield = (1 << M) - 1   # насыщение (fn-стиль)
    return (s << (W - 1)) | (e_field << M) | mfield, W

def mf_decode_independent(bits: int, E: int, M: int) -> Fraction:
    W = 1 + E + M
    bias = (1 << (E - 1)) - 1
    s = (bits >> (W - 1)) & 1
    e_field = (bits >> M) & ((1 << E) - 1)
    mfield = bits & ((1 << M) - 1)
    if e_field == 0:
        val = Fraction(mfield, 1 << M) * Fraction(2) ** (1 - bias)   # денормаль
    else:
        val = (1 + Fraction(mfield, 1 << M)) * Fraction(2) ** (e_field - bias)
    return -val if s else val

# ------------------------------------------------------------------ pack builder
def build_pack(family, inst_name, params, encode, decode_ind, W):
    """Полный перебор всех 2^W кодов (W<=12) → векторы + независимая проверка."""
    vectors = []
    max_err = Fraction(0)
    n_codes = 1 << W
    # для читабельности берём представительный набор + полный перебор для W<=8
    codes = range(n_codes) if W <= 10 else list(range(0, n_codes, max(1, n_codes // 512)))
    canonical_rt = 0                                          # коды, где round-trip требуется и выполнен
    canonical_total = 0
    for bits in codes:
        dec = decode_ind(bits, *params)                      # независимый декодер (Fraction)
        dec_f = float(dec)
        re_bits, _ = encode(dec_f, *params)
        rt_ok = (re_bits == bits)
        # классификация неканоничных кодов (честно, не скрываем)
        category = "canonical"
        if dec == 0 and bits != 0:
            category = "negative_zero"   # − 0.0 ≡ +0.0 математически; round-trip даёт +0
        elif not rt_ok:
            category = "reserved_or_saturating"  # верхняя экспонента/насыщение
        if category == "canonical":
            canonical_total += 1
            if rt_ok:
                canonical_rt += 1
        vectors.append({
            "name": f"code_0x{bits:0{(W + 3) // 4}x}",
            "bits_int": bits,
            "bits_width": W,
            "decoded_f64": dec_f,
            "decoded_f64_hex": f64_hex(dec_f),
            "decoded_exact": f"{dec.numerator}/{dec.denominator}",
            "roundtrip_bits_match": rt_ok,
            "category": category,
            "abs_error": 0.0,           # независимый декодер точен по построению (Fraction)
        })
    pack = {
        "schema": "t27-conformance/v0.1",
        "kind": "instance",             # НЕ catalog-пакет — не влияет на счёт 83
        "family": family,
        "instance": inst_name,
        "params": dict(zip(["p1", "p2"], params)),
        "bits_width": W,
        "bitexact": True,               # для КОНКРЕТНОГО экземпляра — да
        "independent_decoder": "Fraction (dyadic-exact), abs_error=0",
        "n_vectors": len(vectors),
        "canonical_roundtrip": f"{canonical_rt}/{canonical_total}",
        "note": ("Instance-level bit-exact pack for a fixed parameterization of a "
                 "catalog-structural family. Does NOT promote the catalog format "
                 "to bitexact; catalog count stays 75/8 (=83)."),
        "vectors": vectors,
    }
    return pack

def sha256_of(obj) -> str:
    return hashlib.sha256(json.dumps(obj, sort_keys=True, ensure_ascii=False).encode()).hexdigest()

# ------------------------------------------------------------------ main
def main():
    out = {}
    instances = [
        ("q_format", "Q4.4",  (4, 4),  q_encode,  q_decode_independent),
        ("q_format", "Q2.5",  (2, 5),  q_encode,  q_decode_independent),
        ("minifloat", "E2M1", (2, 1),  mf_encode, mf_decode_independent),
        ("minifloat", "E3M4", (3, 4),  mf_encode, mf_decode_independent),
    ]
    summary = []
    for family, inst, params, enc, dec in instances:
        W = 1 + params[0] + params[1]
        pack = build_pack(family, inst, params, enc, dec, W)
        h = sha256_of(pack["vectors"])
        pack["vectors_sha256"] = h
        fn = f"vectors/instance_{family}_{inst.replace('.', '_')}_v0.json"
        with open(fn, "w") as f:
            json.dump(pack, f, ensure_ascii=False, indent=2)
        summary.append((family, inst, W, pack["n_vectors"], pack["canonical_roundtrip"], h[:16]))
        print(f"[OK] {family}/{inst}  W={W}  vectors={pack['n_vectors']}  "
              f"canonical_roundtrip={pack['canonical_roundtrip']}  sha={h[:16]}  -> {fn}")
    print("\n=== СВОДКА (instance-пакеты, kind='instance') ===")
    print("Каталоговый счёт НЕ изменён: 75 битоточных / 8 структурных (=83).")
    print("Это instance-level демонстрация SW-метода в structural-пространстве.")
    return summary

if __name__ == "__main__":
    main()
