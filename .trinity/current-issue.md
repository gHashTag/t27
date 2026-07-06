Wave Loop 451 — Formal boot-evidence expansion + adversarial envelope theorem + CI metric hardening (Variant B default) (Closes #1426)

Branch: wave-loop-451
Issue: #1426

Variant A (physical cold-POR capture with live fixture archive) is preferred if
the bench unblocks (DLC10 cable detected, P12 wired or relay gate available).
Variant B (default) is a software-only continuation that adds a quantified
transaction theorem over an adversarial or envelope-corner operating point,
hardens the `FpgaSmokeResult`/`SuiteSummary` schema so phases cannot silently
drop metrics, and extends the `--fast` suite path to produce a clean machine-
readable summary when the standalone build is skipped.

Deliverables:
1. Quantified adversarial/envelope-corner transaction theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` (or live-capture theorem if Variant A).
2. Schema hardening (builder or non-default guard) for `FpgaSmokeResult` and
   `SuiteSummary` to protect the new `fpga-smoke-gate-standalone` phase metric.
3. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` for the W451 boundary.
4. Close-out artifacts: W451 report, evidence, plan, and W452 cooperation variants.

Blocked / out of scope:
- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Master-merge to clear #1245 — deferred to a dedicated future wave.
