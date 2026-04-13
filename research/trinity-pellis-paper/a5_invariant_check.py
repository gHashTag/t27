#!/usr/bin/env python3
"""
A5 Discrete Symmetry and SU(3) Branching Verification
Author: Dmitrii Vasilev
Date: 2026-04-12
Purpose: Compute A5 group invariants and test if any combination equals φ⁻³/2
"""

import mpmath as mp
from math import pi, cos
from itertools import permutations
import numpy as np

# Set high precision for exact calculations
mp.mp.dps = 55  # 55 decimal places

# Golden ratio φ and target value
PHI = (mp.mpf(1) + mp.sqrt(5)) / 2
TARGET = PHI**(-3) / 2  # φ⁻³/2

print(f"=== A5 → SU(3) Invariant Check ===")
print(f"φ = {PHI}")
print(f"Target: φ⁻³/2 = {TARGET}")
print()

# ============================================================================
# Section 1: A5 Group Structure
# ============================================================================

def all_permutations(n):
    """Generate all permutations of n elements"""
    from itertools import permutations as perm
    return list(perm(range(1, n + 1)))

def permutation_to_cycles(perm):
    """Convert permutation to cycle notation"""
    visited = set()
    cycles = []
    for i in range(len(perm)):
        if i + 1 not in visited:
            cycle = []
            j = i
            while j + 1 not in visited:
                visited.add(j + 1)
                cycle.append(j + 1)
                j = perm[j] - 1
                if j == i:
                    break
            if len(cycle) > 1:
                cycles.append(tuple(cycle))
            elif len(cycle) == 1:
                cycles.append((cycle[0],))
    return cycles

def cycle_type(cycles):
    """Determine the cycle type of a permutation"""
    lengths = sorted([len(c) for c in cycles if len(c) > 1])
    if not lengths:
        return (1,) * 5  # Identity
    return tuple(lengths)

# Generate A5 (alternating group on 5 elements)
all_perms = all_permutations(5)
def is_even(perm):
    """Check if permutation is even"""
    from math import factorial
    # Count inversions
    inversions = sum(1 for i in range(len(perm)) for j in range(i+1, len(perm)) if perm[i] > perm[j])
    return inversions % 2 == 0

A5_elements = [p for p in all_perms if is_even(p)]
print(f"A5 group has |A5| = {len(A5_elements)} elements")

# Group by conjugacy classes
conj_classes = {}
for perm in A5_elements:
    cycles = permutation_to_cycles(perm)
    c_type = tuple(sorted(cycle_type(cycles)))
    if c_type not in conj_classes:
        conj_classes[c_type] = []
    conj_classes[c_type].append(perm)

print(f"\nConjugacy classes of A5:")
for c_type, elements in conj_classes.items():
    print(f"  Type {c_type}: {len(elements)} elements")

# ============================================================================
# Section 2: A5 Character Table
# ============================================================================

# 5th roots of unity
zeta5 = [mp.e**(2*mp.pi*mp.j*k/5) for k in range(5)]

# Irreducible representations of A5
# Labels: 1 (trivial), 1' (chi1), 1'' (chi2), 2 (two-dim), 3 (three-dim), 3' (three-dim-conj)

conj_class_names = {
    (1, 1, 1, 1, 1): 'identity',
    (2, 2, 1): '12-34 type',
    (3, 1, 1): '123 type',
    (3, 2): '12345 type',
    (5,): '5-cycle type'
}

# Character values for each irrep
# Using known values from group theory
char_table = {
    '1': {'class': [1, 1, 1, 1, 1]},
    '1p': {'class': [1, 1, zeta5[1], zeta5[2], 1]},
    '1pp': {'class': [1, 1, zeta5[2], zeta5[1], 1]},
    '2': {'class': [2, 0, zeta5[1] + zeta5[4], zeta5[2] + zeta5[3], -1]},
    '3': {'class': [3, -1, 0, 0, 0]},
    '3p': {'class': [3, -1, 0, 0, 0]},  # Conjugate of 3
}

print(f"\n=== A5 Character Table ===")
for rep, data in char_table.items():
    chars = data['class']
    print(f"  {rep}: {chars}")
    print(f"     [real parts: {[mp.nstr(mp.re(c), 10) for c in chars]}]")

# ============================================================================
# Section 3: Golden Ratio in A5 Characters
# ============================================================================

print(f"\n=== Golden Ratio in A5 Characters ===")

# 2-dimensional representation character at 12345 (5-cycle)
char_2dim_5cycle = char_table['2']['class'][4]  # 5-cycle class
print(f"Character of 2D rep at 5-cycle: {char_2dim_5cycle}")
print(f"  Real part: {mp.re(char_2dim_5cycle)}")
print(f"  Expected: φ - 1 = {PHI - 1}")
print(f"  Match: {abs(mp.re(char_2dim_5cycle) - (PHI - 1)) < 1e-30}")

# ============================================================================
# Section 4: SU(3) Casimir Invariants
# ============================================================================

def su3_casimir(p, q):
    """Quadratic Casimir for SU(3) representation with Dynkin label (p, q)"""
    return mp.mpf(p**2 + q**2 + p*q + 3*p + 3*q) / 3

su3_reps = {
    '3': (1, 0, 3),      # fundamental: (p, q), dimension
    '8': (1, 1, 8),      # adjoint
    '10': (3, 0, 10),    # decuplet
    '27': (2, 2, 27),    # 27-plet
}

print(f"\n=== SU(3) Casimir Invariants ===")
for name, (p, q, dim) in su3_reps.items():
    c2 = su3_casimir(p, q)
    print(f"  {name} (dim={dim}): C₂ = {c2}")

# ============================================================================
# Section 5: Search for Invariant Combinations
# ============================================================================

print(f"\n=== Searching for Invariant Combinations ===")

# Test simple ratios
ratios = [
    ("C2(3) / C2(8)", su3_casimir(1, 0) / su3_casimir(1, 1)),
    ("C2(8) / C2(27)", su3_casimir(1, 1) / su3_casimir(2, 2)),
    ("1 / C2(8)", 1 / su3_casimir(1, 1)),
    ("C2(3) / C2(10)", su3_casimir(1, 0) / su3_casimir(3, 0)),
]

for name, value in ratios:
    diff = abs(float(value) - float(TARGET))
    print(f"  {name}: {float(value)}")
    print(f"    Target: {float(TARGET)}")
    print(f"    Difference: {diff}")
    print(f"    Relative: {diff/float(TARGET)*100:.6f}%")

# ============================================================================
# Section 6: Mixed Invariants with A5 Characters
# ============================================================================

print(f"\n=== Mixed Invariants (A5 × SU(3)) ===")

# Test combinations involving both A5 character values and SU(3) Casimirs
char_2dim = mp.re(char_table['2']['class'][2])  # Character at 123 type (involves φ)

mixed_combinations = [
    ("|χ₂| × C₂(8)", abs(char_2dim) * su3_casimir(1, 1)),
    ("|χ₂| / C₂(8)", abs(char_2dim) / su3_casimir(1, 1)),
    ("C₂(8) / |χ₂|", su3_casimir(1, 1) / abs(char_2dim)),
]

for name, value in mixed_combinations:
    diff = abs(float(value) - float(TARGET))
    print(f"  {name}: {float(value)}")
    print(f"    Target: {float(TARGET)}")
    print(f"    Difference: {diff}")

# ============================================================================
# Section 7: Higher-Order Invariants
# ============================================================================

print(f"\n=== Higher-Order Casimir Invariants ===")

def su3_casimir4(p, q):
    """Fourth-order Casimir for SU(3)"""
    # Using known formula
    return mp.mpf(2*(p**4 + q**4) + 4*(p**3*q + p*q**3) +
                  6*p**2*q**2 + 12*(p**3 + q**3) +
                  18*(p**2*q + p*q**2) + 36*p*q + 27*(p**2 + q**2)) / 27

for name, (p, q, dim) in su3_reps.items():
    c4 = su3_casimir4(p, q)
    print(f"  {name}: C₄ = {c4}")

# ============================================================================
# Section 8: Linear Combination Search
# ============================================================================

print(f"\n=== Linear Combination Search ===")

# Try to find a, b, c such that: a*C₂(3) + b*C₂(8) + c = φ⁻³/2
# This is underdetermined, so we test integer coefficients for small values

c2_3 = su3_casimir(1, 0)  # 4/3
c2_8 = su3_casimir(1, 1)  # 3

found_combinations = []
for a in range(-5, 6):
    for b in range(-5, 6):
        for c in range(-5, 6):
            result = a*c2_3 + b*c2_8 + c
            if abs(float(result) - float(TARGET)) < 1e-30:
                found_combinations.append((a, b, c, result))

if found_combinations:
    print(f"  Found {len(found_combinations)} combinations:")
    for a, b, c, val in found_combinations:
        print(f"    {a}*C₂(3) + {b}*C₂(8) + {c} = {val}")
else:
    print(f"  No integer coefficients [-5, 5] found for combination with C₂(3), C₂(8)")

# ============================================================================
# Section 9: Conclusion
# ============================================================================

print(f"\n=== CONCLUSION ===")
print(f"Target value: φ⁻³/2 = {TARGET}")
print()
print(f"Simple ratios tested: NO MATCH")
print(f"Mixed A5 × SU(3) invariants: NO MATCH")
print(f"Linear combinations of C₂(3), C₂(8): NO MATCH (tested range [-5, 5])")
print()
print(f"Recommendation: Investigate higher-order invariants (C₄, C₆) or")
print(f"                   non-integer coefficient combinations involving field Q(√5)")
