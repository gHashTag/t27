# Running Trinity Cognitive Probes on Real Models

## Overview

Guide to running Trinity Cognitive Probes benchmarks on real language models (LLMs).

##  Quick Start

### Option 1: Run on Kaggle (Online)

```bash
cd ~/agi-hackathon
bash scripts/download_data.sh
# Data will be downloaded to ~/agi-hackathon/data/
```

Then:
1. Open the notebook for the desired track on Kaggle:
   - [THLP Notebook](https://www.kaggle.com/code/playra/trinity-thlp-hippocampal-learning-mc-benchmark)
   - [TTM Notebook](https://www.kaggle.com/code/playra/trinity-ttm-metacognition-mc-benchmark)
   - [TAGP Notebook](https://www.kaggle.com/code/playra/trinity-tagp-attentional-gateway-mc-benchmark)
   - [TEFB Notebook](https://www.kaggle.com/code/playra/trinity-executive-function-mc-benchmark)
   - [TSCP Notebook](https://www.kaggle.com/code/playra/trinity-scp-social-cognition-mc-benchmark)

2. In the model settings, select Claude Sonnet, Claude Opus, GPT-4 Turbo, or GPT-4o
3. Run all cells

### Option 2: Local Evaluation with API

```bash
cd ~/agi-hackathon

# Set up API keys (see below)
python3 scripts/test_single.py --track thlp --model claude
```

##  Setting Up API Keys

### Claude API

```bash
# Install
pip install anthropic>=0.25.0

# Configure
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```

### OpenAI API (GPT-4o, GPT-4 Turbo)

```bash
# Install
pip install openai>=1.12.0

# Configure
export OPENAI_API_KEY="sk-proj-..."
```

##  Dataset Status

| Track | Kaggle URL | Status | Fixes |
|-------|-------------|--------|-------|
| **THLP** | [thlp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc) |  Uploaded |  Multiline fixed |
| **TTM** | [ttm-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc) |  Uploaded |  Multiline fixed |
| **TAGP** | [tagp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc) |  Uploaded |  Multiline fixed |
| **TEFB** | [tefb-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc) |  Uploaded |  Multiline fixed |
| **TSCP** | [tscp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc) |  Uploaded |  Multiline fixed |

**All Kaggle Data Explorer issues fixed:**
-  Multiline in choices →  Fixed (replaced with ` | `)
-  Empty descriptions →  Updated (added source links)

##  Evaluation Scripts

Repository: [github.com/gHashTag/t27](https://github.com/gHashTag/t27)

```bash
cd ~/agi-hackathon
ls scripts/
# evaluate.py          - Full evaluation of all tracks
# test_single.py      - Quick test of single question
# download_data.sh     - Download data from Kaggle
```

##  Results

After running, you will get:
- **Accuracy** - percentage of correct answers
- **Breakdown by question types** - breakdown by cognitive skills
- **Leaderboard comparison** - position among other participants

##  Code Sources

| Component | Repository | Description |
|-----------|-------------|-----------|
| Generators | [t27/external/kaggle/scripts](https://github.com/gHashTag/t27/tree/main/external/kaggle/scripts) | Python 3 + t27 spec |
| Benchmarks | [t27/external/kaggle/notebooks](https://github.com/gHashTag/t27/tree/main/external/kaggle/notebooks) | kaggle-benchmarks SDK |
| Evaluation | [agi-hackathon/scripts](https://github.com/gHashTag/t27) | API integration |

##  Cognitive Zones

| Track | Brain Zone | Description |
|-------|------------|-----------|
| THLP | Hippocampus / CA3→CA1 | Rule induction, pattern learning |
| TTM | PCC / dlPFC | Metacognition, confidence calibration |
| TAGP | Parietal + FEF + Thalamus | Selectivity, attention |
| TEFB | dlPFC + ACC + OFC | Planning, working memory |
| TSCP | TPJ + OFC + mPFC | Theory of mind, social norms |

##  Additional Documentation

- [AGI Hackathon README](~/agi-hackathon/README.md) - Complete hackathon documentation
- [Usage Guide](~/agi-hackathon/USAGE.md) - Quick guide

##  Next Steps

1.  Datasets uploaded to Kaggle
2.  Format issues fixed
3.  Evaluation scripts and README created
4.  **Action**: Run benchmarks on models

### For running on Kaggle:

```bash
# Open Kaggle notebook for desired track
# Select model in settings
# Run all cells
```

### For local evaluation:

```bash
cd ~/agi-hackathon

# Set up API key
export ANTHROPIC_API_KEY="..."   # for Claude
export OPENAI_API_KEY="..."      # for GPT

# Run test
python3 scripts/test_single.py --track thlp --model claude
```

---
**Data source:** [GitHub: t27](https://github.com/gHashTag/t27/tree/main/external/kaggle/scripts)
**License:** CC0-1.0 Public Domain
**Last updated:** 2026-04-15
