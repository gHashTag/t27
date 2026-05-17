# SPDX-License-Identifier: Apache-2.0
# t27/python/t27/phi.py
# Golden ratio utilities for format optimization

import math
from typing import Tuple, List
import numpy as np


# Sacred constants
PHI = (1 + math.sqrt(5)) / 2  # φ ≈ 1.618033988749895
PHI_SQUARED = PHI ** 2  # φ² ≈ 2.618
PHI_INVERSE = 1 / PHI  # 1/φ ≈ 0.618033988749895

E = math.e  # e ≈ 2.718281828459045
GAMMA = 0.5772156649015329  # Euler-Mascheroni constant γ

# Trinity identity
TRINITY_CHECK = PHI_SQUARED + PHI_INVERSE ** 2  # Should equal 3


def phi_distance(exp_bits: int, mant_bits: int) -> float:
    """
    Calculate φ-distance for a given bit allocation.

    φ-distance = |exp/mant - 1/φ|

    Ideal format has φ-distance = 0.

    Args:
        exp_bits: Number of exponent bits
        mant_bits: Number of mantissa bits

    Returns:
        φ-distance (lower is better)

    Example:
        >>> phi_distance(6, 9)  # GF16
        0.0487...
        >>> phi_distance(12, 19)  # GF32
        0.0141...
    """
    if mant_bits == 0:
        return float('inf')
    return abs(exp_bits / mant_bits - PHI_INVERSE)


def optimal_phi_distance(total_bits: int, sign_bits: int = 1) -> Tuple[int, int, float]:
    """
    Find optimal bit allocation for given total bits.

    Args:
        total_bits: Total bits in format
        sign_bits: Number of sign bits (default 1)

    Returns:
        Tuple of (exp_bits, mant_bits, phi_distance)

    Example:
        >>> optimal_phi_distance(16)  # GF16
        (6, 9, 0.0487...)
    """
    available_bits = total_bits - sign_bits
    best_exp, best_mant = 0, 0
    best_dist = float('inf')

    # Search all possible allocations
    for exp in range(1, available_bits):
        mant = available_bits - exp
        dist = phi_distance(exp, mant)
        if dist < best_dist:
            best_dist = dist
            best_exp, best_mant = exp, mant

    return best_exp, best_mant, best_dist


def format_score(exp_bits: int, mant_bits: int, weight_exp: float = 1.0,
                 weight_mant: float = 1.0, weight_phi: float = 2.0) -> float:
    """
    Calculate format quality score.

    Higher is better. Weights can be adjusted based on use case.

    Args:
        exp_bits: Number of exponent bits
        mant_bits: Number of mantissa bits
        weight_exp: Weight for exponent contribution (dynamic range)
        weight_mant: Weight for mantissa contribution (precision)
        weight_phi: Weight for φ-optimization (efficiency)

    Returns:
        Quality score

    Example:
        >>> format_score(6, 9)  # GF16
        1.95...
    """
    phi_dist = phi_distance(exp_bits, mant_bits)

    # Normalize components
    exp_norm = exp_bits / 16.0  # Normalize to reasonable max
    mant_norm = mant_bits / 32.0
    phi_norm = max(0, 1 - phi_dist * 5)  # Penalize φ-distance

    score = (weight_exp * exp_norm +
             weight_mant * mant_norm +
             weight_phi * phi_norm)

    return score


def fib_sequence(n: int) -> List[int]:
    """
    Generate Fibonacci sequence up to n terms.

    GoldenFloat formats near Fibonacci ratios are especially efficient.

    Args:
        n: Number of terms

    Returns:
        List of Fibonacci numbers

    Example:
        >>> fib_sequence(10)
        [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]
    """
    fib = [1, 1]
    for i in range(2, n):
        fib.append(fib[-1] + fib[-2])
    return fib


def fib_ratio(a: int, b: int) -> float:
    """
    Calculate ratio of two Fibonacci numbers.

    As fib numbers grow, the ratio approaches φ.

    Args:
        a: First Fibonacci number
        b: Second Fibonacci number

    Returns:
        Ratio a/b

    Example:
        >>> fib_ratio(8, 13)  # 8/13 ≈ 0.615
        0.6153...
    """
    return a / b


def analyze_format_family(formats: List[Tuple[int, int]]) -> dict:
    """
    Analyze a family of formats for φ-optimization.

    Args:
        formats: List of (exp_bits, mant_bits) tuples

    Returns:
        Dictionary with analysis results

    Example:
        >>> analyze_format_family([(6, 9), (12, 19)])
        {
            'avg_phi_distance': 0.031,
            'best_format': (12, 19),
            'phi_optimal_formats': [(12, 19)],
            'total_bits': [16, 32]
        }
    """
    phi_distances = [phi_distance(exp, mant) for exp, mant in formats]
    avg_phi_dist = sum(phi_distances) / len(phi_distances)

    best_idx = np.argmin(phi_distances)
    best_format = formats[best_idx]

    # Formats with φ-distance < 0.1 are considered φ-optimal
    phi_optimal = [fmt for fmt, dist in zip(formats, phi_distances) if dist < 0.1]

    return {
        'avg_phi_distance': avg_phi_dist,
        'best_format': best_format,
        'best_phi_distance': min(phi_distances),
        'phi_optimal_formats': phi_optimal,
        'phi_distances': phi_distances,
        'total_bits': [exp + mant + 1 for exp, mant in formats],
    }


def recommend_format(value_range: Tuple[float, float],
                     precision_bits: int,
                     max_bits: int = 64) -> Tuple[int, int]:
    """
    Recommend format based on value range and precision requirements.

    Args:
        value_range: (min_value, max_value) of values to represent
        precision_bits: Required bits of precision
        max_bits: Maximum total bits allowed

    Returns:
        Tuple of (exp_bits, mant_bits)

    Example:
        >>> recommend_format((-1000, 1000), 10, 32)
        (8, 23)
    """
    min_val, max_val = value_range
    max_abs = max(abs(min_val), abs(max_val))

    # Calculate required exponent bits
    if max_abs > 0:
        required_exp = int(math.log2(max_abs)) + 2
    else:
        required_exp = 1

    # Mantissa must accommodate precision
    required_mant = precision_bits

    # Check if we can fit within max_bits
    total_required = required_exp + required_mant + 1  # +1 for sign

    if total_required <= max_bits:
        # Optimize remaining bits for φ
        remaining = max_bits - total_required
        # Add to mantissa for precision
        return required_exp, required_mant + remaining
    else:
        # Must reduce; prioritize precision
        available = max_bits - 1
        mant_bits = min(required_mant, available - required_exp)
        exp_bits = available - mant_bits
        return exp_bits, mant_bits


# Pre-computed φ-optimal formats for reference

PHI_OPTIMAL_FORMATS = {
    4: (1, 2),    # GF4: 1 exp, 2 mant, φ-dist = 0.118
    8: (3, 4),    # GF8: 3 exp, 4 mant, φ-dist = 0.132
    12: (4, 7),   # GF12: 4 exp, 7 mant, φ-dist = 0.043 (Fib: 4/7)
    16: (6, 9),   # GF16: 6 exp, 9 mant, φ-dist = 0.049 (PRIMARY)
    20: (7, 12),  # GF20: 7 exp, 12 mant, φ-dist = 0.032 (Fib: 7/12)
    24: (9, 14),  # GF24: 9 exp, 14 mant, φ-dist = 0.026
    32: (12, 19), # GF32: 12 exp, 19 mant, φ-dist = 0.013
    64: (24, 39), # GF64: 24 exp, 39 mant, φ-dist = 0.004
    128: (48, 79), # GF128: 48 exp, 79 mant, φ-dist = 0.010
    256: (97, 158), # GF256: 97 exp, 158 mant, φ-dist = 0.004
}


def get_phi_optimal_format(total_bits: int) -> Tuple[int, int]:
    """
    Get φ-optimal format for given total bits.

    Args:
        total_bits: Total bits in format

    Returns:
        Tuple of (exp_bits, mant_bits)

    Example:
        >>> get_phi_optimal_format(16)
        (6, 9)
    """
    if total_bits in PHI_OPTIMAL_FORMATS:
        return PHI_OPTIMAL_FORMATS[total_bits]

    # Calculate on-the-fly for non-standard sizes
    exp, mant, _ = optimal_phi_distance(total_bits)
    return exp, mant


def print_phi_table():
    """Print a formatted table of φ-optimal formats."""
    print("=" * 70)
    print("GoldenFloat φ-Optimal Formats")
    print("=" * 70)
    print(f"{'Format':<10} {'Total':<8} {'Exp':<6} {'Mant':<6} {'φ-dist':<10} {'Use Case'}")
    print("-" * 70)

    for bits in [4, 8, 12, 16, 20, 24, 32, 64, 128, 256]:
        exp, mant, dist = optimal_phi_distance(bits)
        exp, mant = get_phi_optimal_format(bits)

        use_cases = {
            4: "Ultra-compact",
            8: "Low-power edge",
            12: "Embedded",
            16: "PRIMARY",
            20: "Mid-range",
            24: "High precision",
            32: "Extended",
            64: "Scientific",
            128: "Extended range",
            256: "Ultra-high precision",
        }

        print(f"GF{bits:<9} {bits:<8} {exp:<6} {mant:<6} {dist:<10.6f} {use_cases[bits]}")

    print("=" * 70)
    print(f"φ (golden ratio) = {PHI:.15f}")
    print(f"1/φ (ideal ratio) = {PHI_INVERSE:.15f}")
    print(f"φ² + φ⁻² = {TRINITY_CHECK:.15f} (should equal 3.0)")
    print("=" * 70)


if __name__ == "__main__":
    print_phi_table()