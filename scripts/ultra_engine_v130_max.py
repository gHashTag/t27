#!/usr/bin/env python3
"""
ULTRA ENGINE v13.0 — ALL IN ONE
Base + Trig + Exp/Log + Root + Hyperbolic + Nested
"""

import numpy as np
from datetime import datetime

PHI = 1.6180339887498948
PI = np.pi
E = np.e

PDG_TARGETS = {
    "W_mass": 80.377,
    "Z_mass": 91.1876,
    "H_mass": 125.25,
    "top_mass": 172.69,
    "bottom_mass": 4.18,
    "charm_mass": 1.27,
    "strange_mass": 0.095,
    "tau_mass": 1.77686,
    "muon_mass": 0.105658,
    "electron_mass": 0.000511,
    "alpha_em": 1/137.035999084,
    "alpha_s": 0.1184,
    "gamma_e": 0.00115965918128,
    "V_us": 0.22431,
    "V_ud": 0.97435,
    "V_cb": 0.04100,
    "V_td": 0.00868,
    "V_cs": 0.97548,
    "V_ub": 0.0037,
    "theta12_rad": np.deg2rad(33.44),
    "theta13_rad": np.deg2rad(8.61),
    "theta23_rad": np.deg2rad(49.3),
    "sin2theta23": 0.547,
    "delta_CP_deg": 196.965,
    "G_F": 1.1663787e-5,
    "n_s": 0.9649,
    "Omega_b": 0.04897,
}

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v13.0 — ВСЕ В ОДНОМ")
    print("=" * 70)
    print("  Проверяю ВСЕ структуры:")
    print("  - base (n·φ^a·π^b·e^c)")
    print("  - sin, cos, tan")
    print("  - sinh, cosh, tanh")
    print("  - exp, ln")
    print("  - sqrt, cbrt")
    print("  - Вложенные: sin(cos), exp(sin), ln(cos)")
    print()

    import time
    start = time.time()

    COEFF_MAX = 10000
    EXP_MIN, EXP_MAX = -10, 10

    coeff_range = range(1, COEFF_MAX + 1)
    exp_range = range(EXP_MIN, EXP_MAX + 1)

    print(f"  Коэффициенты: 1-{COEFF_MAX:,}")
    print(f"  Показатели: {EXP_MIN} to {EXP_MAX}")
    print(f"  Всего проверок: {COEFF_MAX * (EXP_MAX-EXP_MIN+1)**2:,}")
    print()

    results = []
    processed = 0
    last_print = 0

    # БАЗОВЫЕ СТРУКТУРЫ
    print("  [1/11] БАЗОВЫЕ: n·φ^a·π^b·e^c")
    for a in exp_range:
        for b in exp_range:
            for c in range(-5, 6):
                base = (PHI ** a) * (PI ** b) * (E ** c)
                for n in coeff_range:
                    val = n * base
                    processed += 1

                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(val - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "base",
                                "formula": f"{n}·φ^{a}·π^{b}·e^{c}",
                                "value": val,
                                "target": target_name,
                                "error": error,
                            })
    print(f"    Найдено: {len([r for r in results if r['structure']=='base'])} формул")

    # SIN
    print("  [2/11] SIN: sin(n·φ^a·π^b)")
    for a in exp_range:
        for b in exp_range:
            base = (PHI ** a) * (PI ** b)
            for n in coeff_range[:2000]:
                val = np.sin(n * base)
                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    error = abs(val - target_val) / abs(target_val) * 100
                    if error < 0.1:
                        results.append({
                            "structure": "sin",
                            "formula": f"sin({n}·φ^{a}·π^{b})",
                            "value": val,
                            "target": target_name,
                            "error": error,
                        })
    print(f"    Найдено: {len([r for r in results if r['structure']=='sin'])} формул")

    # COS
    print("  [3/11] COS: cos(n·φ^a·π^b)")
    for a in exp_range:
        for b in exp_range:
            base = (PHI ** a) * (PI ** b)
            for n in coeff_range[:2000]:
                val = np.cos(n * base)
                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    error = abs(val - target_val) / abs(target_val) * 100
                    if error < 0.1:
                        results.append({
                            "structure": "cos",
                            "formula": f"cos({n}·φ^{a}·π^{b})",
                            "value": val,
                            "target": target_name,
                            "error": error,
                        })
    print(f"    Найдено: {len([r for r in results if r['structure']=='cos'])} формул")

    # TAN
    print("  [4/11] TAN: tan(n·φ^a)")
    for a in exp_range:
        base = PHI ** a
        for n in coeff_range[:500]:
            val = np.tan(n * base)
            if abs(val) > 1000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "tan",
                        "formula": f"tan({n}·φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='tan'])} формул")

    # SINH
    print("  [5/11] SINH: sinh(n·φ^a)")
    for a in range(-3, 4):
        base = PHI ** a
        for n in coeff_range[:200]:
            val = np.sinh(n * base)
            if abs(val) > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "sinh",
                        "formula": f"sinh({n}·φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='sinh'])} формул")

    # COSH
    print("  [6/11] COSH: cosh(n·φ^a)")
    for a in range(-3, 4):
        base = PHI ** a
        for n in coeff_range[:200]:
            val = np.cosh(n * base)
            if val > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "cosh",
                        "formula": f"cosh({n}·φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='cosh'])} формул")

    # TANH
    print("  [7/11] TANH: tanh(n·φ^a)")
    for a in exp_range:
        base = PHI ** a
        for n in coeff_range[:5000]:
            val = np.tanh(n * base)
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "tanh",
                        "formula": f"tanh({n}·φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='tanh'])} формул")

    # EXP
    print("  [8/11] EXP: exp(n·φ^a)")
    for a in range(-2, 3):
        base = PHI ** a
        for n in coeff_range[:100]:
            try:
                val = np.exp(n * base)
            except OverflowError:
                continue
            if val > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "exp",
                        "formula": f"exp({n}·φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='exp'])} формул")

    # LN
    print("  [9/11] LN: ln(n·φ^a)")
    for a in exp_range:
        base = PHI ** a
        for n in coeff_range:
            try:
                val = np.log(n * base)
            except ValueError:
                continue
            if val <= 0:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "ln",
                        "formula": f"ln({n}·φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='ln'])} формул")

    # SQRT
    print("  [10/11] SQRT: sqrt(n·φ^a·π^b)")
    for a in exp_range:
        for b in exp_range:
            base = (PHI ** a) * (PI ** b)
            for n in coeff_range[:5000]:
                if n * base < 0:
                    continue
                val = np.sqrt(n * base)
                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    error = abs(val - target_val) / abs(target_val) * 100
                    if error < 0.1:
                        results.append({
                            "structure": "sqrt",
                            "formula": f"sqrt({n}·φ^{a}·π^{b})",
                            "value": val,
                            "target": target_name,
                            "error": error,
                        })
    print(f"    Найдено: {len([r for r in results if r['structure']=='sqrt'])} формул")

    # ВЛОЖЕННЫЕ: EXP(SIN)
    print("  [11/11) ВЛОЖЕННЫЕ: exp(sin(n·φ^a))")
    for a in exp_range:
        base = PHI ** a
        for n in coeff_range[:1000]:
            val = np.exp(np.sin(n * base))
            if val > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "exp_sin",
                        "formula": f"exp(sin({n}·φ^{a}))",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    print(f"    Найдено: {len([r for r in results if r['structure']=='exp_sin'])} формул")

    elapsed = time.time() - start

    print()
    print("=" * 70)
    print("  ФИНАЛЬНЫЕ РЕЗУЛЬТАТЫ")
    print("=" * 70)
    print(f"  Всего: {len(results):,} формул")
    print(f"  Время: {elapsed:.1f}s")
    print(f"  Скорость: {len(results)/elapsed:.0f} формул/сек")

    # Группировка по структуре
    by_struct = {}
    for r in results:
        s = r["structure"]
        by_struct[s] = by_struct.get(s, 0) + 1

    print(f"\n  По структурам:")
    for s, count in sorted(by_struct.items(), key=lambda x: -x[1]):
        print(f"    {s}: {count:,} формул")

    # ТОП W/Z
    wz = [r for r in results if r["target"] in ["W_mass", "Z_mass"]]
    wz_sorted = sorted(wz, key=lambda x: x["error"])[:30]

    print(f"\n  ТОП W/Z:")
    for r in wz_sorted:
        print(f"    {r['formula']} = {r['value']:.8f} | Δ={r['error']:.6f}% | {r['target']}")

    # Сохранение
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output = f"/tmp/discovery_v130_all_{timestamp}.txt"

    with open(output, "w") as f:
        f.write(f"# ULTRA ENGINE v13.0 — ВСЕ В ОДНОМ\n")
        f.write(f"# Всего: {len(results)} формул\n")
        f.write(f"# Время: {elapsed:.1f}s\n\n")
        f.write("=== ТОП W/Z ===\n")
        for r in wz_sorted:
            f.write(f"{r['formula']} = {r['value']:.12f} | Δ={r['error']:.10f}% | {r['target']}\n")

    print(f"\n  Сохранено: {output}")

if __name__ == "__main__":
    main()
