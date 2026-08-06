# IGLA CODER Dataset v0.1 — Seed

**Status:** SEED (not a training corpus).
**Pair count:** 8.
**Total spec bytes:** ~31 KB.
**Estimated token count:** ~7-10K (bpe token equivalent, rough estimate; measured at v0.2 with a tokenizer spec).

## Purpose

First public seed of the IGLA CODER training corpus: parallel `(spec, gen)` pairs where `spec` is a `.tri` T27 specification and `gen` is the corresponding generated code (Zig / Rust / Verilog / TypeScript — the latter deferred to Wave-3).

## What is here (v0.1)

Eight `.tri` specs, one per pilot ring of the 2026-07-08 ecosystem Wave (`ring-105-001..007`):

| Pair ID | Spec | Target lang | Codegen status | Ring |
|---|---|---|---|---|
| 0001-experience-format | organism/experience.tri | None (spec-only) | n/a | 105-001 |
| 0002-mozg-state-machine | organism/mozg.tri | Zig | TRAIN-BOX pending | 105-002 |
| 0003-dna-schema | organism/dna.tri | Zig | TRAIN-BOX pending | 105-002 |
| 0004-git-orchestrator | git/orchestrator.tri | Zig | TRAIN-BOX pending | 105-003 |
| 0005-mcp-tool-registry | mcp/tool_registry.tri | Rust | STUB (Wave-2) | 105-004 |
| 0006-scene-schema | scenes/scene_schema.tri | TypeScript | DEFERRED (Wave-3) | 105-005 |
| 0007-ring-runtime | organism/ring_runtime.tri | Rust | STUB (Wave-2) | 105-006 |
| 0008-igla-manifest | dataset/igla_coder_manifest.tri | None (spec-only) | n/a | 105-007 |

## What is NOT here (honest gaps)

- **No generated code.** All pairs are spec-only or spec-with-codegen-stub. Wave-2 (rust codegen) and Wave-3 (typescript codegen + real train-box regeneration) fill this in.
- **No held-out evaluation set.** `held-out-eval/` is empty. See DECONTAM.md.
- **No tokenizer.** Byte-level fallback assumed; explicit tokenizer spec is Wave-4.
- **No BPB baseline.** Impossible without a real train run.
- **~7-10K tokens is 3+ orders below phi-1-small floor.** IGLA CODER v0.1 CANNOT train a working model. This is a seed, not a corpus. See report/FINAL_REPORT.md §W-18.

## What v0.2..v0.5 must add (roadmap)

- v0.2: populate held-out-eval, rerun decontam bidirectionally, publish decontam_report_v02.md.
- v0.3: apply WP-10 deterministic alpha-rename augmentation to reach ~20K train tokens.
- v0.4: add tokenizer spec `specs/tokenizer/tri_tokenizer.tri`.
- v0.5: reach >=200K train tokens (deficit <=262x per WP-10 analog); actual train-box run authorised.

## Files

- `MANIFEST.json` — canonical index (schema in `specs/dataset/igla_coder_manifest.tri`)
- `DECONTAM.md` — decontamination policy and v0.1 status
- `pairs/<pair_id>/spec.tri` — the .tri spec
- `pairs/<pair_id>/metadata.json` — per-pair metadata (sha256, ring_id, target_lang, license)
- `held-out-eval/` — v0.1: empty by design

## License

Each pair inherits its source repo's license, recorded per-pair in `metadata.json`.

- ring-105-001..004 + 007: Apache-2.0 (from gHashTag/t27)
- ring-105-005, 105-006: Apache-2.0 (anonymised abstractions of PRIVATE 999-multibots-* repos; NO secrets, NO PII, NO wallet, NO tokens in this dataset)

## SHA-256 reproducibility

All `spec_sha256` values in MANIFEST.json are computed with Python stdlib `hashlib.sha256(open(path,"rb").read())` at build time. Re-running `compute_manifest.py` in this workspace must yield identical hex digests.

## Not for training as-is

**Do not attempt to train IGLA CODER on this dataset.** It is a seed for architecture and pipeline validation, not for capability. Real training gates on Wave-5.
