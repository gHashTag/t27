# Trinity Cognitive Probes — Now on Kaggle

> phi^2 + 1/phi^2 = 3 = TRINITY

## 65,133 MC Questions Across 5 Cognitive Domains

A comprehensive benchmark suite for evaluating AGI cognitive capabilities,
designed around neuroanatomical brain zone mappings and the Trinity ternary
computing architecture.

Part of the [Google DeepMind x Kaggle: Measuring Progress Toward AGI](https://www.kaggle.com/competitions/llm-cognitive-capabilities) hackathon.

## Tracks

| Track | Domain | Questions | Brain Zones | Kaggle |
|-------|--------|-----------|-------------|--------|
| THLP | Pattern Learning, Belief Update | 19,681 | Hippocampus, Entorhinal Cortex | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc) |
| TTM | Metacognition, Calibration | 4,931 | PCC, dlPFC | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc) |
| TAGP | Attention, Filtering | 17,601 | Parietal Cortex, FEF | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc) |
| TEFB | Executive Function, Planning | 21,081 | dlPFC, ACC, OFC | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc) |
| TSCP | Social Cognition, Theory of Mind | 2,839 | TPJ, mPFC | [Dataset](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc) |

**Total: 66,133 MC questions** with ground truth answers (A/B/C/D), difficulty levels, and brain zone metadata.

## Question Format

Each question follows a standardized MC format:

```csv
id,question_type,question,choices,answer
thlp_pattern_0001,mc,"What pattern emerges...","A) Linear | B) Fibonacci | C) Random | D) Constant",B
```

## Evaluation Metrics

- **Accuracy**: Binary correct/incorrect per question
- **Calibration**: Model confidence vs. actual correctness
- **Composite Score**: 60% accuracy + 20% calibration + 20% mean score
- **Format Validity**: Percentage of parseable A/B/C/D responses
- **Latency**: p50/p95/p99 inference time per question

## Benchmark Runner

A spec-first benchmark runner is included in the repository:

```bash
# Install dependencies
pip install anthropic openai

# Run benchmark (100 samples per track, 3 models)
./scripts/benchmark/run_kaggle_probes.sh

# Dry run (no API calls)
./scripts/benchmark/run_kaggle_probes.sh --dry-run
```

### Supported Models

| Provider | Model | Notes |
|----------|-------|-------|
| Anthropic | claude-3-5-sonnet-20241022 | Primary benchmark target |
| OpenAI | gpt-4o-mini-2024-07-18 | Cost-efficient baseline |
| Meta | llama-3.1-8b-instruct | Open-weight reference |

## Benchmark Results

*To be populated after live benchmark run.*

Results will be stored in `outputs/benchmark/benchmark_results.json`.

## Architecture

Trinity Cognitive Probes are part of the T27 (TRI-27) spec-first ternary
computing architecture. Each cognitive domain maps to specific brain zones,
reflecting the neuroanatomical basis of the assessment:

```
     Hippocampus          PCC          Parietal
    (Learning)      (Metacognition)   (Attention)
         |                |               |
    [THLP: 19,681]   [TTM: 4,931]   [TAGP: 17,601]
         |                |               |
         +--------+-------+-------+-------+
                  |               |
             [TEFB: 21,081]  [TSCP: 2,839]
              dlPFC/ACC/OFC   TPJ/mPFC
            (Executive Fn)  (Social Cog)
```

## License

All datasets are released under **CC0-1.0** (Public Domain).

## Repository

- **Source**: [github.com/gHashTag/t27](https://github.com/gHashTag/t27)
- **Spec**: `specs/benchmarks/trinity_cognitive_probe_runner.t27`
- **Runner**: `gen/benchmarks/trinity_probe_runner.py`
- **Organization**: [gHashTag/trinity](https://github.com/gHashTag)

## Citation

```bibtex
@misc{trinity_cognitive_probes_2026,
  title={Trinity Cognitive Probes: Ternary Hyperdimensional Computing AGI Benchmark},
  author={gHashTag},
  year={2026},
  url={https://github.com/gHashTag/t27}
}
```

---

**phi^2 + 1/phi^2 = 3 | TRINITY**
