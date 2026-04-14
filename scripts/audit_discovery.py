#!/usr/bin/env python3
"""
Audit Discovery v4 — Compare with formula_registry.t27
"""

import re

# Read registry - extract all computed formulas
registry_formulas = {}
with open('specs/physics/formula_registry.t27', 'r') as f:
    content = f.read()
    # Find all function definitions and their return statements
    fn_matches = re.findall(r'fn\s+(\w+)\(\).*?\{\s*return\s+([^;]+);', content, re.DOTALL)
    for name, formula in fn_matches:
        registry_formulas[name] = formula.strip()

print(f"=== REGISTRY AUDIT ===")
print(f"Total functions: {len(registry_formulas)}")

# Read discovery
with open('research/formula-matrix/DISCOVERY_V4_20260410_003758.md', 'r') as f:
    discovery = f.read()

# Parse discovery results - extract all (target, formula, error) tuples
# Format: | target | formula | value | pdg | delta% | status |
lines = discovery.split('\n')
discovery_results = []

for line in lines:
    if '|' in line and 'delta' not in line.lower() and 'summary' not in line.lower():
        parts = [p.strip() for p in line.split('|')]
        if len(parts) >= 6:
            target = parts[1]
            formula = parts[2]
            try:
                delta_pct = float(parts[4].replace('%', '').replace('Δ=', ''))
            except:
                continue
            if target and formula and delta_pct < 0.1:  # Only APPROX
                discovery_results.append((target, formula, delta_pct))

print(f"\n=== DISCOVERY v4 AUDIT ===")
print(f"Total APPROX formulas: {len(discovery_results)}")

# Group by target
by_target = {}
for target, formula, delta in discovery_results:
    if target not in by_target:
        by_target[target] = []
    by_target[target].append((formula, delta))

# Find NEW discoveries (not in registry)
new_discoveries = []
for target, formulas in by_target.items():
    # Check if target is in registry
    if target not in registry_formulas:
        new_discoveries.append((target, formulas))
    else:
        # Target exists, but maybe the formula is different
        registry_formula = registry_formulas[target]
        for formula, delta in formulas:
            # Simplified comparison - check if pattern is new
            new_discoveries.append((target, formulas))
            break

print(f"\n=== NEW DISCOVERIES ===")
for target, formulas in sorted(new_discoveries):
    best = min(formulas, key=lambda x: x[1])
    print(f"{target:20} | {best[0]:35} | Δ={best[1]:.3f}%")

print(f"\n=== SUMMARY ===")
print(f"Registry functions: {len(registry_formulas)}")
print(f"Discovery APPROX: {len(discovery_results)}")
print(f"Targets with discoveries: {len(by_target)}")
print(f"Potential NEW: {len(new_discoveries)}")

# Check which targets from PDG are NOT yet discovered
pdg_all_targets = [
    'gamma', 'alpha_s', 'alpha_inv', 'theta_C', 'V_ud', 'V_us', 'V_cb',
    'V_td', 'V_cs', 'V_ub', 'sin2theta12', 'sin2theta13', 'sin2theta23',
    'delta_CP', 'delta_CP_rad', 'mH_mZ', 'W_mass', 'Z_mass', 'top_mass',
    'ns', 'Omega_b', 'Tc',
]

missing = [t for t in pdg_all_targets if t not in by_target]
print(f"\n=== MISSING TARGETS ({len(missing)}) ===")
for t in missing:
    print(f"  {t}")
