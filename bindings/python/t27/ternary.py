# t27 Ternary Operations Module
# Python implementation of TRINITY's ternary logic
# φ² + 1/φ² = 3 | TRINITY

"""
Ternary operations for TRINITY framework.

This module implements the core ternary logic operations (K3 Kleene logic)
in pure Python, providing a drop-in replacement for binary operations where
ternary logic ({-1, 0, +1}) is preferred.
"""

from enum import Enum
from typing import Union, List, Optional
import numpy as np


class Trit(Enum):
    """Ternary value: {-1, 0, +1} representing Kleene {False, Unknown, True}."""
    NEG = -1   # K_FALSE
    ZERO = 0   # K_UNKNOWN (restraint)
    POS = 1    # K_TRUE

    def __int__(self) -> int:
        return self.value

    def __float__(self) -> float:
        return float(self.value)

    def __neg__(self) -> 'Trit':
        """Negate trit."""
        return Trit(-self.value)

    def __invert__(self) -> 'Trit':
        """Logical NOT (same as negation for trits)."""
        return Trit.POS if self == Trit.NEG else (Trit.ZERO if self == Trit.ZERO else Trit.NEG)

    def __repr__(self) -> str:
        return f"Trit.{self.name}"

    def __str__(self) -> str:
        if self == Trit.POS:
            return "+1"
        elif self == Trit.NEG:
            return "-1"
        else:
            return "0"

    @staticmethod
    def from_int(value: int) -> 'Trit':
        """Convert integer to Trit (clamps to {-1, 0, 1})."""
        if value > 0:
            return Trit.POS
        elif value < 0:
            return Trit.NEG
        else:
            return Trit.ZERO

    @staticmethod
    def from_float(value: float) -> 'Trit':
        """Convert float to Trit (clamps to {-1, 0, 1})."""
        return Trit.from_int(int(np.sign(value)))


# K3 Kleene Logic Operations

def k3_and(a: Union[Trit, int], b: Union[Trit, int]) -> Trit:
    """
    Kleene AND operation: minimum of truth values.

    Truth table:
      ∧  |  F  |  U  |  T
     ---|-----|-----|-----
      F |  F  |  F  |  F
      U |  F  |  U  |  U
      T |  F  |  U  |  T

    Args:
        a: First operand (Trit or int)
        b: Second operand (Trit or int)

    Returns:
        Trit result of a ∧ b
    """
    if not isinstance(a, Trit):
        a = Trit.from_int(a)
    if not isinstance(b, Trit):
        b = Trit.from_int(b)

    # AND = minimum in Kleene K3 ordering: F < U < T
    # POS (1) > ZERO (0) > NEG (-1)
    if a == Trit.NEG or b == Trit.NEG:
        return Trit.NEG
    elif a == Trit.ZERO and b == Trit.ZERO:
        return Trit.ZERO
    else:  # Both POS or one POS one ZERO
        return Trit.POS if a == Trit.POS and b == Trit.POS else Trit.ZERO


def k3_or(a: Union[Trit, int], b: Union[Trit, int]) -> Trit:
    """
    Kleene OR operation: maximum of truth values.

    Truth table:
      ∨  |  F  |  U  |  T
     ---|-----|-----|-----
      F |  F  |  U  |  T
      U |  U  |  U  |  T
      T |  T  |  T  |  T

    Args:
        a: First operand (Trit or int)
        b: Second operand (Trit or int)

    Returns:
        Trit result of a ∨ b
    """
    if not isinstance(a, Trit):
        a = Trit.from_int(a)
    if not isinstance(b, Trit):
        b = Trit.from_int(b)

    # OR = maximum in Kleene K3 ordering
    if a == Trit.POS or b == Trit.POS:
        return Trit.POS
    elif a == Trit.ZERO and b == Trit.ZERO:
        return Trit.ZERO
    else:  # One NEG, one ZERO
        return Trit.ZERO


def k3_not(a: Union[Trit, int]) -> Trit:
    """
    Kleene NOT operation: truth value inversion.

    Truth table:
      ¬ |  F  |  U  |  T
     ---|-----|-----|-----
         |  T  |  U  |  F

    Note: K_UNKNOWN is preserved (¬U = U in Kleene logic).

    Args:
        a: Operand (Trit or int)

    Returns:
        Trit result of ¬a
    """
    if not isinstance(a, Trit):
        a = Trit.from_int(a)

    # NOT inverts: POS <-> NEG, ZERO stays ZERO
    if a == Trit.POS:
        return Trit.NEG
    elif a == Trit.NEG:
        return Trit.POS
    else:
        return Trit.ZERO


def k3_implies(a: Union[Trit, int], b: Union[Trit, int]) -> Trit:
    """
    Kleene implication: ¬a ∨ b.

    Truth table:
      → |  F  |  U  |  T
     ---|-----|-----|-----
      F |  T  |  T  |  T  (ex falso quodlibet)
      U |  U  |  U  |  T
      T |  F  |  U  |  T

    Args:
        a: Antecedent (Trit or int)
        b: Consequent (Trit or int)

    Returns:
        Trit result of a → b
    """
    return k3_or(k3_not(a), b)


def k3_equiv(a: Union[Trit, int], b: Union[Trit, int]) -> Trit:
    """
    Kleene equivalence: (a→b) ∧ (b→a).

    Truth table:
      ↔ |  F  |  U  |  T
     ---|-----|-----|-----
      F |  T  |  U  |  F
      U |  U  |  U  |  U
      T |  F  |  U  |  T

    Args:
        a: First operand (Trit or int)
        b: Second operand (Trit or int)

    Returns:
        Trit result of a ↔ b
    """
    return k3_and(k3_implies(a, b), k3_implies(b, a))


def is_restraint(t: Trit) -> bool:
    """
    Check if a trit represents restraint (bounded rationality).

    In Kleene K3: K_UNKNOWN = restraint = don't-care = undefined.

    Args:
        t: Trit to check

    Returns:
        True if t == ZERO (K_UNKNOWN), False otherwise
    """
    return t == Trit.ZERO


def apply_restraint(values: List[Trit]) -> List[Trit]:
    """
    Apply restraint optimization: replace all K_UNKNOWN with K_FALSE.

    This implements "safe defaults" for bounded rationality.

    Args:
        values: List of Trit values

    Returns:
        List with K_UNKNOWN replaced by K_FALSE
    """
    return [Trit.NEG if is_restraint(t) else t for t in values]


# Vectorized operations for NumPy arrays

def k3_and_vector(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """
    Vectorized K3 AND operation for NumPy arrays.

    Maps integer values to trits, performs AND, maps back.

    Args:
        a: NumPy array of integers (will be clipped to {-1, 0, 1})
        b: NumPy array of integers (will be clipped to {-1, 0, 1})

    Returns:
        NumPy array of result trits (as -1, 0, 1)
    """
    a_clipped = np.clip(a, -1, 1)
    b_clipped = np.clip(b, -1, 1)

    # AND = minimum
    result = np.minimum(a_clipped, b_clipped)

    # Handle zero case (if either is zero, result is zero unless both are pos)
    # This is a simplification - for full K3 semantics, need proper logic
    zero_mask = (a_clipped == 0) | (b_clipped == 0)
    both_pos = (a_clipped == 1) & (b_clipped == 1)

    result = np.where(zero_mask, np.where(both_pos, 1, 0), result)
    return result


def k3_or_vector(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """
    Vectorized K3 OR operation for NumPy arrays.

    Args:
        a: NumPy array of integers (will be clipped to {-1, 0, 1})
        b: NumPy array of integers (will be clipped to {-1, 0, 1})

    Returns:
        NumPy array of result trits (as -1, 0, 1)
    """
    a_clipped = np.clip(a, -1, 1)
    b_clipped = np.clip(b, -1, 1)

    # OR = maximum
    result = np.maximum(a_clipped, b_clipped)

    # Handle zero case
    zero_mask = (a_clipped == 0) | (b_clipped == 0)
    both_neg = (a_clipped == -1) & (b_clipped == -1)

    result = np.where(zero_mask, np.where(both_neg, -1, 0), result)
    return result


# TernaryWord class for packed trit storage

class TernaryWord:
    """
    Packed ternary word storage (27 trits).

    Trits are packed 2 bits each: -1 → 10b (2), 0 → 00b (0), +1 → 01b (1)
    A 27-trit word requires ceil(27 * 2 / 8) = 7 bytes.
    """

    TRITS_PER_WORD = 27
    BYTES_PER_WORD = 7
    BITS_PER_TRIT = 2

    def __init__(self, data: Optional[bytes] = None):
        """
        Initialize TernaryWord.

        Args:
            data: 7-byte packed data (defaults to all zeros)
        """
        if data is None:
            self.data = bytes(self.BYTES_PER_WORD)
        elif len(data) == self.BYTES_PER_WORD:
            self.data = data
        else:
            raise ValueError(f"TernaryWord requires {self.BYTES_PER_WORD} bytes, got {len(data)}")

    @classmethod
    def from_trits(cls, trits: List[Trit]) -> 'TernaryWord':
        """
        Create TernaryWord from list of trits.

        Args:
            trits: List of up to 27 trits

        Returns:
            TernaryWord with packed trits
        """
        data = bytearray(cls.BYTES_PER_WORD)
        for i, trit in enumerate(trits[:cls.TRITS_PER_WORD]):
            encoded = {Trit.NEG: 2, Trit.ZERO: 0, Trit.POS: 1}[trit]
            byte_idx = (i * cls.BITS_PER_TRIT) // 8
            bit_idx = (i * cls.BITS_PER_TRIT) % 8

            if bit_idx == 0:
                data[byte_idx] = (data[byte_idx] & 0b11111100) | encoded
            else:
                data[byte_idx] = (data[byte_idx] & 0b00000011) | (encoded << 2)

        return cls(bytes(data))

    def get_trit(self, index: int) -> Trit:
        """
        Extract trit at given index.

        Args:
            index: Trit index (0-26)

        Returns:
            Trit value at index
        """
        if index < 0 or index >= self.TRITS_PER_WORD:
            raise IndexError(f"Trit index {index} out of range (0-{self.TRITS_PER_WORD-1})")

        bit_pos = index * self.BITS_PER_TRIT
        byte_idx = bit_pos // 8
        bit_idx = bit_pos % 8

        encoded = (self.data[byte_idx] >> bit_idx) & 0b11
        return {0: Trit.ZERO, 1: Trit.POS, 2: Trit.NEG}[encoded]

    def to_array(self) -> np.ndarray:
        """
        Convert to NumPy array of integers (-1, 0, 1).

        Returns:
            NumPy array of shape (27,)
        """
        result = np.zeros(self.TRITS_PER_WORD, dtype=np.int8)
        for i in range(self.TRITS_PER_WORD):
            result[i] = self.get_trit(i).value
        return result

    def __repr__(self) -> str:
        return f"TernaryWord({self.data.hex()})"
