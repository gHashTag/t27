#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Свод ключевых метрик 83 числовых форматов каталога t27 (SSOT).
Источники: gen/numeric/formats_catalog.json (77) + specs/numeric/gf*.t27 (6) +
conformance/vectors/INDEX_all_formats.json (kind/n_vectors) + HW-статус сессии 28.06.2026.
ЧЕСТНОСТЬ: HW = только реально измеренное на AX7203. Покрытие тегируется.
"""
import json, math, csv, re

cat = json.load(open('/tmp/cat.json'))['formats']
idx = json.load(open('/tmp/idx.json'))['packs']
ix = {p['id']: p for p in idx}

# --- 6 GF форматов, отсутствующих в catalog.json (взяты из specs/numeric/*.t27) ---
gf_extra = {
 'gf10':  dict(bits=10,  s_bits=1, e_bits=3,  m_bits=6),
 'gf14':  dict(bits=14,  s_bits=1, e_bits=5,  m_bits=8),
 'gf48':  dict(bits=48,  s_bits=1, e_bits=18, m_bits=29),
 'gf96':  dict(bits=96,  s_bits=1, e_bits=36, m_bits=59),
 'gf512': dict(bits=512, s_bits=1, e_bits=195,m_bits=316),
 'gf1024':dict(bits=1024,s_bits=1, e_bits=391,m_bits=632),
}
for fid, g in gf_extra.items():
    g.update(id=fid, name=f"GoldenFloat {fid.upper()} ({g['bits']}-bit)",
             bias=(2**(g['e_bits']-1)-1) if g['e_bits'] else 0,
             cluster='GoldenFloat', status='Experimental', standard='Trinity GoldenFloat',
             use_case='расширенный диапазон GF-семейства', gf_relation='self',
             source='specs/numeric/'+fid+'.t27', phi_distance=None, storage=f"u{g['bits']}")

formats = {f['id']: f for f in cat}
formats.update(gf_extra)

# --- HW-статус (только измеренное на железе, сессия 28.06.2026) ---
# E = полная цепь доказательств опубликована на #199 (CI run + SHA + flash log)
# C = self-report локального агента + дизайн на main, UART-лог ещё не на #199
HW = {
 'bfloat16':    dict(decode_hw='1 [измерено: 8/8 corner; E:#199 run 28326217079]'),
 'int8':        dict(decode_hw='1 [измерено: 256/256 exhaustive; C:design on main, log≠#199]'),
 'nf4':         dict(decode_hw='1 [измерено: 16/16 exhaustive; C:design on main, log≠#199]'),
 'fp8_e4m3':    dict(decode_hw='1 [измерено: 256/256 exhaustive; C:design on main, log≠#199]'),
 'gf6':         dict(compute_hw='1 [измерено: 512/512 bit-exact; E:#199 artifact 7931202948]'),
 'gf8':         dict(compute_hw='1 [измерено: 512/512 bit-exact; E:#199 post-fix c0d24cac2]'),
}

def dynamic_range_decades(e_bits, m_bits, bias):
    """Грубая оценка десятичного динамического диапазона нормальных чисел."""
    if not e_bits: return None
    emax = (2**e_bits - 1) - 1 - bias  # минус 1 на Inf/NaN-кодировку (IEEE-like)
    emin = 1 - bias
    try:
        hi = (2 - 2**-m_bits) * (2.0**emax)
        lo = 2.0**emin
        return round(math.log10(hi) - math.log10(lo), 1)
    except (OverflowError, ValueError):
        # для очень больших экспонент считаем в log-домене
        return round((emax - emin) * math.log10(2), 1)

def decimal_digits(m_bits):
    if m_bits is None: return None
    return round((m_bits + 1) * math.log10(2), 1)  # +1 implicit

rows = []
for fid, f in formats.items():
    p = ix.get(fid, {})
    e = f.get('e_bits'); m = f.get('m_bits'); bias = f.get('bias', 0)
    hw = HW.get(fid, {})
    rows.append(dict(
        id=fid,
        name=f.get('name', fid),
        bits=f.get('bits'),
        layout=f"{f.get('s_bits','?')}/{e if e is not None else '?'}/{m if m is not None else '?'}",
        bias=bias,
        decimal_digits=decimal_digits(m),
        dyn_range_dec=dynamic_range_decades(e, m, bias) if e else None,
        phi_distance=f.get('phi_distance'),
        cluster=f.get('cluster','?'),
        gf_relation=f.get('gf_relation','?'),
        status=f.get('status','?'),
        standard=f.get('standard','?'),
        use_case=f.get('use_case','?'),
        # conformance (SSOT INDEX)
        sw_kind=p.get('kind','(нет в INDEX)'),
        sw_vectors=p.get('n_vectors'),
        # HW (только реально измеренное)
        decode_hw=hw.get('decode_hw','0'),
        compute_hw=hw.get('compute_hw','0'),
        source=f.get('source','?'),
    ))

# сортировка: GoldenFloat сначала (по битам), потом по кластеру/битам
cluster_order = ['GoldenFloat','Ieee754Binary','MlLowPrecision','Microscaling','QuantTuned',
                 'PositUnumIII','Lns','ExtendedFloat','Ieee754Decimal','IntegerFixed',
                 'HistoricalVendor','CompressionTrick','Theoretical']
def sk(r):
    c = r['cluster']
    return (cluster_order.index(c) if c in cluster_order else 99, r['bits'] or 0, r['id'])
rows.sort(key=sk)

# --- CSV ---
cols = ['id','name','bits','layout','bias','decimal_digits','dyn_range_dec','phi_distance',
        'cluster','gf_relation','status','standard','use_case','sw_kind','sw_vectors',
        'decode_hw','compute_hw','source']
with open('/home/user/workspace/metrics_83/metrics_83_formats.csv','w',newline='',encoding='utf-8') as fh:
    w = csv.DictWriter(fh, fieldnames=cols); w.writeheader()
    for r in rows: w.writerow(r)

json.dump(rows, open('/home/user/workspace/metrics_83/metrics_83.json','w'), ensure_ascii=False, indent=1)

# --- сводка ---
from collections import Counter
print("ИТОГО форматов:", len(rows))
print("По кластерам:", dict(Counter(r['cluster'] for r in rows)))
print("SW kind:", dict(Counter(r['sw_kind'] for r in rows)))
print("decode-HW >0:", [r['id'] for r in rows if r['decode_hw']!='0'])
print("compute-HW >0:", [r['id'] for r in rows if r['compute_hw']!='0'])
print("Σ SW vectors:", sum(r['sw_vectors'] or 0 for r in rows))
