# DARPA CLARA Preparation Plan — Trinity S³AI

## Context

**Why this change is being made:** DARPA CLARA (PA-25-07-02) seeks compositional ML+AR systems with polynomial-time guarantees. Trinity/T27 possesses a unique competitive advantage: native ternary logic (Trit {-1,0,+1}) is isomorphic to Kleene's 3-valued logic (True/Unknown/False), enabling hardware-accelerated automated reasoning.

**Deadline:** April 17, 2026, 16:00 ET (Amendment 1) — 13 days remaining
**Budget:** Up to $2M for Phase 1 + Phase 2 (24 months)
**Critical Blocker:** US entity requirement — must resolve immediately

---

## Critical Files Reference

| File | Purpose |
|------|---------|
| `specs/base/types.t27` | Trit enum {-1,0,+1} = Kleene K3 isomorphism foundation |
| `specs/vsa/ops.t27` | VSA bind/unlink = symbolic reasoning primitives |
| `specs/math/constants.t27` | φ² + φ⁻² = 3 sacred identity |
| `architecture/graph.tri` | Module dependency graph (tiers 0-7) |
| `docs/nona-02-organism/NUMERIC-STANDARD-001.md` | GoldenFloat16 = Bayesian ML component |
| `specs/nn/attention.t27` | Sacred attention (d_k^(-φ³), φ-RoPE) |

---

## Recommended Approach: AR Specs First

**Answer to user's question:** Start with `specs/ar/` creation, not the proposal document.

**Rationale:**
1. AR specs provide the technical foundation for the proposal's scientific argument
2. The Kleene-Trit isomorphism proof is the central differentiator
3. Proposal cannot be written without concrete technical deliverables
4. AR specs demonstrate actual capability (TA1 metric), not just promises

---

## Implementation Plan

### Phase 0: Eligibility Resolution (Day 0, <1 hour)
**Blocking issue:** CLARA FAQ states "Only US entities are eligible"

**Action:**
```bash
# Email to CLARA@darpa.mil
Subject: CLARA PA-25-07-02 Eligibility Inquiry - Non-US Entity

Body:
- Describe Trinity S³AI project
- Ask about partnership options with US academic institution
- Request clarification on eligibility for international collaboration
```

**Pivot strategy:** If US-entity blocker is hard, position the proposal as **"architecture ready, looking for US PI"** — we complete the technical foundation (AR specs, isomorphism proof, neuroanatomy) regardless of final submission eligibility. This creates a "ready-to-deploy" package that strengthens any partnership discussion.

**Deliverable:** Email sent + response documented

---

### Phase 1: Create `specs/ar/` Domain (Days 1-3)

**New directory structure:**
```
specs/ar/
├── ternary_logic.t27      # Kleene K3 via Trit arithmetic
├── rules_engine.t27       # Forward/backward chaining
├── proof_trace.t27        # ≤10 step unfolding explanations
├── explainability.t27     # XAI interface
├── restraint.t27          # Bounded rationality (Trit=0)
├── composition.t27        # ML+AR composition patterns
└── BRAIN_MAP.md           # Neuroanatomical taxonomy
```

**Spec 1: `specs/ar/ternary_logic.t27`** (Core scientific contribution)

> **CRITICAL:** Must reference `specs/base/types.t27` and `specs/base/ops.t27` — do NOT duplicate Trit operations. Use `use base::types` and `use base::ops` to ensure single source of truth and prevent divergence.

```t27
// Ternary Logic as Kleene K3
// Isomorphism: Trit {-1, 0, +1} ≅ Kleene {False, Unknown, True}

module TernaryLogic {
    use base::types;
    use base::ops;  // CRITICAL: reference existing ops, don't duplicate

    // Kleene's Strong Three-Valued Logic
    // Truth values: K_FALSE = -1, K_UNKNOWN = 0, K_TRUE = +1
    const K_FALSE : Trit = .neg;
    const K_UNKNOWN : Trit = .zero;
    const K_TRUE : Trit = .pos;

    // Kleene conjunction (minimum)
    fn k3_and(a: Trit, b: Trit) -> Trit {
        return trit_min(a, b);
    }

    // Kleene disjunction (maximum)
    fn k3_or(a: Trit, b: Trit) -> Trit {
        return trit_max(a, b);
    }

    // Kleene implication (→): ¬a ∨ b
    fn k3_implies(a: Trit, b: Trit) -> Trit {
        return trit_max(trit_negate(a), b);
    }

    // Kleene equivalence (↔): (a→b) ∧ (b→a)
    fn k3_equiv(a: Trit, b: Trit) -> Trit {
        const ab = k3_implies(a, b);
        const ba = k3_implies(b, a);
        return trit_min(ab, ba);
    }

    // Forward chaining: from facts and rules, derive conclusions
    // Rule: IF p THEN q (p → q). Given p is true, derive q.
    fn forward_chain(rule_antecedent: Trit, rule_consequent: Trit, fact: Trit) -> Trit {
        // If fact matches antecedent and rule is true, return consequent
        const fact_matches = trit_eq(fact, rule_antecedent);
        return k3_and(fact_matches, rule_consequent);
    }

    // Backward chaining: from goal, search for supporting rules
    fn backward_chain(goal: Trit, rules: []Rule) -> Trit {
        // Simplified: return K_UNKNOWN if no rule supports goal
        var result : Trit = K_UNKNOWN;
        for (rules) |rule| {
            if (trit_eq(rule.consequent, goal) == K_TRUE) {
                result = trit_max(result, rule.antecedent);
            }
        }
        return result;
    }

    // Restraint: bounded rationality via Trit=0
    fn is_restraint(t: Trit) -> bool {
        return t == K_UNKNOWN;
    }

    // Proof: Trit-K3 isomorphism
    // 1. Bijection: f(Trit.neg) = K_FALSE, f(Trit.zero) = K_UNKNOWN, f(Trit.pos) = K_TRUE
    // 2. Homomorphism: trit_and ≅ k3_and, trit_or ≅ k3_or, trit_not ≅ k3_not
    // 3. Preserves truth structure: ordering, identity, negation

    test k3_conjunction_truth_table
        // Verify Kleene AND matches trit_min
        given a = [K_FALSE, K_UNKNOWN, K_TRUE]
        and   b = [K_FALSE, K_UNKNOWN, K_TRUE]
        for each pair (ai, bj):
            when result = k3_and(ai, bj)
            then result == trit_min(ai, bj)

    test k3_disjunction_truth_table
        // Verify Kleene OR matches trit_max
        given a = [K_FALSE, K_UNKNOWN, K_TRUE]
        and   b = [K_FALSE, K_UNKNOWN, K_TRUE]
        for each pair (ai, bj):
            when result = k3_or(ai, bj)
            then result == trit_max(ai, bj)

    test k3_implication_when_antecedent_true
        // If p=true, p→q = q
        given p = K_TRUE and q = K_FALSE
        when result = k3_implies(p, q)
        then result == q

    test k3_implication_when_antecedent_false
        // If p=false, p→q = true (ex falso quodlibet)
        given p = K_FALSE and q = K_FALSE
        when result = k3_implies(p, q)
        then result == K_TRUE

    test k3_equivalence_reflexive
        // p ≡ p = true for all p
        for t in [K_FALSE, K_UNKNOWN, K_TRUE]:
            assert k3_equiv(t, t) == K_TRUE

    test forward_chain_modus_ponens
        // Given p→q and p, derive q
        given rule = {antecedent: K_TRUE, consequent: K_TRUE}
        and   fact = K_TRUE
        when result = forward_chain(rule.antecedent, rule.consequent, fact)
        then result == K_TRUE

    test restraint_returns_true_for_unknown
        given t = K_UNKNOWN
        when result = is_restraint(t)
        then result == true

    invariant k3_and_commutative
        // Kleene AND is commutative
        for all a, b in {K_FALSE, K_UNKNOWN, K_TRUE}:
            assert k3_and(a, b) == k3_and(b, a)

    invariant k3_or_commutative
        // Kleene OR is commutative
        for all a, b in {K_FALSE, K_UNKNOWN, K_TRUE}:
            assert k3_or(a, b) == k3_or(b, a)

    invariant k3_and_associative
        // Kleene AND is associative
        for all a, b, c:
            assert k3_and(k3_and(a, b), c) == k3_and(a, k3_and(b, c))

    invariant k3_or_associative
        // Kleene OR is associative
        for all a, b, c:
            assert k3_or(k3_or(a, b), c) == k3_or(a, k3_or(b, c))

    invariant k3_and_identity
        // true AND x = x
        for all x:
            assert k3_and(K_TRUE, x) == x

    invariant k3_or_identity
        // false OR x = x
        for all x:
            assert k3_or(K_FALSE, x) == x

    invariant k3_double_negation
        // ¬¬x = x
        for all x:
            assert trit_not(trit_not(x)) == x

    invariant trit_k3_isomorphism_preserves_ordering
        // K_FALSE < K_UNKNOWN < K_TRUE preserved
        assert trit_compare(K_FALSE, K_UNKNOWN) == -1
        assert trit_compare(K_UNKNOWN, K_TRUE) == -1

    invariant trit_k3_isomorphism_preserves_negation
        // ¬K_FALSE = K_TRUE, ¬K_TRUE = K_FALSE, ¬K_UNKNOWN = K_UNKNOWN
        assert trit_negate(K_FALSE) == K_TRUE
        assert trit_negate(K_TRUE) == K_FALSE
        assert trit_negate(K_UNKNOWN) == K_UNKNOWN

    bench k3_and_latency
        measure: cycles for single k3_and operation
        target: < 10 cycles (hardware Trit AND)

    bench k3_or_latency
        measure: cycles for single k3_or operation
        target: < 10 cycles (hardware Trit OR)

    bench k3_implies_latency
        measure: cycles for single k3_implies operation
        target: < 20 cycles (NOT + OR)
}

// Rule type for chaining
struct Rule {
    antecedent : Trit,
    consequent : Trit,
}
```

**Remaining specs** (outline):
- `rules_engine.t27`: Horn clause execution, resolution principle
- `proof_trace.t27`: ≤10 step unfolding, explanation tree
- `explainability.t27`: XAI interface mapping to Trit states
- `restraint.t27`: Bounded rationality, "don't-care" optimization
- `composition.t27`: ML+AR composition patterns (CNN→K3→Proof)

**Update `architecture/graph.tri`:**
```tri
spec "triar-logic" {
    tier = 2;
    description = "Kleene K3 ternary logic via Trit arithmetic";
    path = "specs/ar/ternary_logic.t27";
    exports = ["K_FALSE", "K_UNKNOWN", "K_TRUE", "k3_and", "k3_or", "k3_implies"];
    deps = ["tritype-base"];
    competency = "AutomatedReasoning";
    status = "done";
}
```

---

### Phase 2: Neuroanatomical Taxonomy (Days 3-5)

**Apply the user's 55-algorithm taxonomy** to create `specs/algo/` structure:

> **Path mapping for Trinity .tri files** — these moves happen in the `trinity` repo, NOT in t27:

| Old Path (trinity/) | New Path (trinity/) | Category |
|---------------------|---------------------|----------|
| `specs/algo/relu.tri` | `specs/algo/hillock/relu.tri` | Activations |
| `specs/algo/sigmoid.tri` | `specs/algo/hillock/sigmoid.tri` | Activations |
| `specs/algo/dense.tri` | `specs/algo/cortex/dense.tri` | Layers |
| `specs/algo/conv2d.tri` | `specs/algo/cortex/conv2d.tri` | Layers |
| `specs/algo/lstm.tri` | `specs/algo/hippocampus/lstm.tri` | RNN/Sequence |
| `specs/algo/multi_head_attn.tri` | `specs/algo/prefrontal/multi_head_attn.tri` | Transformer |
| `specs/algo/sgd.tri` | `specs/algo/synapse/sgd.tri` | Optimizers |
| `specs/algo/cross_entropy.tri` | `specs/algo/dopamine/cross_entropy.tri` | Loss |
| `specs/algo/dqn.tri` | `specs/algo/ganglia/dqn.tri` | RL |
| `specs/algo/mlp.tri` | `specs/algo/pathway/mlp.tri` | Composite |

**Note:** t27 repo only receives `specs/algo/BRAIN_MAP.md` as documentation. The actual `.tri` file moves happen in the separate `trinity` repository.

**Structure in trinity/specs/algo/:**

```
specs/algo/
├── hillock/           # Axon Hillock (Activations: relu, sigmoid, tanh, gelu...)
├── cortex/            # Cerebral Cortex (Layers: dense, conv2d, pooling...)
├── hippocampus/       # Hippocampal Formation (RNN: lstm, gru, attention...)
├── prefrontal/        # Prefrontal Cortex (Transformer: multi_head_attn...)
├── synapse/           # Synaptic Plasticity (Optimizers: sgd, adam...)
├── dopamine/          # Dopaminergic System (Loss: cross_entropy, mse...)
├── ganglia/           # Basal Ganglia (RL: dqn, ppo, sac...)
└── pathway/           # Neural Pathways (Composite: mlp...)
```

**Each file gets:**
```t27
algorithm: relu
module: brain.hillock.threshold
strand_i:
  phi_identity: "φ² + 1/φ² = 3"
  numeric: GF16
  ternary_map: "{-1: inhibit, 0: subthreshold, +1: fire}"
strand_ii:
  brain_region: axon_hillock
  function: threshold_activation
  biological_analog: "All-or-none action potential at ~-55mV"
strand_iii:
  t27_target: "hillock_relu.t27"
  backends: [zig, verilog]
clara:
  family: Neural Networks (NN)
  complexity: O(n)
```

**Execute user's migration script** (if available) or create manually.

---

### Phase 3: Kleene-Trit Isomorphism Proof (Days 5-6)

**Scientific paper section** for proposal:

```markdown
# Theorem: Trit-Kleene Isomorphism

## Statement
The balanced ternary type Trit = {-1, 0, +1} is isomorphic to Kleene's
strong three-valued logic K3 = {False, Unknown, True} under structure
preserving bijection f: Trit → K3.

## Proof
1. **Bijection**: f(-1) = False, f(0) = Unknown, f(+1) = True is bijective.
2. **Homomorphism**: Operations preserved:
   - trit_and ≅ ∧_K3 (conjunction as minimum)
   - trit_or ≅ ∨_K3 (disjunction as maximum)
   - trit_not ≅ ¬_K3 (negation as inversion)
3. **Truth structure**: Ordering preserved: False < Unknown < True.

## Implications for CLARA
- Native hardware AR: TRI-27 ISA performs K3 operations in O(1)
- Restraint = Trit 0 exactly matches CLARA's "bounded rationality"
- Polynomial inference guaranteed: O(n) for n-literal formulas

## References
- Kleene, S. C. (1952). Introduction to Metamathematics.
- LCWS: https://logicalmethods.ai/textbook/many-valued/
```

---

### Phase 4: Composition Patterns (Days 6-8)

**Document ML+AR composition** for CLARA TA2 requirement:

| Pattern | ML Component | AR Component | t27 Implementation |
|---------|-------------|--------------|-------------------|
| CNN + Rules | `specs/algo/cortex/conv2d.tri` | `specs/ar/rules_engine.t27` | conv2d → k3_and → proof_trace |
| MLP + Bayesian | `specs/algo/pathway/mlp.tri` | `specs/numeric/gf16.t27` | mlp + gf16_prob → datalog |
| Transformer + XAI | `specs/algo/prefrontal/multi_head_attn.tri` | `specs/ar/explainability.t27` | attention → ≤10 step unfold |
| RL + Guardrails | `specs/algo/ganglia/ppo_actor.tri` | `specs/ar/restraint.t27` | policy → restraint check → action |

**Create `docs/clara/CLARA-COMPOSITION-PATTERNS.md`** with detailed examples.

---

### Phase 5: Proposal Document (Days 8-11)

**Structure per CLARA solicitation:**

```markdown
# Technical Abstract (1 page)
Trinity S³AI proposes Ternary Automated Reasoning (TAR): a hardware-
accelerated ML+AR composition framework where Kleene's 3-valued logic
executes natively on ternary computing substrate. Key innovation: Trit
{-1,0,+1} ≅ K3 {False,Unknown,True} isomorphism enables O(1) AR operations
on FPGA, with polynomial-time inference guarantees.

# 1. Technical Approach (3 pages)
## 1.1 AR Layer: Kleene-K3 on Trit Arithmetic
- specs/ar/ternary_logic.t27: K3 operations as native Trit arithmetic
- Proof of isomorphism (see Section 2)
- Restraint via Trit=0 (bounded rationality)

## 1.2 ML Components: 55 Algorithms in Neuroanatomical Taxonomy
- specs/algo/hillock/*: Activations (axon hillock threshold)
- specs/algo/cortex/*: Layers (cortical columns)
- specs/algo/ganglia/*: RL (basal ganglia Go/NoGo)

## 1.3 Composition: TA2 Library
- 4 composition patterns (CNN+Rules, MLP+Bayesian, etc.)
- Polynomial complexity proofs (see Section 3)

# 2. Basis for Confidence (2 pages)
- 81 PHI LOOP skills committed with hash seals
- Semantic equivalence proven (#487)
- FPGA prototype: 63 tok/s @ 1W on XC7A100T
- GoldenFloat16: 0.00% gap vs FP32 (BENCH-004b)

# 3. Polynomial-Time Tractability (2 pages)
- Theorem 1: K3 conjunction = trit_min = O(1)
- Theorem 2: Forward chaining = O(n) for n-literal Horn clauses
- Theorem 3: VSA similarity = O(n) SIMD-optimized
- Theorem 4: Attention inference = O(L×H) where L=length, H=hidden

# 4. Verifiability (1 page)
- .t27 → Verilog formal verification pipeline
- Proof traces with ≤10 step unfolding
- 3000+ tests with invariant checking
```

---

### Phase 6: Demo Pipeline (Days 11-12)

**Minimal working example:**

```t27
// CLARA Demo: MNIST + Proof Trace
// Input: 28×28 image → Output: digit + explanation

module ClaraDemo {
    use algo::cortex::conv2d;
    use algo::cortex::dense;
    use algo::hillock::softmax;
    use ar::ternary_logic;
    use ar::proof_trace;

    fn classify_mnist(image: [784]f64) -> Classification {
        // ML forward pass
        const conv1_out = conv2d_forward(image, conv1_weights);
        const dense_out = dense_forward(conv1_out, dense_weights);
        const probs = softmax(dense_out);

        // AR explanation: WHY this digit?
        const proof = generate_proof_trace(
            hypothesis = "digit=7",
            evidence = conv1_out,
            rules = mnist_rules,
            max_steps = 10
        );

        return Classification {
            prediction = argmax(probs),
            confidence = max(probs),
            explanation = proof,
        };
    }
}
```

**Verification:**
```bash
tri gen           # .t27 → .zig
tri test          # verify proof correctness
tri verdict --toxic  # safety check
```

---

## Verification

**End-to-end testing:**
1. `specs/ar/ternary_logic.t27` passes all tests (K3 truth tables)
2. Kleene-Trit isomorphism proof verified by automated checker
3. Composition patterns execute with ≤10 step explanations
4. Demo pipeline produces prediction + proof trace
5. All specs validate with `tri spec validate`

**Success criteria:**
- [ ] Email to CLARA@darpa.mil sent + response received
- [ ] 7 AR specs created in `specs/ar/`
- [ ] Neuroanatomical taxonomy applied to 55 algorithms
- [ ] Kleene-Trit isomorphism proof completed
- [ ] 4 composition patterns documented
- [ ] Proposal document draft complete
- [ ] Demo pipeline executable with proof trace

---

## Files to Modify

| File | Action |
|------|--------|
| `specs/ar/ternary_logic.t27` | CREATE (core K3 spec) |
| `specs/ar/rules_engine.t27` | CREATE (Horn clauses) |
| `specs/ar/proof_trace.t27` | CREATE (≤10 step XAI) |
| `specs/ar/explainability.t27` | CREATE (interface) |
| `specs/ar/restraint.t27` | CREATE (bounded rationality) |
| `specs/ar/composition.t27` | CREATE (ML+AR patterns) |
| `architecture/graph.tri` | MODIFY (add triar-logic spec) |
| `docs/clara/CLARA-COMPOSITION-PATTERNS.md` | CREATE |
| `docs/nona-02-organism/KLEENE-TRIT-ISOMORPHISM.md` | CREATE |
| `specs/algo/BRAIN_MAP.md` | CREATE (neuroanatomy) |

---

## Notes

- **Do NOT touch existing specs** — only add new AR domain
- **Follow .t27 canonical format**: module/fn/test/invariant/bench
- **Use existing Trit operations** from `specs/base/ops.t27`
- **VSA operations** (`specs/vsa/ops.t27`) provide symbolic reasoning primitives
- **GoldenFloat16** (`specs/numeric/gf16.t27`) = Bayesian component
- **FPGA MAC** (`specs/fpga/mac.t27`) = hardware verification target
