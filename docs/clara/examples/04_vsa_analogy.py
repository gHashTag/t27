#!/usr/bin/env python3
"""
Example 4: VSA Analogy Reasoning with Bind/Unbind
===================================================

Composition: Entity Encoding → VSA Bind/Unbind → Similarity Search → AR

This example demonstrates VSA analogy reasoning:
- A:B :: C:? using the property: bind(A, B) unbind(C) ≈ D
- Bundle superposition for set-like reasoning
- Position-aware encoding for sequence understanding

Key VSA properties demonstrated:
- bind(a, bind(a, b)) = b (self-inverse)
- bundle3 for consensus voting
- permute for position-aware encoding

Author: T27 Trinity Ternary Project
SPDX-License-Identifier: Apache-2.0
"""

from dataclasses import dataclass
from typing import List, Tuple, Dict, Optional
import math
import random


# ============================================================================
# T27 Ternary Types (from specs/base/types.t27)
# ============================================================================

TRIT_NEG = -1
TRIT_ZERO = 0
TRIT_POS = 1

Trit = int

VSA_DIM = 1024


# ============================================================================
# VSA Operations (from specs/vsa/ops.t27)
# ============================================================================

def generate_random_hv(seed: int, dim: int = VSA_DIM) -> List[Trit]:
    """Generate a deterministic random hypervector."""
    random.seed(seed)
    return [random.choice([TRIT_NEG, TRIT_ZERO, TRIT_POS]) for _ in range(dim)]


def bind(a: List[Trit], b: List[Trit], length: int = VSA_DIM) -> List[Trit]:
    """
    Bind operation (XOR-like).

    Property: bind(a, bind(a, b)) = b (self-inverse)
    Used for: associative memory, role-value pairing
    """
    result = []
    for i in range(length):
        ai, bi = a[i], b[i]
        if ai == TRIT_ZERO:
            result.append(bi)
        elif bi == TRIT_ZERO:
            result.append(ai)
        else:
            # Both non-zero: multiply
            result.append(TRIT_POS if ai == bi else TRIT_NEG)
    return result


def unbind(bound: List[Trit], key: List[Trit], length: int = VSA_DIM) -> List[Trit]:
    """
    Unbind operation (inverse of bind).

    For XOR-like bind: unbind(x, y) = bind(x, y)
    """
    return bind(bound, key, length)


def bundle2(a: List[Trit], b: List[Trit], length: int = VSA_DIM) -> List[Trit]:
    """
    Bundle operation (majority vote of 2 vectors).

    Used for: superposition, set union
    """
    result = []
    for i in range(length):
        ai, bi = a[i], b[i]
        if ai == TRIT_ZERO:
            result.append(bi)
        elif bi == TRIT_ZERO:
            result.append(ai)
        else:
            # Both non-zero: determine majority
            sum_val = ai + bi
            if sum_val > 0:
                result.append(TRIT_POS)
            elif sum_val < 0:
                result.append(TRIT_NEG)
            else:
                result.append(TRIT_ZERO)
    return result


def bundle3(a: List[Trit], b: List[Trit], c: List[Trit],
             length: int = VSA_DIM) -> List[Trit]:
    """
    Bundle operation (majority vote of 3 vectors).

    Used for: robust superposition, noise reduction
    """
    result = []
    for i in range(length):
        ai, bi, ci = a[i], b[i], c[i]
        sum_val = ai + bi + ci
        if sum_val > 0:
            result.append(TRIT_POS)
        elif sum_val < 0:
            result.append(TRIT_NEG)
        else:
            result.append(TRIT_ZERO)
    return result


def permute(v: List[Trit], length: int = VSA_DIM, shift: int = 1) -> List[Trit]:
    """
    Circular shift of hypervector.

    Used for: sequence encoding, position tagging
    """
    shift = shift % length
    result = [v[(i - shift) % length] for i in range(length)]
    return result


def cosine_similarity(a: List[Trit], b: List[Trit], length: int = VSA_DIM) -> float:
    """Cosine similarity: (a·b) / (||a|| * ||b||)."""
    dot = sum(a[i] * b[i] for i in range(length))
    norm_a = math.sqrt(sum(1 for i in range(length) if a[i] != TRIT_ZERO))
    norm_b = math.sqrt(sum(1 for i in range(length) if b[i] != TRIT_ZERO))
    if norm_a == 0.0 or norm_b == 0.0:
        return 0.0
    return dot / (norm_a * norm_b)


def hamming_distance(a: List[Trit], b: List[Trit], length: int = VSA_DIM) -> int:
    """Count positions where a[i] != b[i]."""
    return sum(1 for i in range(length) if a[i] != b[i])


# ============================================================================
# VSA Analogy Reasoner
# ============================================================================

@dataclass
class Entity:
    """An entity with a name and hypervector representation."""
    name: str
    hypervector: List[Trit]


class VSAAnalogyReasoner:
    """
    VSA-based analogy reasoning system.

    Solves analogies of the form: A:B :: C:?
    Using the property: bind(A, B) unbind(C) ≈ D
    """

    def __init__(self, dim: int = VSA_DIM):
        self.dim = dim
        self.entities: Dict[str, Entity] = {}
        self._initialize_entities()

    def _initialize_entities(self):
        """Initialize entity hypervectors for analogy tasks."""
        # Semantic analogies
        entity_seeds = {
            "king": 1001, "man": 1002, "queen": 1003, "woman": 1004,
            "dog": 2001, "puppy": 2002, "cat": 2003, "kitten": 2004,
            "paris": 3001, "france": 3002, "tokyo": 3003, "japan": 3004,
            "berlin": 3005, "germany": 3006,
            # Programming analogies
            "code": 4001, "compile": 4002, "binary": 4003,
            "source": 4004, "executable": 4005,
            # T27 specific
            "trit": 5001, "ternary": 5002, "bit": 5003, "binary": 5004,
            "phi": 5005, "golden_ratio": 5006,
        }

        for name, seed in entity_seeds.items():
            self.entities[name] = Entity(name, generate_random_hv(seed, self.dim))

    def encode_entity(self, name: str) -> List[Trit]:
        """Get or create entity hypervector."""
        if name not in self.entities:
            seed = hash(name) % 10000
            self.entities[name] = Entity(name, generate_random_hv(seed, self.dim))
        return self.entities[name].hypervector

    def solve_analogy(self, a: str, b: str, c: str,
                     candidates: List[str]) -> Tuple[str, float, Dict[str, float]]:
        """
        Solve analogy A:B :: C:?

        Algorithm:
        1. Compute bound = bind(A, B)
        2. Compute query = unbind(bound, C)
        3. Find candidate with highest similarity to query

        Returns: (best_candidate, similarity, all_similarities)
        """
        a_hv = self.encode_entity(a)
        b_hv = self.encode_entity(b)
        c_hv = self.encode_entity(c)

        # bind(A, B) captures the relationship
        bound = bind(a_hv, b_hv, self.dim)

        # unbind with C to find D
        query = unbind(bound, c_hv, self.dim)

        # Compare with candidates
        similarities = {}
        for candidate in candidates:
            cand_hv = self.encode_entity(candidate)
            sim = cosine_similarity(query, cand_hv, self.dim)
            similarities[candidate] = sim

        # Find best match
        best = max(similarities.items(), key=lambda x: x[1])

        return best[0], best[1], similarities

    def bind_self_inverse_check(self, a: str, b: str) -> float:
        """
        Verify bind self-inverse property: bind(a, bind(a, b)) ≈ b

        Returns similarity between original B and recovered B
        """
        a_hv = self.encode_entity(a)
        b_hv = self.encode_entity(b)

        bound = bind(a_hv, b_hv, self.dim)
        recovered = unbind(bound, a_hv, self.dim)

        return cosine_similarity(b_hv, recovered, self.dim)


class VSABundleReasoner:
    """
    VSA bundle reasoning for set-like operations.

    Demonstrates:
    - bundle2 for superposition
    - bundle3 for consensus
    - Individual concept recovery from bundle
    """

    def __init__(self, dim: int = VSA_DIM):
        self.dim = dim
        self.concepts: Dict[str, List[Trit]] = {}

    def encode_concept(self, name: str, seed: Optional[int] = None) -> List[Trit]:
        """Encode a concept to hypervector."""
        if seed is None:
            seed = hash(name) % 10000
        hv = generate_random_hv(seed, self.dim)
        self.concepts[name] = hv
        return hv

    def create_superposition(self, concepts: List[str]) -> List[Trit]:
        """
        Create superposition of concepts using bundle.

        Uses bundle3 for 3+ concepts (robust consensus voting).
        """
        if not concepts:
            return [TRIT_ZERO] * self.dim

        if len(concepts) == 1:
            return self.encode_concept(concepts[0])

        if len(concepts) == 2:
            a = self.encode_concept(concepts[0])
            b = self.encode_concept(concepts[1])
            return bundle2(a, b, self.dim)

        # For 3+ concepts, use bundle3 iteratively
        result = self.encode_concept(concepts[0])
        for i in range(1, len(concepts)):
            c = self.encode_concept(concepts[i])
            result = bundle3(result, result, c, self.dim)

        return result

    def probe_concept(self, bundle_hv: List[Trit], concept: str,
                     threshold: float = 0.5) -> Tuple[bool, float]:
        """
        Probe if a concept is in the bundle.

        Returns: (present, similarity)
        """
        concept_hv = self.encode_concept(concept)
        sim = cosine_similarity(bundle_hv, concept_hv, self.dim)
        return sim >= threshold, sim


class VSASequenceEncoder:
    """
    VSA sequence encoding with position-aware binding.

    Uses permute for position encoding.
    Demonstrates order-sensitive reasoning.
    """

    def __init__(self, dim: int = VSA_DIM):
        self.dim = dim
        self.items: Dict[str, List[Trit]] = {}

    def encode_item(self, item: str, seed: Optional[int] = None) -> List[Trit]:
        """Encode an item to hypervector."""
        if seed is None:
            seed = hash(item) % 10000
        hv = generate_random_hv(seed, self.dim)
        self.items[item] = hv
        return hv

    def encode_sequence(self, items: List[str]) -> List[Trit]:
        """
        Encode sequence with position-aware binding.

        Algorithm: bundle(items[0], permute(items[1], 1), permute(items[2], 2), ...)
        """
        if not items:
            return [TRIT_ZERO] * self.dim

        result = self.encode_item(items[0])

        for i, item in enumerate(items[1:], 1):
            item_hv = self.encode_item(item)
            permuted = permute(item_hv, self.dim, i)
            result = bundle2(result, permuted, self.dim)

        return result

    def probe_position(self, sequence_hv: List[Trit], item: str,
                      position: int) -> float:
        """
        Probe if item is at specific position in sequence.

        Returns similarity between sequence and permuted item at position.
        """
        item_hv = self.encode_item(item)
        permuted = permute(item_hv, self.dim, position)
        return cosine_similarity(sequence_hv, permuted, self.dim)


# ============================================================================
# Main: Example Usage
# ============================================================================

def main():
    """Run VSA analogy and bundle reasoning examples."""
    print("=" * 60)
    print("VSA Analogy Reasoning - Bind/Unbind/Bundle/Permute")
    print("=" * 60)
    print()

    # Example 1: Semantic analogies
    print("-" * 60)
    print("Example 1: Semantic Analogies (king:man :: queen:?)")
    print("-" * 60)

    reasoner = VSAAnalogyReasoner()

    analogies = [
        ("king", "man", "queen", ["woman", "girl", "princess", "female"]),
        ("dog", "puppy", "cat", ["kitten", "puppy", "animal", "pet"]),
        ("paris", "france", "tokyo", ["japan", "china", "asia", "kyoto"]),
        ("paris", "france", "berlin", ["germany", "europe", "munich", "paris"]),
    ]

    for a, b, c, candidates in analogies:
        print(f"\nAnalogy: {a}:{b} :: {c}:?")
        best, sim, all_sims = reasoner.solve_analogy(a, b, c, candidates)
        print(f"  Answer: {best} (similarity: {sim:.3f})")
        print(f"  All candidates: {all_sims}")

    # Example 2: Bind self-inverse property
    print()
    print("-" * 60)
    print("Example 2: Bind Self-Inverse Property")
    print("-" * 60)
    print("Property: bind(A, bind(A, B)) ≈ B")
    print()

    test_pairs = [("king", "queen"), ("paris", "france"), ("code", "compile")]
    for a, b in test_pairs:
        sim = reasoner.bind_self_inverse_check(a, b)
        print(f"  bind({a}, bind({a}, {b})) ≈ {b}: similarity = {sim:.3f}")

    # Example 3: Bundle superposition
    print()
    print("-" * 60)
    print("Example 3: Bundle Superposition (Set-like Reasoning)")
    print("-" * 60)

    bundle_reasoner = VSABundleReasoner()

    # Create superposition of fruits
    fruits = ["apple", "banana", "orange", "grape"]
    fruit_bundle = bundle_reasoner.create_superposition(fruits)

    print(f"\nBundle of: {', '.join(fruits)}")
    print(f"  Non-zero trits: {sum(1 for t in fruit_bundle if t != TRIT_ZERO)}/{VSA_DIM}")

    # Probe individual fruits
    print("\n  Probing individual fruits:")
    for fruit in fruits + ["carrot", "steak"]:
        present, sim = bundle_reasoner.probe_concept(fruit_bundle, fruit)
        status = "✓" if present else "✗"
        print(f"    {status} {fruit}: {sim:.3f}")

    # Example 4: Sequence encoding
    print()
    print("-" * 60)
    print("Example 4: Sequence Encoding (Position-Aware)")
    print("-" * 60)
    print("Using permute for position: bundle(item[0], permute(item[1], 1), ...)")
    print()

    seq_encoder = VSASequenceEncoder()

    sequences = [
        ["breakfast", "lunch", "dinner"],
        ["first", "second", "third", "fourth"],
    ]

    for seq in sequences:
        seq_hv = seq_encoder.encode_sequence(seq)
        print(f"\nSequence: {' → '.join(seq)}")
        print(f"  Encoded (non-zero trits): {sum(1 for t in seq_hv if t != TRIT_ZERO)}/{VSA_DIM}")

        # Probe positions
        print("  Position probes:")
        for item in seq:
            for pos in range(len(seq)):
                sim = seq_encoder.probe_position(seq_hv, item, pos)
                if sim > 0.5:
                    marker = "★" if seq.index(item) == pos else " "
                    print(f"    {marker} {item} at pos {pos}: {sim:.3f}")

    # Summary
    print()
    print("=" * 60)
    print("VSA Properties Demonstrated:")
    print("  1. Bind/Unbind for associative memory")
    print("  2. Self-inverse: bind(A, bind(A, B)) = B")
    print("  3. Bundle for superposition/set reasoning")
    print("  4. Permute for position-aware sequence encoding")
    print("  5. Cosine similarity for nearest-neighbor search")
    print("=" * 60)


if __name__ == "__main__":
    main()
