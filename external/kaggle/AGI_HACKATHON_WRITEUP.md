# Trinity Cognitive Probes — AGI Hackathon Write-up

## Competition
**Measuring Progress Toward AGI: Cognitive Abilities**
DeepMind x Kaggle
Submission window: March 17 – April 16, 2026
Prize pool: $200,000

## Team: Trinity S³AI (Playra)

## Overview
Trinity Cognitive Probes is a comprehensive benchmark suite targeting five key cognitive domains identified in the DeepMind paper "Measuring Progress Toward AGI":

- **Learning** (THLP) — Pattern induction, belief update, rule learning
- **Metacognition** (TTM) — Confidence calibration, error detection, meta-learning
- **Attention** (TAGP) — Selective filtering, sustained attention, attention shifting
- **Executive Functions** (TEFB) — Multi-step planning, working memory, cognitive flexibility
- **Social Cognition** (TSCP) — Theory of Mind, pragmatic inference, social norms

## Technical Approach

### Data Generation
All questions are generated using the Trinity S³AI t27 specification language, ensuring:
- Consistent quality across tracks
- ASCII-only source code (L3 compliance)
- TDD-validated specifications (L4 compliance)
- Single source of truth for mathematics and numerics (SSOT-MATH compliance)

### Data Format
Each track provides a multiple-choice (MC) dataset with:
- `id`: Unique question identifier
- `question_type`: `mc` (multiple-choice) or `factual`
- `question`: The question text
- `choices`: 4 options formatted as A) ... B) ... C) ... D) ...
- `answer`: Correct letter (A, B, C, or D)

### Benchmark Notebooks
Each track includes an 8-cell benchmark notebook:
1. Install dependencies
2. Import libraries and configuration
3. Download dataset from Kaggle API
4. Load CSV and filter MC rows
5. Define MCAnswer dataclass
6. Inner benchmark (per-question, `store_task=False`)
7. Outer benchmark (aggregate accuracy)
8. Run and submit to leaderboard

## Track-by-Track Summary

### THLP — Hippocampal Learning Probe
- **Brain Zone**: Hippocampus / CA3→CA1
- **Questions**: 19,681 MC questions
- **Cognitive Task**: Pattern completion, error-driven learning
- **Brain Regions**: Hippocampus (pattern storage), Entorhinal Cortex (spatial reasoning)
- **Key Challenge**: Can models learn new patterns from limited examples?
- **Dataset**: [playra/trinity-cognitive-probes-thlp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc)

### TTM — Theory of Mind Metacognition
- **Brain Zone**: Prefrontal Cortex / mPFC
- **Questions**: 733 MC questions (v5: 600 unique questions)
- **Cognitive Task**: Confidence calibration, error detection, meta-learning
- **Brain Regions**: PCC (metacognitive monitoring), dlPFC (executive control)
- **Key Challenge**: Do models know what they don't know?
- **Dataset**: [playra/trinity-cognitive-probes-tmp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc)

### TAGP — Attentional Gateway Probe
- **Brain Zone**: Parietal Cortex + Frontal Eye Fields + Thalamus
- **Questions**: 17,601 MC questions
- **Cognitive Task**: Selective filtering, sustained attention, attention shifting
- **Brain Regions**: Parietal Cortex (spatial attention), Frontal Eye Fields (top-down control), Thalamus (gateway)
- **Key Challenge**: Can models maintain focus under distraction?
- **Dataset**: [playra/trinity-cognitive-probes-tagp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc)

### TEFB — Executive Function Battery
- **Brain Zone**: dlPFC + ACC + OFC
- **Questions**: 21,081 MC questions
- **Cognitive Task**: Multi-step planning, working memory, cognitive flexibility
- **Brain Regions**: dlPFC (working memory, planning), ACC (conflict monitoring), OFC (value-based decision)
- **Key Challenge**: Can models plan, inhibit, and switch strategies?
- **Dataset**: [playra/trinity-cognitive-probes-tefb-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc)

### TSCP — Social Cognition Probe
- **Brain Zone**: TPJ + OFC + mPFC
- **Questions**: 2,839 MC questions (v5: 500 unique questions)
- **Cognitive Task**: Theory of Mind, pragmatic inference, social norms
- **Brain Regions**: TPJ (Theory of Mind), OFC (social value), mPFC (mentalizing)
- **Key Challenge**: Do models understand social context and mental states?
- **Dataset**: [playra/trinity-cognitive-probes-tscp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc)

## Total Dataset Statistics

| Track | Questions | v5 Unique | v2 Total | Brain Zones |
|-------|-----------|-------------|-----------|-------------|
| THLP  | 19,681    | 19,681      | Hippocampus, EC |
| TTM   | 600       | 733         | PCC, dlPFC |
| TAGP  | 17,601    | 17,601      | Parietal, FEF, Thalamus |
| TEFB  | 21,081    | 21,081      | dlPFC, ACC, OFC |
| TSCP  | 500       | 2,839       | TPJ, OFC, mPFC |
| **TOTAL** | **59,682** | **59,882**   | |

## License
All datasets released under **CC0-1.0** (Public Domain)

## Citation
```
Trinity Cognitive Probes — Ternary Hyperdimensional Computing AGI Benchmark
DeepMind x Kaggle "Measuring Progress Toward AGI" Hackathon, 2026
```

## Additional Resources

### Adversarial Examples
For research purposes, adversarial examples are available for TTM and TSCP tracks to test model robustness:
- TTM Adversarial: Challenging confidence calibration
- TSCP Adversarial: Challenging Theory of Mind reasoning

### Notebooks
Each track includes a benchmark notebook on Kaggle:
- [THLP Hippocampal Learning MC](https://www.kaggle.com/code/playra/trinity-thlp-hippocampal-learning-mc-benchmark)
- [TTM Metacognition MC](https://www.kaggle.com/code/playra/trinity-ttm-metacognition-mc-benchmark)
- [TAGP Attentional Gateway MC](https://www.kaggle.com/code/playra/trinity-tagp-attentional-gateway-mc-benchmark)
- [TEFB Executive Function MC](https://www.kaggle.com/code/playra/trinity-tefb-executive-function-mc-benchmark)
- [TSCP Social Cognition MC](https://www.kaggle.com/code/playra/trinity-tscp-social-cognition-mc-benchmark)

## Conclusion
Trinity Cognitive Probes provides a comprehensive evaluation of AGI capabilities across five cognitive domains, with:
- Neuroanatomically grounded benchmark design
- Consistent data format across all tracks
- Reproducible benchmark notebooks
- Open-source public domain datasets

Total: 59,882 multiple-choice questions across 5 cognitive tracks targeting key brain regions and cognitive processes identified as critical for measuring progress toward AGI.
