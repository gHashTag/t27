# Experience: Kaggle Cognitive Probes Benchmark

## Episode Summary

- **Episode ID**: phi-2026-04-15T08:08:00Z#kaggle-bench
- **Skill ID**: kaggle-cognitive-benchmark
- **Session ID**: 2026-04-15T08:08:00Z#kaggle-bench
- **Issue ID**: KAGGLE-BENCH-001
- **Actor**: agent:perplexity-computer
- **Started**: 2026-04-15T08:08:00Z
- **Status**: complete (pending live benchmark run)

## What Was Done

### Phase 1: SPEC
- Created `specs/benchmarks/trinity_cognitive_probe_runner.t27`
- Defines 5 cognitive tracks, MC question format, benchmark configuration
- Includes test/invariant/bench blocks per SOUL.md Article II
- TRINITY invariant verified: phi^2 + 1/phi^2 = 3

### Phase 2: SEAL
- spec_hash_after: `sha256:e5432742efdee2df977d5856c4feddc25cd99e10d1f345f031c52f7f23563c3c`
- Active skill and issue binding set

### Phase 3: GEN
- Generated `gen/benchmarks/trinity_probe_runner.py` from spec
- Python permitted for benchmarks (not critical path)
- gen_hash: `sha256:af73393ebb6761cb288af8418ab8c15c8053509dac0e85c3b7f97b90f87a9536`
- Supports Anthropic Claude, OpenAI GPT, Together/Llama APIs
- MC answer parsing with regex fallbacks

### Phase 4: TEST
- Dry run with mock data: PASSED
- Dry run with real CSV from `external/kaggle/data/`: PASSED
- TRINITY invariant verified at import time
- All 5 tracks loaded successfully
- Output table formatted correctly (15 results = 5 tracks x 3 models)

### Phase 5: BENCHMARK RUNNER
- Created `scripts/benchmark/run_kaggle_probes.sh` launcher
- Requires API keys (ANTHROPIC_API_KEY, OPENAI_API_KEY, TOGETHER_API_KEY)
- Live benchmark pending: 1500 API calls (100 samples x 5 tracks x 3 models)

### Phase 6: KAGGLE UPDATES
- Created `scripts/benchmark/update_kaggle_descriptions.py`
- Updates all 5 datasets with:
  - Corrected row counts in About description
  - github.com/gHashTag/t27 repo link in README
  - CC0-1.0 license enforcement
  - BibTeX citation block
- Requires `kaggle` CLI with playra credentials

### Phase 7: PROMO
- Created `docs/promo/trinity-cognitive-launch.md`
- Comprehensive launch document with track table, brain zone mappings
- ASCII art architecture diagram
- Citation block and license info

## Conformance

- Conformance vectors: `conformance/benchmark_cognitive_probe.json`
- 5 test vectors: all PASSED
- 3 invariants: 2 verified, 1 pending (kaggle_metadata_synced)

## Kaggle Dataset Verification

| Track | Slug | Rows | Columns | License | Issues Found |
|-------|------|------|---------|---------|--------------|
| THLP | trinity-cognitive-probes-thlp-mc | 19,681 | 5 | CC0 | None |
| TTM | trinity-cognitive-probes-tmp-mc | 4,931 | 1* | CC0 | Multiline CSV, About row count |
| TAGP | trinity-cognitive-probes-tagp-mc | 17,601 | 5 | CC0 | None |
| TEFB | trinity-cognitive-probes-tefb-mc | 21,081 | 5 | CC0 | About row count outdated |
| TSCP | trinity-cognitive-probes-tscp-mc | 2,839 | 1* | CC0 | Multiline CSV, MIT in text |

*Data Explorer shows 1 column due to multiline newlines in choices field.

## Files Created

| File | Hash | Purpose |
|------|------|---------|
| specs/benchmarks/trinity_cognitive_probe_runner.t27 | sha256:e543... | Benchmark spec |
| gen/benchmarks/trinity_probe_runner.py | sha256:af73... | Python runner |
| scripts/benchmark/run_kaggle_probes.sh | sha256:d7dc... | Launcher script |
| scripts/benchmark/update_kaggle_descriptions.py | sha256:026b... | Kaggle updater |
| docs/promo/trinity-cognitive-launch.md | sha256:ca89... | Promo document |
| conformance/benchmark_cognitive_probe.json | — | Conformance vectors |
| .trinity/experience/kaggle-benchmarking.md | — | This file |

## Verdict

**CLEAN** — All spec tests pass. No toxic mutations. Generated code matches spec.

### Remaining Actions (require credentials)
1. Run live benchmark with API keys
2. Execute `update_kaggle_descriptions.py` with Kaggle CLI
3. Fix multiline CSV for TTM and TSCP (see `fix_kaggle_datasets.py`)

## Lessons Learned

1. Kaggle Data Explorer does not handle multiline values in CSV fields.
   Always use ` | ` separator instead of newlines for choices.
2. Kaggle "About" description is separate from README.md — both need updating.
3. The `external/kaggle/data/` directory contains the source CSV files;
   Kaggle has renamed versions (v5, _new).

---

phi^2 + 1/phi^2 = 3 | TRINITY
