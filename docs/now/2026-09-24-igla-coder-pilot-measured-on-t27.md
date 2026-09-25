# NOW -- IGLA-Coder pilot is measured on t27, judged by t27c (2026-09-24)

## The WARS lane 03 checkpoint now exists, with numbers (Closes #1041)

- The IGLA coder pipeline (`specs/igla/coder/*.t27`) had no measured checkpoint: the WARS arena lists lane 03 (IGLA CODER) as "no executable checkpoint verified", and `pipeline.t27` described the pipeline without a result. The `igla-coder-gpu` pilot changes that.
- Trained checkpoints now exist: **fp100m + t27 fine-tune** and **tern100m + t27 fine-tune** (100m params, 2.5B pretrain tokens on permissive code, 3-epoch fine-tune on the `.t27` corpus).
- Measured on the fill-in-the-body benchmark, **judged by `t27c test-report`** (the exact compiler judge): on the strict set of 391 items (both constant mutants make a test fail), fp solves **8/391** and tern **5/391** within 10 tries; over all 466 items, fp pass@10 **5.8%** (27 solved), tern **5.4%** (25 solved); compile@1 stays **12-14%** at every size, which is the bottleneck. t27 validation bits/byte: fp 0.4589, tern 0.4957.
- Recorded in `specs/igla/coder/pipeline.t27` as a documented PILOT MEASUREMENT block with its provenance (`gHashTag/igla-coder-gpu research/T27_BENCH.md`).
- **Honesty limit (BINDING):** the checkpoint is not yet published with a verifiable SHA, so the live WARS arena lane stays unregistered until it is; a 100m model solves about 2% of held-out functions in ten tries; this is a software result judged by `t27c`, not a hardware or deployed-network one.
