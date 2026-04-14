#!/usr/bin/env python3
"""
Count Trinity search space for LEE analysis.
Reads FORMULA_TABLE.md and enumerates all unique expressions.

Usage:
    python3 scripts/lee_search_space_count.py
"""

import re
import json
from pathlib import Path

# Parse FORMULA_TABLE.md
FORMULA_TABLE = Path(__file__).parent.parent / "research/trinity-pellis-paper/FORMULA_TABLE.md"

# Parse formulas from markdown table
formulas = []
in_table = False
hits_pdg = 0

with open(FORMULA_TABLE) as f:
    for line in f:
        if "Core Formula Table" in line:
            in_table = True
        elif line.strip().startswith("|") and in_table:
            # Parse markdown table row: | ID | Name | Formula | ...
            parts = [p.strip() for p in line.split("|")[1:-1]]
            if len(parts) >= 4 and parts[3]:
                formula_str = parts[3].strip()
                # Remove extra spaces around formula
                formula_str = re.sub(r'\s+', '', formula_str)
                formulas.append(formula_str)

# Remove duplicates
formulas = list(set(formulas))

# Search space parameters
operators = ['+', '-', '*', '/', '**', 'sqrt', 'log']
constants = ['phi', 'pi', 'e']
max_complexity = 6

# Calculate search space size
# N_k = n^k × m × p × q where:
# n = number of operators (6)
# m = max complexity + 1 = 7 nodes in tree
# p = number of constants (3)
# q = number of constant assignments (3)

n_k = len(operators) ** max_complexity
m = max_complexity + 1
p = len(constants)
q = 3  # constant assignments: each constant can be 0-2 in exponent

N_total = n_k * m * p * q

print(f"=== Trinity Search Space Count ===")
print(f"Operators: {', '.join(operators)}")
print(f"Constants: {', '.join(constants)}")
print(f"Max complexity: {max_complexity} nodes")
print(f"m (nodes): {m}")
print(f"p (constants): {p}")
print(f"q (assignments): {q}")
print()
print(f"Total unique expressions (N_total): {N_total:,}")
print(f"Unique formulas found in table: {len(formulas)}")
print()

# Load existing sacred_constants.csv for PDG hit rate
sacred_file = Path(__file__).parent.parent / "docs/lab/papers/sacred/sacred_constants.csv"
pdg_constants = 0

if sacred_file.exists():
    with open(sacred_file) as f:
        lines = f.readlines()
        for line in lines[1:]:  # Skip header
            parts = line.strip().split(',')
            if len(parts) >= 2:
                formula_id = parts[0].strip()
                delta_pct = parts[1].strip()
                # Convert percentage to decimal
                if '%' in delta_pct:
                    delta_pct = float(delta_pct.rstrip('%')) / 100.0
                    if delta_pct < 0.001:
                        hits_pdg += 1

        pdg_constants = len(lines[1:])
        print(f"PDG constants tested: {pdg_constants}")
        print(f"Trinity hits (Δ < 0.001%): {hits_pdg}")
        if pdg_constants > 0:
            print(f"Trinity hit rate: {hits_pdg}/{pdg_constants} = {hits_pdg/pdg_constants:.4%}")
        print()

# Expected random hits (look-elsewhere correction)
# If 18 formulas with Δ < 0.01%, and random probability = 0.001% each
# Then expected random hits = N_random × p_random = 10000 × 0.001 = 10 hits
expected_random = 10000 * 0.001
print(f"Expected random hits (N=10000, p=0.001%): {expected_random}")

# Output JSON
output = {
    "N_total_expressions": N_total,
    "N_tested": len(formulas),
    "N_hits_pdg": hits_pdg,
    "PDG_constants_tested": pdg_constants,
    "expected_random_hits": expected_random
}

if pdg_constants > 0:
    output["PDG_hit_rate"] = f"{hits_pdg/pdg_constants:.4%}"

output_path = Path(__file__).parent.parent / "research/lee-analysis/search_space_count.json"
with open(output_path, 'w') as f:
    json.dump(output, f, indent=2)

print(f"Saved to: {output_path}")
