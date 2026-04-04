# Trinity Cognitive Benchmarks — AGI Hackathon

## Overview

5 multiple-choice benchmark tracks for the Google DeepMind x Kaggle
**"Measuring Progress Toward AGI: Cognitive Abilities"** hackathon.

- **Prize pool:** $200,000
- **Submission window:** March 17 – April 16, 2026
- **Winners announced:** June 1, 2026
- **Competition:** [kaggle.com/competitions/kaggle-measuring-agi](https://www.kaggle.com/competitions/kaggle-measuring-agi)

## Tracks

| Track | Full Name                        | Questions | Brain Zone                   |
|-------|----------------------------------|-----------|------------------------------|
| THLP  | Hippocampal Learning Probe       | 1,152     | Hippocampus / CA3→CA1        |
| TTM   | Theory of Mind Metacognition     | 733       | Prefrontal Cortex / mPFC     |
| TSCP  | Social Cognition Probe           | 1,584     | Social Brain / TPJ + mPFC    |
| TEFB  | Executive Function Battery       | 1,805     | Executive / dlPFC + ACC      |
| TAGP  | Attentional Gateway Probe        | 2,200     | Attention / Parietal+Frontal |

**Total: 7,474 MC questions across 5 cognitive domains**

## Datasets (Kaggle)

| Track | Dataset ID                                   | Link |
|-------|----------------------------------------------|------|
| THLP  | `playra/trinity-cognitive-probes-thlp-mc`    | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc) |
| TTM   | `playra/trinity-cognitive-probes-tmp-mc`     | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc) |
| TSCP  | `playra/trinity-cognitive-probes-tscp-mc`    | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc) |
| TEFB  | `playra/trinity-cognitive-probes-tefb-mc`    | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc) |
| TAGP  | `playra/trinity-cognitive-probes-tagp-mc`    | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc) |

## Notebooks (Kaggle)

| Track | Notebook |
|-------|----------|
| THLP  | [THLP Hippocampal Learning MC](https://www.kaggle.com/code/playra/trinity-thlp-hippocampal-learning-mc-benchmark) |
| TTM   | [TTM Metacognition MC](https://www.kaggle.com/code/playra/trinity-ttm-metacognition-mc-benchmark) |
| TSCP  | [TSCP Social Cognition MC](https://www.kaggle.com/code/playra/trinity-tscp-social-cognition-mc-benchmark) |
| TEFB  | [TEFB Executive Function MC](https://www.kaggle.com/code/playra/trinity-tefb-executive-function-mc-benchmark) |
| TAGP  | [TAGP Attentional Gateway MC](https://www.kaggle.com/code/playra/trinity-tagp-attentional-gateway-mc-benchmark) |

## Data Format

All CSVs share a consistent schema:

| Column          | Description                              |
|-----------------|------------------------------------------|
| `id`            | Unique question identifier               |
| `question_type` | `mc` (multiple-choice) or `factual`      |
| `question`      | The question text                        |
| `choices`       | Answer options formatted as A) ... B) ... C) ... D) ... |
| `answer`        | Correct letter (A, B, C, or D)           |

## Notebook Structure

Each benchmark notebook follows an identical 8-cell structure:

1. **Install** — `kaggle-benchmarks` and dependencies
2. **Import** — libraries and configuration
3. **Download** — dataset from Kaggle API
4. **Load** — CSV into DataFrame, filter MC rows, full dataset eval
5. **Schema** — `MCAnswer` dataclass for structured output
6. **Inner task** — `{track}_single_mc` (per-question, `store_task=False`)
7. **Outer benchmark** — `{track}_mc_benchmark` (aggregate accuracy)
8. **Run & Submit** — execute and `%choose` for leaderboard

## Cognitive Taxonomy

Based on the DeepMind paper "Measuring Progress Toward AGI: Cognitive Abilities":

```
Perception ─┐
Generation ─┤
Attention ──┤── Problem Solving
Learning ───┤── Social Cognition
Memory ─────┤
Metacognition ─┘
Executive Functions ─┘
```

Our 5 tracks target the areas with the largest evaluation gaps:
- **Learning** (THLP) — Can the model learn new patterns on the fly?
- **Metacognition** (TTM) — Does the model know what it doesn't know?
- **Attention** (TAGP) — Can the model maintain focus under noise?
- **Executive Functions** (TEFB) — Can the model plan, inhibit, and switch?
- **Social Cognition** (TSCP) — Does the model understand social contexts?

## Local Development

```bash
# Install dependencies
pip install kaggle-benchmarks kaggle pandas

# Run any notebook locally (requires Kaggle API credentials)
jupyter notebook notebooks/thlp_mc_benchmark.ipynb
```

## License

CC0-1.0 — Public Domain

## Citation

```
Trinity Cognitive Probes — Ternary Hyperdimensional Computing AGI Benchmark
DeepMind x Kaggle "Measuring Progress Toward AGI" Hackathon, 2026
```
