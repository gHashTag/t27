#!/usr/bin/env python3
"""
Example 1: Medical Diagnosis Pipeline
=====================================

Composition: CNN → VSA Encoding → AR Reasoning → XAI Explanation

This example demonstrates a complete medical diagnosis pipeline where:
1. A CNN extracts features from medical images
2. Features are encoded to ternary hypervectors (VSA)
3. AR performs bounded reasoning with step limit
4. XAI generates explainable output

Author: T27 Trinity Ternary Project
License: Apache 2.0
"""

from dataclasses import dataclass
from typing import List, Tuple, Optional
import math


# ============================================================================
# T27 Ternary Types (from specs/base/types.t27)
# ============================================================================

TRIT_NEG = -1
TRIT_ZERO = 0
TRIT_POS = 1

Trit = int  # Type alias for ternary value


# ============================================================================
# VSA Operations (from specs/vsa/ops.t27)
# ============================================================================

VSA_DIM = 1024
SIM_COSINE = 0
SIM_HAMMING = 1
SIM_DOT = 2


def to_trits(vector: List[float], dim: int = VSA_DIM) -> List[Trit]:
    """Convert float vector to ternary hypervector."""
    trits = []
    for v in vector[:dim]:
        if v > 0.33:
            trits.append(TRIT_POS)
        elif v < -0.33:
            trits.append(TRIT_NEG)
        else:
            trits.append(TRIT_ZERO)
    # Pad if needed
    while len(trits) < dim:
        trits.append(TRIT_ZERO)
    return trits[:dim]


def dot_product(a: List[Trit], b: List[Trit], length: int) -> float:
    """Compute dot product Σ a[i] * b[i]."""
    acc = 0
    for i in range(length):
        acc += a[i] * b[i]
    return float(acc)


def vector_norm(v: List[Trit], length: int) -> float:
    """Compute L2 norm: sqrt(Σ v[i]²)."""
    nonzero = sum(1 for i in range(length) if v[i] != TRIT_ZERO)
    return math.sqrt(nonzero)


def cosine_similarity(a: List[Trit], b: List[Trit], length: int) -> float:
    """Cosine similarity: (a·b) / (||a|| * ||b||)."""
    dot = dot_product(a, b, length)
    norm_a = vector_norm(a, length)
    norm_b = vector_norm(b, length)
    if norm_a == 0.0 or norm_b == 0.0:
        return 0.0
    return dot / (norm_a * norm_b)


def similarity(a: List[Trit], b: List[Trit], length: int, metric: int = SIM_COSINE) -> float:
    """Compute similarity between two hypervectors."""
    if metric == SIM_COSINE:
        return cosine_similarity(a, b, length)
    elif metric == SIM_HAMMING:
        distance = sum(1 for i in range(length) if a[i] != b[i])
        return 1.0 - (distance / length)
    return dot_product(a, b, length)


# ============================================================================
# AR Operations (from specs/ar/*.t27)
# ============================================================================

MAX_STEPS = 10
MIN_QUALITY = 0.7


@dataclass
class Fact:
    """A logical fact for AR reasoning."""
    predicate: str
    value: str
    confidence: float = 1.0


@dataclass
class Rule:
    """A reasoning rule."""
    if_facts: List[Fact]
    then_conclusion: str


@dataclass
class Step:
    """A single reasoning step."""
    step_number: int
    action: str
    premise: str
    conclusion: str


@dataclass
class Conclusion:
    """The result of AR reasoning."""
    class_name: str
    confidence: float
    trace: List[Step]
    steps_used: int


def forward_chain(facts: List[Fact], rules: List[Rule],
                   max_steps: int = MAX_STEPS) -> Conclusion:
    """
    Forward-chaining AR with step limit enforcement.

    This implements bounded rationality - stops after MAX_STEPS
    to prevent infinite chains and ensure explainability.
    """
    trace: List[Step] = []
    steps = 0
    conclusion = "UNKNOWN"

    # Simple forward chain for demonstration
    for rule in rules:
        if steps >= max_steps:
            break

        # Check if all rule conditions are satisfied
        if_facts_satisfied = all(
            any(f.predicate == r_f.predicate and f.value == r_f.value
                for f in facts)
            for r_f in rule.if_facts
        )

        if if_facts_satisfied:
            steps += 1
            trace.append(Step(
                step_number=steps,
                action="apply_rule",
                premise=str([f"{f.predicate}={f.value}" for f in rule.if_facts]),
                conclusion=rule.then_conclusion
            ))
            conclusion = rule.then_conclusion

    # Calculate confidence based on step efficiency
    confidence = MIN_QUALITY if steps > 0 else 0.0
    if steps <= max_steps // 2:
        confidence = 0.9
    elif steps <= max_steps * 0.75:
        confidence = 0.8

    return Conclusion(
        class_name=conclusion,
        confidence=confidence,
        trace=trace,
        steps_used=steps
    )


# ============================================================================
# XAI Module (from specs/ar/explainability.t27)
# ============================================================================

def generate_explanation(trace: List[Step], max_steps: int = MAX_STEPS) -> str:
    """
    Generate explanation with step limit enforcement.

    Explanation is bounded to ≤10 steps per CLARA requirements.
    """
    if not trace:
        return "No reasoning trace available."

    if len(trace) > max_steps:
        return f"Error: Explanation exceeds {max_steps} steps ({len(trace)} steps)."

    lines = ["Reasoning Trace:"]
    for step in trace:
        lines.append(f"  Step {step.step_number}: {step.action}")
        lines.append(f"    Premise: {step.premise}")
        lines.append(f"    Conclusion: {step.conclusion}")

    lines.append(f"\nTotal steps: {len(trace)} (≤{max_steps} limit)")
    return "\n".join(lines)


# ============================================================================
# Medical Diagnosis Pipeline
# ============================================================================

@dataclass
class MedicalCase:
    """A medical case from memory."""
    case_id: str
    diagnosis: str
    features_hv: List[Trit]  # Pre-encoded hypervector


class MedicalDiagnosisSystem:
    """
    Complete medical diagnosis pipeline combining ML, VSA, AR, and XAI.
    """

    def __init__(self):
        # Simulated CNN weights (in real system, these would be trained)
        self.cnn_weights = [0.5, -0.2, 0.8, ...]  # Placeholder

        # Load pre-encoded medical case hypervectors
        self.case_memory: List[MedicalCase] = self._load_case_memory()

        # Load diagnostic rules
        self.diagnostic_rules = self._load_diagnostic_rules()

    def _load_case_memory(self) -> List[MedicalCase]:
        """Load pre-encoded medical cases for VSA similarity search."""
        # In real system, these would be pre-encoded from training data
        return [
            MedicalCase("case_001", "pneumonia", self._generate_hypervector(1)),
            MedicalCase("case_002", "bronchitis", self._generate_hypervector(2)),
            MedicalCase("case_003", "healthy", self._generate_hypervector(3)),
            MedicalCase("case_004", "tuberculosis", self._generate_hypervector(4)),
            MedicalCase("case_005", "covid_19", self._generate_hypervector(5)),
        ]

    def _generate_hypervector(self, seed: int) -> List[Trit]:
        """Generate a deterministic hypervector for simulation."""
        import random
        random.seed(seed)
        return [random.choice([TRIT_NEG, TRIT_ZERO, TRIT_POS])
                for _ in range(VSA_DIM)]

    def _load_diagnostic_rules(self) -> List[Rule]:
        """Load AR diagnostic rules."""
        return [
            Rule(
                if_facts=[Fact("symptom", "fever"), Fact("symptom", "cough")],
                then_conclusion="respiratory_infection"
            ),
            Rule(
                if_facts=[Fact("finding", "consolidation"),
                         Fact("finding", "infiltrates")],
                then_conclusion="pneumonia"
            ),
            Rule(
                if_facts=[Fact("symptom", "dry_cough"),
                         Fact("symptom", "fatigue")],
                then_conclusion="bronchitis"
            ),
            Rule(
                if_facts=[Fact("finding", "normal"), Fact("symptom", "none")],
                then_conclusion="healthy"
            ),
        ]

    def extract_features(self, image) -> List[float]:
        """
        Step 1: CNN feature extraction.

        In real system, this would use a trained CNN model.
        """
        # Simulated feature extraction
        # Returns a 1024-dimensional feature vector
        import random
        random.seed(hash(str(image)))  # Deterministic for demo
        return [random.uniform(-1.0, 1.0) for _ in range(VSA_DIM)]

    def encode_to_trits(self, features: List[float]) -> List[Trit]:
        """Step 2: VSA encoding (continuous → ternary)."""
        return to_trits(features, VSA_DIM)

    def retrieve_similar_cases(self, query_hv: List[Trit],
                               top_k: int = 3,
                               threshold: float = 0.5) -> List[MedicalCase]:
        """
        Step 3: VSA similarity search over case memory.

        Uses cosine similarity to find similar medical cases.
        """
        results = []
        for case in self.case_memory:
            sim = similarity(query_hv, case.features_hv, VSA_DIM, SIM_COSINE)
            if sim >= threshold:
                results.append((sim, case))

        # Sort by similarity and return top-k
        results.sort(reverse=True, key=lambda x: x[0])
        return [case for _, case in results[:top_k]]

    def diagnose(self, image, symptoms: List[str]) -> dict:
        """
        Complete diagnosis pipeline.

        Pipeline:
        1. CNN extracts features
        2. VSA encodes to ternary hypervectors
        3. Retrieve similar cases via similarity search
        4. AR performs bounded reasoning (≤10 steps)
        5. XAI generates explanation (≤10 steps)

        Returns:
            Dict with diagnosis, confidence, explanation, and similar cases
        """
        # Step 1: Feature extraction
        features = self.extract_features(image)
        print(f"[ML] Extracted {len(features)} features from image")

        # Step 2: VSA encoding
        hv = self.encode_to_trits(features)
        print(f"[VSA] Encoded to {VSA_DIM}-dim ternary hypervector")

        # Step 3: Retrieve similar cases
        similar_cases = self.retrieve_similar_cases(hv, top_k=3)
        print(f"[VSA] Retrieved {len(similar_cases)} similar cases")

        # Step 4: AR reasoning
        facts = [Fact("symptom", s) for s in symptoms]
        for case in similar_cases:
            facts.append(Fact("similar_case", case.diagnosis))

        conclusion = forward_chain(facts, self.diagnostic_rules, MAX_STEPS)
        print(f"[AR] Diagnosis: {conclusion.class_name} (confidence: {conclusion.confidence:.2f})")
        print(f"[AR] Reasoning used {conclusion.steps_used}/{MAX_STEPS} steps")

        # Step 5: XAI explanation
        explanation = generate_explanation(conclusion.trace, MAX_STEPS)
        print(f"[XAI] Generated explanation ({len(conclusion.trace)} steps)")

        return {
            "diagnosis": conclusion.class_name,
            "confidence": conclusion.confidence,
            "explanation": explanation,
            "similar_cases": [{"id": c.case_id, "diagnosis": c.diagnosis}
                            for c in similar_cases],
            "steps_used": conclusion.steps_used,
            "step_limit_enforced": conclusion.steps_used <= MAX_STEPS
        }


# ============================================================================
# Main: Example Usage
# ============================================================================

def main():
    """Run the medical diagnosis example."""
    print("=" * 60)
    print("Medical Diagnosis Pipeline - ML + VSA + AR + XAI")
    print("=" * 60)
    print()

    # Initialize the system
    system = MedicalDiagnosisSystem()

    # Simulate a medical image
    patient_image = "chest_xray_patient_001.jpg"
    patient_symptoms = ["fever", "cough", "shortness_of_breath"]

    print(f"Patient: {patient_image}")
    print(f"Symptoms: {', '.join(patient_symptoms)}")
    print()

    # Run diagnosis
    result = system.diagnise(patient_image, patient_symptoms)

    print()
    print("-" * 60)
    print("DIAGNOSIS RESULT")
    print("-" * 60)
    print(f"Diagnosis: {result['diagnosis']}")
    print(f"Confidence: {result['confidence']:.2f}")
    print()
    print("Explanation:")
    print(result['explanation'])
    print()
    print("Similar Cases:")
    for case in result['similar_cases']:
        print(f"  - {case['id']}: {case['diagnosis']}")
    print()
    print(f"Step limit enforced: {result['step_limit_enforced']}")
    print("=" * 60)


if __name__ == "__main__":
    main()
