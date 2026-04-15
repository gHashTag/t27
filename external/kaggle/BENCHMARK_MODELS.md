# Benchmarking Trinity Cognitive Probes on Real Models

## Overview

Trinity Cognitive Probes benchmarks are designed to evaluate AI models on multiple cognitive domains. This guide explains how to run benchmarks on real language models (LLMs).

## Available Benchmarks

| Track | Notebook | Dataset | Questions |
|-------|----------|---------|-----------|
| **THLP** | [thlp_mc_benchmark.ipynb](notebooks/thlp_mc_benchmark.ipynb) | [playra/trinity-cognitive-probes-thlp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc) | 19,680 |
| **TTM** | [ttm_mc_benchmark.ipynb](notebooks/ttm_mc_benchmark.ipynb) | [playra/trinity-cognitive-probes-tmp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc) | 2,482 |
| **TAGP** | [tagp_mc_benchmark.ipynb](notebooks/tagp_mc_benchmark.ipynb) | [playra/trinity-cognitive-probes-tagp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc) | 17,600 |
| **TEFB** | [tefb_mc_benchmark.ipynb](notebooks/tefb_mc_benchmark.ipynb) | [playra/trinity-cognitive-probes-tefb-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc) | 21,080 |
| **TSCP** | [tscp_mc_benchmark.ipynb](notebooks/tscp_mc_benchmark.ipynb) | [playra/trinity-cognitive-probes-tscp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc) | 2,838 |

## How to Run Benchmarks Locally

### Prerequisites

```bash
pip install kaggle kaggle-benchmarks pandas protobuf
```

Configure Kaggle API credentials:
```bash
kaggle api authenticate
```

### Running a Single Track

```bash
cd external/kaggle/notebooks
jupyter notebook ttm_mc_benchmark.ipynb
```

### Running All Tracks

Modify the notebook to iterate over all 5 tracks, or run them separately.

## Notebook Structure

Each benchmark notebook follows this structure:

1. **Install** - `kaggle-benchmarks` and dependencies
2. **Import** - libraries and configuration
3. **Download** - dataset from Kaggle API
4. **Load** - CSV into DataFrame, filter MC rows, full dataset eval
5. **Schema** - `MCAnswer` dataclass for structured output
6. **Inner task** - `{track}_single_mc` (per-question, `store_task=False`)
7. **Outer benchmark** - `{track}_mc_benchmark` (aggregate accuracy)
8. **Run & Submit** - execute and `%choose` for leaderboard

## Models to Test

### Recommended Models

| Model | Provider | Context | Notes |
|--------|-----------|---------|-------|
| **Claude 3.5 Sonnet** | Anthropic | State-of-the-art reasoning |
| **Claude 3.5 Opus** | Anthropic | Most capable model |
| **GPT-4 Turbo** | OpenAI | Fast and efficient |
| **GPT-4o** | OpenAI | Best reasoning |
| **Llama 3** | Meta | Open source, good local benchmarking |

### Running on Kaggle

The notebooks are designed to run directly on Kaggle. After opening a notebook:

1. Select the model to test in the configuration section
2. Run all cells
3. Results will be submitted to the competition leaderboard

## Expected Results

For each track, you will see:
- **Accuracy** - Percentage of correct answers
- **Valid responses** - Number of questions with valid answers
- **Total questions** - Number of questions evaluated
- **Per-category breakdown** - Performance by question type

## Cognitive Domains Tested

1. **Learning (THLP)** - Pattern induction, belief update, rule learning
2. **Metacognition (TTM)** - Confidence calibration, error detection, meta-learning
3. **Attention (TAGP)** - Selective filtering, sustained attention, attention shifting
4. **Executive Functions (TEFB)** - Multi-step planning, working memory, cognitive flexibility
5. **Social Cognition (TSCP)** - Theory of Mind, pragmatic inference, social norms

## Leaderboard

Results are automatically submitted to the Kaggle competition leaderboard:
- Track-level scores
- Overall aggregate score
- Comparison with other participants

## Troubleshooting

### Authentication Issues

```bash
# Check if credentials are configured
kaggle competitions list
```

### Memory Errors

```bash
# Increase available memory in notebook
import os
os.environ['KAGGLE_MAX_MEMORY_GB'] = '8'
```

### Timeout Errors

Increase timeout in the notebook's `@kbench.task` decorator:
```python
@kbench.task(name='ttm_single_mc', timeout=300)
```

## Citation

If you use results in research, please cite:

```
Trinity Cognitive Probes — Ternary Hyperdimensional Computing AGI Benchmark
DeepMind x Kaggle "Measuring Progress Toward AGI" Hackathon, 2026
```
