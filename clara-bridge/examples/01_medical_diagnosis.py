# SPDX-License-Identifier: Apache-2.0
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions
# and limitations under the License.
#

#!/usr/bin/env python3
"""
Trinity CLARA - Example 01: Medical Diagnosis with Ternary Logic

This example demonstrates:
1. Ternary K3 reasoning (True, Unknown, False)
2. Bounded proof traces (≤10 steps)
3. Explainable reasoning with proof trace output

Usage:
    python 01_medical_diagnosis.py
"""

from typing import List, Tuple
from enum import Enum


class K3Value(Enum):
    """Kleene K3 ternary logic values."""
    K_TRUE = "T"
    K_UNKNOWN = "U"
    K_FALSE = "F"


class TernaryReasoner:
    """Ternary reasoner using Kleene K3 semantics."""

    def __init__(self):
        self.proof_trace: List[str] = []
        self.max_steps = 10

    def k3_and(self, a: K3Value, b: K3Value) -> K3Value:
        """K3 AND operation."""
        if a == K3Value.K_FALSE or b == K3Value.K_FALSE:
            result = K3Value.K_FALSE
        elif a == K3Value.K_UNKNOWN or b == K3Value.K_UNKNOWN:
            result = K3Value.K_UNKNOWN
        else:
            result = K3Value.K_TRUE
        self.proof_trace.append(f"k3_and({a.value}, {b.value}) = {result.value}")
        return result

    def k3_or(self, a: K3Value, b: K3Value) -> K3Value:
        """K3 OR operation."""
        if a == K3Value.K_TRUE or b == K3Value.K_TRUE:
            result = K3Value.K_TRUE
        elif a == K3Value.K_UNKNOWN or b == K3Value.K_UNKNOWN:
            result = K3Value.K_UNKNOWN
        else:
            result = K3Value.K_FALSE
        self.proof_trace.append(f"k3_or({a.value}, {b.value}) = {result.value}")
        return result

    def k3_not(self, a: K3Value) -> K3Value:
        """K3 NOT operation."""
        if a == K3Value.K_TRUE:
            result = K3Value.K_FALSE
        elif a == K3Value.K_FALSE:
            result = K3Value.K_TRUE
        else:
            result = K3Value.K_UNKNOWN
        self.proof_trace.append(f"k3_not({a.value}) = {result.value}")
        return result

    def is_valid(self) -> Tuple[bool, str]:
        """Check if proof trace is within bounds."""
        if len(self.proof_trace) > self.max_steps:
            return False, f"Proof trace exceeded {self.max_steps} steps"
        return True, f"Valid: {len(self.proof_trace)} steps (≤{self.max_steps})"


def medical_diagnosis_example():
    """
    Medical diagnosis example using ternary reasoning.

    Scenario:
    - Patient has symptoms S1 (fever), S2 (cough), S3 (headache)
    - Rules: R1: (S1 ∧ S2) → D1 (flu)
             R2: (D1 ∨ S3) → D2 (rest needed)
    - Goal: Determine diagnosis and treatment recommendations
    """

    reasoner = TernaryReasoner()

    # Input symptoms (some may be unknown)
    s1 = K3Value.K_TRUE  # fever present
    s2 = K3Value.K_TRUE  # cough present
    s3 = K3Value.K_UNKNOWN  # headache status unknown

    print("=== Medical Diagnosis with Ternary K3 ====\n")
    print(f"Symptoms: fever={s1.value}, cough={s2.value}, headache={s3.value}\n")

    # Rule 1: (S1 ∧ S2) → D1 (flu diagnosis)
    # Apply AND to symptoms
    s1_and_s2 = reasoner.k3_and(s1, s2)
    # If both true, flu is diagnosed
    d1 = s1_and_s2
    print(f"Rule 1: (S1 ∧ S2) → D1")
    print(f"  S1 ∧ S2 = {s1_and_s2.value} → Flu diagnosis: {d1.value}\n")

    # Rule 2: (D1 ∨ S3) → D2 (rest needed)
    # Apply OR to diagnosis and symptom
    d1_or_s3 = reasoner.k3_or(d1, s3)
    d2 = d1_or_s3
    print(f"Rule 2: (D1 ∨ S3) → D2")
    print(f"  D1 ∨ S3 = {d1_or_s3.value} → Rest needed: {d2.value}\n")

    # Verify proof trace
    is_valid, message = reasoner.is_valid()
    print("=== Proof Trace ===")
    for i, step in enumerate(reasoner.proof_trace, 1):
        print(f"{i}. {step}")
    print(f"\nValidation: {message}")

    return d1, d2, is_valid


if __name__ == "__main__":
    d1, d2, is_valid = medical_diagnosis_example()

    print("\n=== Summary ===")
    print(f"Flu diagnosis (D1): {d1.value}")
    print(f"Rest needed (D2): {d2.value}")
    print(f"Proof trace valid: {is_valid}")
    print(f"\nφ² + 1/φ² = 3 | TRINITY")
