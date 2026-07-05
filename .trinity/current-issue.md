Wave Loop 450 — Formal boot-evidence expansion + standalone-build snapshot + CI hardening (Variant B default) (Closes #1425)

Branch: wave-loop-450
Issue: #1425

Variant A (physical cold-POR capture with live fixture archive) is preferred if
the bench unblocks (DLC10 cable detected, P12 wired or relay gate available).
Variant B (default) is a software-only continuation that adds a quantified
transaction theorem over the W448 dry-run-live operating point, hardens the
standalone-build report schema with a snapshot test, and optionally splits the
standalone lake-package build into its own suite phase.

Deliverables:
1. Quantified dry-run-live transaction theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` (or live-capture theorem if Variant A).
2. Snapshot test for the smoke-gate `validate_lean_standalone` report block shape.
3. Optional: dedicated suite phase / `--fast` mode for the standalone build.
4. Competitor refresh in `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
5. Close-out artifacts: W450 report, evidence, plan, and W451 cooperation variants.

Blocked / out of scope:
- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Master-merge to clear #1245 — deferred to a dedicated future wave.
