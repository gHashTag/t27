#!/usr/bin/env python3
"""
Example 2: Legal Document QA with VSA Semantic Memory
======================================================

Composition: Query Encoder → VSA Similarity Search → Retrieval → AR

This example demonstrates question answering over legal documents using:
1. Query encoding to hypervectors
2. VSA similarity search for context retrieval
3. AR reasoning over retrieved context
4. Bounded step limit for explainability

Author: Dmitrii Vasilev (T27 Project)
SPDX-License-Identifier: Apache-2.0
"""

from dataclasses import dataclass
from typing import List, Dict, Tuple
import math


# ============================================================================
# T27 Ternary Types
# ============================================================================

TRIT_NEG = -1
TRIT_ZERO = 0
TRIT_POS = 1

Trit = int

VSA_DIM = 1024
SIM_COSINE = 0


# ============================================================================
# VSA Operations (simplified from specs/vsa/ops.t27)
# ============================================================================

def to_trits(text: str, dim: int = VSA_DIM) -> List[Trit]:
    """Encode text to ternary hypervector (simplified hash-based)."""
    import hashlib
    hash_val = hashlib.sha256(text.encode()).digest()

    trits = []
    for i in range(dim):
        byte_idx = (i // 4) % len(hash_val)
        bit_mask = 1 << (i % 4)
        byte_val = hash_val[byte_idx]

        if byte_val & bit_mask:
            trits.append(TRIT_POS)
        else:
            trits.append(TRIT_NEG)

    return trits


def dot_product(a: List[Trit], b: List[Trit], length: int) -> float:
    """Compute dot product Σ a[i] * b[i]."""
    return sum(a[i] * b[i] for i in range(length))


def vector_norm(v: List[Trit], length: int) -> float:
    """Compute L2 norm: sqrt(Σ v[i]²)."""
    return math.sqrt(sum(1 for i in range(length) if v[i] != TRIT_ZERO))


def cosine_similarity(a: List[Trit], b: List[Trit], length: int) -> float:
    """Cosine similarity: (a·b) / (||a|| * ||b||)."""
    dot = dot_product(a, b, length)
    norm_a = vector_norm(a, length)
    norm_b = vector_norm(b, length)
    if norm_a == 0.0 or norm_b == 0.0:
        return 0.0
    return dot / (norm_a * norm_b)


# ============================================================================
# AR Operations (from specs/ar/datalog_engine.t27)
# ============================================================================

MAX_STEPS = 10


@dataclass
class Fact:
    predicate: str
    value: str


@dataclass
class Rule:
    if_predicate: str
    if_value: str
    then_conclusion: str


@dataclass
class Step:
    step_number: int
    action: str
    premise: str
    conclusion: str


@dataclass
class Answer:
    answer: str
    confidence: float
    trace: List[Step]
    context_sources: List[str]


def forward_chain(facts: List[Fact], rules: List[Rule],
                   max_steps: int = MAX_STEPS) -> Answer:
    """
    Forward-chaining AR with bounded steps.

    Implements bounded rationality - stops after MAX_STEPS.
    """
    trace: List[Step] = []
    steps = 0
    answer = "UNKNOWN"

    knowledge = set(f"{f.predicate}:{f.value}" for f in facts)

    for rule in rules:
        if steps >= max_steps:
            break

        key = f"{rule.if_predicate}:{rule.if_value}"
        if key in knowledge:
            steps += 1
            trace.append(Step(
                step_number=steps,
                action="apply_rule",
                premise=f"{rule.if_predicate}={rule.if_value}",
                conclusion=rule.then_conclusion
            ))
            knowledge.add(f"derived:{rule.then_conclusion}")
            answer = rule.then_conclusion

    confidence = 0.9 if steps > 0 else 0.0
    if steps > max_steps // 2:
        confidence = 0.7

    return Answer(
        answer=answer,
        confidence=confidence,
        trace=trace,
        context_sources=[f.source for f in facts if hasattr(f, 'source')]
    )


# ============================================================================
# Legal Document QA System
# ============================================================================

@dataclass
class LegalDocument:
    doc_id: str
    title: str
    content: str
    category: str
    hypervector: List[Trit]


class LegalQASystem:
    """
    Legal question answering with VSA semantic memory retrieval.
    """

    def __init__(self, similarity_threshold: float = 0.5):
        self.similarity_threshold = similarity_threshold
        self.documents: List[LegalDocument] = self._load_documents()
        self.rules: List[Rule] = self._load_legal_rules()

    def _load_documents(self) -> List[LegalDocument]:
        """Load legal documents and pre-encode hypervectors."""
        docs = [
            LegalDocument(
                doc_id="DOC_001",
                title="Contract Law Basics",
                content="A contract requires offer, acceptance, consideration, and mutual assent to be valid.",
                category="contract",
                hypervector=[]
            ),
            LegalDocument(
                doc_id="DOC_002",
                title="Intellectual Property Rights",
                content="Copyright protects original works of authorship including software code and documentation.",
                category="ip",
                hypervector=[]
            ),
            LegalDocument(
                doc_id="DOC_003",
                title="Open Source Licensing",
                content="Apache 2.0 license provides explicit patent grant and requires attribution for modifications.",
                category="license",
                hypervector=[]
            ),
            LegalDocument(
                doc_id="DOC_004",
                title="Data Privacy Requirements",
                content="Personal data processing requires explicit consent and purpose limitation under GDPR.",
                category="privacy",
                hypervector=[]
            ),
            LegalDocument(
                doc_id="DOC_005",
                title="Liability in Software",
                content="Software is typically provided 'as is' with disclaimers of warranty limiting liability.",
                category="liability",
                hypervector=[]
            ),
        ]

        # Pre-encode hypervectors
        for doc in docs:
            combined = f"{doc.title} {doc.content}"
            doc.hypervector = to_trits(combined, VSA_DIM)

        return docs

    def _load_legal_rules(self) -> List[Rule]:
        """Load legal reasoning rules."""
        return [
            Rule(
                if_predicate="has_offer",
                if_value="yes",
                then_conclusion="contract_formed_pending_acceptance"
            ),
            Rule(
                if_predicate="contract_formed_pending_acceptance",
                if_value="yes",
                then_conclusion="requires_acceptance"
            ),
            Rule(
                if_predicate="license_type",
                if_value="apache_2.0",
                then_conclusion="includes_patent_grant"
            ),
            Rule(
                if_predicate="license_type",
                if_value="mit",
                then_conclusion="implicit_patent_grant"
            ),
            Rule(
                if_predicate="data_type",
                if_value="personal",
                then_conclusion="requires_consent"
            ),
        ]

    def retrieve_context(self, query: str, top_k: int = 3) -> List[Tuple[float, LegalDocument]]:
        """
        Retrieve relevant documents using VSA similarity search.

        Uses cosine similarity over pre-encoded hypervectors.
        """
        query_hv = to_trits(query, VSA_DIM)

        results = []
        for doc in self.documents:
            sim = cosine_similarity(query_hv, doc.hypervector, VSA_DIM)
            if sim >= self.similarity_threshold:
                results.append((sim, doc))

        # Sort by similarity and return top-k
        results.sort(reverse=True, key=lambda x: x[0])
        return results[:top_k]

    def extract_facts(self, documents: List[Tuple[float, LegalDocument]]) -> List[Fact]:
        """Extract facts from retrieved documents."""
        facts = []
        for sim, doc in documents:
            f = Fact("context_source", doc.doc_id)
            f.source = doc.doc_id
            facts.append(f)

            # Extract simple keyword facts
            content_lower = doc.content.lower()
            if "contract" in content_lower:
                f = Fact("document_type", "contract")
                f.source = doc.doc_id
                facts.append(f)
            if "apache" in content_lower:
                f = Fact("license_type", "apache_2.0")
                f.source = doc.doc_id
                facts.append(f)
            if "mit" in content_lower:
                f = Fact("license_type", "mit")
                f.source = doc.doc_id
                facts.append(f)
            if "patent" in content_lower:
                f = Fact("has_patent_grant", "yes")
                f.source = doc.doc_id
                facts.append(f)
            if "personal data" in content_lower or "personal" in content_lower:
                f = Fact("data_type", "personal")
                f.source = doc.doc_id
                facts.append(f)

        return facts

    def answer_question(self, question: str) -> Dict:
        """
        Complete QA pipeline.

        Pipeline:
        1. Encode query to hypervector
        2. Retrieve similar documents via similarity search
        3. Extract facts from retrieved context
        4. AR reasoning with bounded steps (≤10)
        5. Return answer with explanation and sources
        """
        # Step 1-2: Retrieve context
        retrieved = self.retrieve_context(question, top_k=3)

        # Step 3: Extract facts
        facts = self.extract_facts(retrieved)

        # Step 4: AR reasoning
        answer = forward_chain(facts, self.rules, MAX_STEPS)

        # Format result
        return {
            "question": question,
            "answer": answer.answer,
            "confidence": answer.confidence,
            "explanation": self._format_explanation(answer),
            "sources": self._format_sources(retrieved),
            "steps_used": len(answer.trace),
            "step_limit": MAX_STEPS
        }

    def _format_explanation(self, answer: Answer) -> str:
        """Format reasoning explanation."""
        if not answer.trace:
            return "No reasoning steps performed."

        lines = ["Reasoning Trace:"]
        for step in answer.trace:
            lines.append(f"  Step {step.step_number}: {step.action}")
            lines.append(f"    {step.premise} → {step.conclusion}")

        return "\n".join(lines)

    def _format_sources(self, retrieved: List[Tuple[float, LegalDocument]]) -> List[Dict]:
        """Format retrieved sources."""
        return [
            {
                "doc_id": doc.doc_id,
                "title": doc.title,
                "similarity": sim,
                "category": doc.category
            }
            for sim, doc in retrieved
        ]


# ============================================================================
# Main: Example Usage
# ============================================================================

def main():
    """Run the legal QA example."""
    print("=" * 60)
    print("Legal Document QA - VSA Semantic Memory + AR")
    print("=" * 60)
    print()

    # Initialize the system
    qa = LegalQASystem(similarity_threshold=0.4)

    # Example questions
    questions = [
        "What does Apache 2.0 license include?",
        "What is required for a valid contract?",
        "Does open source software have patent protection?",
    ]

    for question in questions:
        print("-" * 60)
        print(f"Q: {question}")
        print()

        result = qa.answer_question(question)

        print(f"A: {result['answer']}")
        print(f"Confidence: {result['confidence']:.2f}")
        print()
        print("Explanation:")
        print(result['explanation'])
        print()
        print("Sources:")
        for source in result['sources']:
            print(f"  - {source['title']} (similarity: {source['similarity']:.3f})")
        print()
        print(f"Reasoning steps: {result['steps_used']}/{result['step_limit']}")
        print()

    print("=" * 60)


if __name__ == "__main__":
    main()
