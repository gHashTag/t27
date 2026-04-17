# Trinity Ring R007: First Parameter Golf Submission

**Status**: Spec Complete (pending runtime)

## Overview

First Trinity submission to [OpenAI Parameter Golf](https://github.com/openai/parameter-golf) competition.

**Constraint**: <16MB artifact, val_bpb ≤ 1.1378, 3 training runs (p < 0.01)

**Trinity Advantage**:
- VSA/Trit/GF recurrence (native compute)
- Parameter tying (shared weights across layers)
- Bit-packing (2 trits/byte, 8x compression)
- Swarm search (vs hand-tuning competitors)

## Artifacts

1. `specs/examples/r008_parameter_golf.t27` — This spec
2. `train_gpt.py` — Training script (stub)
3. `model.trib` — Trinity VM bytecode
4. `submission.json` — Submission metadata
5. `README.md` — This file

## Model Configuration

TinyLM (from R004):
- Hidden size: 32
- Hidden layers: 1
- Embedding dim: 128
- Vocab size: 4096
- Seq length: 256

**Estimated size**:
- Parameters: ~7B = 7 × 10⁹
- Int8 quantization: ~7GB / 4 = ~1.75GB
- Bit-packing (8x): ~220MB

**Compression needed**: 220MB → 16MB (~13.5x required)

## Submission Format

```json
{
  "model": "Trinity-PG-7B",
  "team": "Trinity Team",
  "artifact_size_bytes": 220000000,
  "val_bpb": 1.15,
  "train_runs": [
    {"seed": 1, "val_bpb": 1.14, "val_loss": 2.45},
    {"seed": 2, "val_bpb": 1.16, "val_loss": 2.38},
    {"seed": 3, "val_bpb": 1.15, "val_loss": 2.41}
  ],
  "tri_version": "0.1.0",
  "submission_timestamp": "2025-01-01T00:00:00Z"
}
```

## Next Steps

1. Implement `train_gpt.py` stub
2. Generate `model.trib` from spec
3. Create `submission.json`
4. Run 3 training seeds
5. Verify val_bpb ≤ 1.1378
6. Submit to OpenAI

## Dependencies

- `specs/examples/toy_lm.t27` (R004 — Minimal LM)
- `specs/r006_swarm_config.t27` (Swarm config)
- `specs/r008_parameter_golf.t27` (This spec)

## Notes

This is a **spec-only** ring. Runtime implementation (actual training) is **NOT DONE**.
R007 will be complete when `train_gpt.py` produces live numbers and submission is uploaded.

---

**Ring Status**: SPEC DONE, RUNTIME PENDING
