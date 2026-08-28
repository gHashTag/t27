# NOW -- A command that reports what nobody measured, and got it wrong first (2026-08-29)

## A command that reports what nobody measured, and got it wrong first (Refs #2754)

- tri gates unmeasured finds workflows with no default-branch run: 28 of 58 here, against tri gates dead which asks the opposite question
- its first version said 58 of 58 -- broken jq, and unwrap_or_default turned every failed query into never ran
- no default on error now: could not ask is counted separately and never rendered as did not run
- loop-recon saved as a reusable workflow with its shape written down beside it
