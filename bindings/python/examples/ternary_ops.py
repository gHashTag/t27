"""
Ternary Logic Operations Demo
Demonstrates K3 Kleene logic operations
"""

from t27 import Trit, k3_and, k3_or, k3_not, k3_implies, k3_equiv, is_restraint, apply_restraint
import numpy as np

print("=" * 50)
print("Ternary Logic Operations Demo")
print("=" * 50)
print()

# Basic trit creation
print("1. Creating Trits:")
print(f"   Trit.POS  = {Trit.POS}  ({Trit.POS.value})")
print(f"   Trit.ZERO = {Trit.ZERO} ({Trit.ZERO.value})")
print(f"   Trit.NEG  = {Trit.NEG}  ({Trit.NEG.value})")
print()

# Conversion
print("2. Converting to/from integers:")
for i in [-2, -1, 0, 1, 2]:
    trit = Trit.from_int(i)
    print(f"   {i:2d} -> {trit} ({trit.value})")
print()

# AND operation
print("3. K3 AND (k3_and) - Truth Table:")
print("   ∧ |  F  |  U  |  T")
print("  ---|-----|-----|-----")
for a in [Trit.NEG, Trit.ZERO, Trit.POS]:
    row = []
    for b in [Trit.NEG, Trit.ZERO, Trit.POS]:
        result = k3_and(a, b)
        row.append(f"  {str(result)}  ")
    label = " F " if a == Trit.NEG else (" U " if a == Trit.ZERO else " T ")
    print(f"  {label}|{'|'.join(row)}")
print()

# OR operation
print("4. K3 OR (k3_or) - Truth Table:")
print("   ∨ |  F  |  U  |  T")
print("  ---|-----|-----|-----")
for a in [Trit.NEG, Trit.ZERO, Trit.POS]:
    row = []
    for b in [Trit.NEG, Trit.ZERO, Trit.POS]:
        result = k3_or(a, b)
        row.append(f"  {str(result)}  ")
    label = " F " if a == Trit.NEG else (" U " if a == Trit.ZERO else " T ")
    print(f"  {label}|{'|'.join(row)}")
print()

# NOT operation
print("5. K3 NOT (k3_not):")
for t in [Trit.NEG, Trit.ZERO, Trit.POS]:
    result = k3_not(t)
    print(f"   NOT {str(t)} = {str(result)}")
print()

# Implication
print("6. K3 Implication (k3_implies):")
print("   Note: 'ex falso quodlibet' - False implies anything")
for a in [Trit.NEG, Trit.ZERO, Trit.POS]:
    for b in [Trit.NEG, Trit.ZERO, Trit.POS]:
        result = k3_implies(a, b)
        print(f"   {str(a)} → {str(b)} = {str(result)}")
print()

# Equivalence
print("7. K3 Equivalence (k3_equiv):")
print("   (a→b) ∧ (b→a)")
for a in [Trit.NEG, Trit.ZERO, Trit.POS]:
    for b in [Trit.NEG, Trit.ZERO, Trit.POS]:
        result = k3_equiv(a, b)
        print(f"   {str(a)} ↔ {str(b)} = {str(result)}")
print()

# Restraint (bounded rationality)
print("8. Restraint (Bounded Rationality):")
print(f"   is_restraint(Trit.POS)  = {is_restraint(Trit.POS)}")
print(f"   is_restraint(Trit.ZERO) = {is_restraint(Trit.ZERO)}")
print(f"   is_restraint(Trit.NEG)  = {is_restraint(Trit.NEG)}")
print()

# Apply restraint (safe defaults)
print("9. Apply Restraint (replace UNKNOWN with FALSE for safety):")
values = [Trit.POS, Trit.ZERO, Trit.NEG, Trit.ZERO, Trit.POS]
print(f"   Before: {[str(v) for v in values]}")
safe = apply_restraint(values)
print(f"   After:  {[str(v) for v in safe]}")
print()

# Vectorized operations
print("10. Vectorized Operations (NumPy arrays):")
a = np.array([1, 0, -1, 0, 1])
b = np.array([1, -1, 0, 0, -1])
print(f"    a = {a}")
print(f"    b = {b}")
from t27 import k3_and_vector, k3_or_vector
print(f"    a ∧ b = {k3_and_vector(a, b)}")
print(f"    a ∨ b = {k3_or_vector(a, b)}")
print()

# TernaryWord packing
print("11. Ternary Word Packing:")
from t27 import TernaryWord
trits = [Trit.POS, Trit.ZERO, Trit.NEG] * 9  # 27 trits
word = TernaryWord.from_trits(trits)
print(f"    Packed 27 trits into {len(word.data)} bytes")
print(f"    Hex: {word.data.hex()}")
print(f"    First 5 trits: {[str(word.get_trit(i)) for i in range(5)]}")
print()

print("=" * 50)
print("Demo Complete!")
print("=" * 50)
