# Kaggle Datasets - Status Summary

## ✅ All Datasets Uploaded and Fixed

| Track | Kaggle URL | Status | Issues Fixed |
|-------|------------|--------|---------|
| **THLP** | [trinity-cognitive-probes-thlp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc) | ✅ Ready | Multiline fixed |
| **TTM** | [trinity-cognitive-probes-tmp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc) | ✅ Ready | Multiline fixed |
| **TAGP** | [trinity-cognitive-probes-tagp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc) | ✅ Ready | Multiline fixed |
| **TEFB** | [trinity-cognitive-probes-tefb-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc) | ✅ Ready | Multiline fixed |
| **TSCP** | [trinity-cognitive-probes-tscp-mc](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc) | ✅ Ready | Multiline fixed |

**Total Questions:** 65,133 MC questions across 5 cognitive tracks
**License:** CC0-1.0 (Public Domain) for all datasets
**Source:** [github.com/gHashTag/t27](https://github.com/gHashTag/t27/tree/main/external/kaggle)

## Issues Fixed

**Multiline choices:** Replaced `\n` with ` | ` in CSV choices column for all tracks (TTM, TSCP, TEFB, TAGP, THLP). This fixes Kaggle Data Explorer showing 5 columns instead of correct 4 columns.

## Next Steps

### 1. Benchmark on Real Models

Use the benchmark notebooks in `external/kaggle/notebooks/`:
- [THLP Benchmark](notebooks/thlp_mc_benchmark.ipynb)
- [TTM Benchmark](notebooks/ttm_mc_benchmark.ipynb)
- [TAGP Benchmark](notebooks/tagp_mc_benchmark.ipynb)
- [TEFB Benchmark](notebooks/tefb_mc_benchmark.ipynb)
- [TSCP Benchmark](notebooks/tscp_mc_benchmark.ipynb)

### 2. Kaggle Notebooks (Online)

Open notebooks directly on Kaggle:
- [THLP](https://www.kaggle.com/code/playra/trinity-thlp-hippocampal-learning-mc-benchmark)
- [TTM](https://www.kaggle.com/code/playra/trinity-ttm-metacognition-mc-benchmark)
- [TAGP](https://www.kaggle.com/code/playra/trinity-tagp-attentional-gateway-mc-benchmark)
- [TEFB](https://www.kaggle.com/code/playra/trinity-executive-function-mc-benchmark)
- [TSCP](https://www.kaggle.com/code/playra/trinity-scp-social-cognition-mc-benchmark)

### 3. Models to Test

**Recommended Models:**
- Claude 3.5 Sonnet (anthropic)
- Claude 3.5 Opus (anthropic)
- GPT-4 Turbo (openai)
- GPT-4o (openai)

**API Keys Required:**
- Anthropic: `export ANTHROPIC_API_KEY="..."`
- OpenAI: `export OPENAI_API_KEY="..."`

## Repository

- **Datasets:** [github.com/gHashTag/t27/tree/main/external/kaggle](https://github.com/gHashTag/t27/tree/main/external/kaggle)
- **Notebooks:** [github.com/gHashTag/t27/tree/main/external/kaggle/notebooks](https://github.com/gHashTag/t27/tree/main/external/kaggle/notebooks)
- **Scripts:** [github.com/gHashTag/t27/tree/main/external/kaggle/scripts](https://github.com/gHashTag/t27/tree/main/external/kaggle/scripts)

**Last Updated:** 2026-04-15
