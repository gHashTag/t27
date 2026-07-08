# DECONTAM.md — IGLA CODER v0.1

**Version:** 0.1
**Method:** Lee 2022 k=50 substring bidirectional check.
**Reference:** Lee et al., "Deduplicating Training Data Makes Language Models Better" (arXiv:2107.06499).

## Contract (mandatory when the dataset is used to train IGLA CODER/RACE)

For every training pair `(spec, gen)` and every held-out evaluation program `eval_i`:

1. **Forward direction:** no 50-character substring of `spec` or `gen` appears verbatim in any `eval_i` (case-sensitive, whitespace-normalised).
2. **Reverse direction:** no 50-character substring of any `eval_i` appears verbatim in any `spec` or `gen`.

**Failure of either direction** => the pair is quarantined (moved to `dataset/igla-coder/v0.1/quarantine/`), never enters `pairs/`.

## v0.1 status: SKIPPED — trivially clean

Held-out evaluation set at v0.1 is **empty**. The directory `held-out-eval/` exists but contains no programs. Therefore:

- Bidirectional decontam is **trivially clean** (there is nothing to compare against).
- `manifest.decontam_bidirectional = false` (honest: check was not meaningfully performed).
- Per-pair `metadata.json` marks `decontam_status = "Skipped"`.

**This is a known v0.1 gap and is intentional.** Populating the held-out set before running decontam properly is a v0.2 responsibility.

## v0.2 plan (out of scope this Wave)

1. Author 12-16 held-out programs (matching topic distribution of pairs but name-disjoint).
2. Store under `held-out-eval/<eval_id>/spec.tri` (and `gen.<lang>` where applicable).
3. Add `decontam_check.py` (deterministic, stdlib-only, k=50).
4. Re-run: assert every pair passes both directions. Fail-loud on any violation.
5. Publish `decontam_report_v02.md` alongside MANIFEST v0.2.

## Bibliographic honesty

- Lee 2022 (arXiv:2107.06499) establishes the k=50 threshold for near-duplicate detection in LM training corpora. Applying the same threshold bidirectionally to eval-vs-train is a stricter variant used by BIG-bench maintainers and OpenAI's HumanEval-style protocols.
- We do NOT claim originality on this method. We ship a placeholder because v0.1 is a **seed**, not a training corpus.

## Verdict

`DECONTAM v0.1 = SKIPPED (justified). NOT SAFE TO TRAIN on this dataset as-is; awaiting held-out eval.`
