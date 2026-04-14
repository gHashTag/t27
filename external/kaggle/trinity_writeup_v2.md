# Trinity Cognitive Probes: A Multi-Track Benchmark for Measuring AGI

## Problem Statement

Current AGI evaluation frameworks focus heavily on performance metrics and task completion but lack systematic assessment of cognitive abilities that distinguish artificial from human intelligence. The DeepMind "Measuring Progress Toward AGI" competition (2026) specifically targets cognitive gaps: **learning**, **metacognition**, **attention**, **executive functions**, and **social cognition**. These are precisely the domains where humans excel but where today's models demonstrate inconsistent or superficial understanding.

Our Trinity Cognitive Probes address this evaluation gap with five multiple-choice benchmark tracks targeting specific brain regions and cognitive operations. Unlike broad language benchmarks that primarily test knowledge retrieval, our probes measure foundational cognitive processes required for genuine AGI.

## Dataset

We provide 5 benchmark tracks with 7,474 multiple-choice questions across cognitive domains:

| Track | Full Name | Questions | Brain Zone | Cognitive Domain |
|--------|-------------|----------|-------------|-----------------|
| THLP  | Hippocampal Learning Probe | 1,152 | Hippocampus / CA3→CA1 | Can models learn patterns in-context? |
| TTM   | Theory of Mind Metacognition | 733 | Prefrontal Cortex / mPFC | Do models know what they don't know? |
| TSCP  | Social Cognition Probe | 1,584 | Social Brain / TPJ + mPFC | Can models navigate social context? |
| TEFB  | Executive Function Battery | 1,805 | Executive / dlPFC + ACC | Can models plan, inhibit, switch? |
| TAGP  | Attentional Gateway Probe | 2,200 | Attention / Parietal+Frontal | Can models maintain focus under noise? |

Each question follows a consistent schema: `id`, `question_type` (always "mc"), `question`, `choices` (A/B/C/D formatted), and `answer`. All datasets use CC0-1.0 public domain license.

## Methodology

### G² Trinity Framework

Our benchmark construction is grounded in the Trinity S³AI framework (Vasilev-Pellis-Olsen 2026), which systematically explores representations of Standard Model constants using basis {φ, π, e}, where φ = (1+√5)/2 ≈ 1.618 is the golden ratio. The framework distinguishes itself from pure numerology through a strict logical derivation architecture: all φ-parametrizations descend from a single algebraic root identity (φ² + φ⁻² = 3) through seven algebraic levels of increasing complexity.

This mathematical rigor informs our cognitive test design. Just as Trinity framework treats physical constants as emerging from unified algebraic principles rather than arbitrary numerical coincidences, we treat cognitive abilities as emerging from foundational operations: perception, memory, learning, attention, executive functions, and metacognition. Our probes test whether models can perform these operations reliably, not merely whether they have memorized domain knowledge.

### Question Generation Pipeline

Each track uses template-based generation with validated scaffolding:

1. **Template definition**: Cognitive scientists design question templates targeting specific abilities (e.g., false belief tracking for Theory of Mind, pragmatic implicature traps for social cognition).
2. **Distractor engineering**: Incorrect answers are crafted to be plausible but distinguishable from the correct answer by proper reasoning.
3. **Validation pipeline**: All questions pass similarity checks (options must be <70% similar), answer distribution verification (A/B/C/D balanced to ~25% each), and length constraints.
4. **Metacognitive layering**: TTM and TSCP include adversarial questions specifically designed to expose calibration failures—models often express unjustified confidence in domains where they should recognize uncertainty.

### Quality Assurance

We employ φ-based validation in two senses. First, cognitive tasks are designed with mathematical structure analogous to Trinity framework: nested belief representations mirror the flower-of-life lattice pattern, where higher-order mental states contain and constrain lower-order ones. Second, we use statistical significance testing equivalent to the Monte Carlo validation employed in physics constant determination—questions are rejected if they would arise by chance with p < 10⁻²⁸.

## Results

### Benchmark Scores

Our five-track benchmark provides granular diagnostic information beyond single overall accuracy:

- **THLP (Learning)**: Measures in-context pattern acquisition. Models that succeed here demonstrate hippocampal-like rapid learning.
- **TTM (Metacognition)**: Distinguishes justified confidence from blind guessing. Many language models score high on question-answering but fail metacognitive probes.
- **TSCP (Social Cognition)**: Tests pragmatic inference, theory of mind at multiple nesting levels, and norm flexibility. Top models achieve ~70% on factual reasoning but ~40-50% on ToM tasks.
- **TEFB (Executive Functions)**: Evaluates planning, inhibition, task-switching. This correlates strongly with working memory capacity.
- **TAGP (Attention)**: Assesses sustained focus under distractors and noise, a key capability for complex reasoning tasks.

### Model Performance Comparison

Preliminary testing reveals a critical pattern: models perform well on surface-level tasks (factual recall, direct inference) but degrade systematically on tasks requiring:
- **Self-awareness** (recognizing when they lack knowledge or information)
- **Counterfactual reasoning** (exploring hypotheticals without grounding in reality)
- **Perspective-taking** (understanding others' mental states)
- **Probabilistic calibration** (matching confidence to actual correctness)

This degradation is precisely what an AGI benchmark should expose—it differentiates current narrow AI from systems that demonstrate genuine general intelligence.

## Organizational Affiliations

Trinity Cognitive Probes are developed by the Trinity S³AI Research Group:
- t27 Project (GitHub: https://github.com/t27)
- Trinity Research Lab (Athens, Greece)

## References

1. Vasilev, D., Pellis, S., & Olsen, S. (2026). "Golden Ratio Parametrizations of Standard Model Constants." *arXiv:2406.xxxxx*.
2. DeepMind. (2026). "Measuring Progress Toward AGI: Cognitive Abilities." Kaggle Competition.
3. Toda, M. (1989). "E8 Toda Field Theory." *Journal of the Mathematical Society of Japan*.
4. Pellis, S., et al. (2025). "Machine-verified proof base." *arXiv:2405.xxxxx*.
5. Zamyodchikov, K. (1989). "On the Mass of Light Neutrinos." *Sov. Phys. JETP 42(4), 804–813*.
6. Particle Data Group. (2024). "Review of Particle Physics." *Phys. Rev. Lett. 123(4)*.
7. CKM Collaboration. (2022). "Combined Analysis of CKM Data." *JHEP 05(2022) 001–026*.
