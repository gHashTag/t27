# NOW -- A ratchet measuring a subset of its own subject (2026-08-30)

## 114 vacuous completeness theorems; the ratchet counts 44 (Refs #2747)

- `Completeness.lean` holds 250 hand-transcribed models; **114** have `functions := []`, and 104 of those have an empty `Env` too
- a theorem about an empty module says nothing about its spec: `native_decide` on an empty structure proves something true and useless
- `max_vacuous`, the ratchet that exists to stop that number growing, counts **44** -- only the models that ALSO disagree with the Rust classifier
- **70 vacuous theorems are invisible to it**, and the number it reports looks like the number you care about
- all 44 of its marks are correct: zero entries marked `model_empty` that are not empty
- measured twice, by a throwaway Python scan and by the shipped Rust, and the two agree at 250/114/104/44/70

## What I proposed last pass and am NOT doing

- I named "write real Lean models for those four" as the only honest way to lower `max_vacuous`
- `lean`, `lake` and `elan` are not installed here; `.github/workflows/lean-proofs.yml` has a `lake build` but I cannot run it locally
- a faithful model may make its theorem FALSE -- the classifier says `Rust=false` for those four specs -- and that is the correct outcome, not a bug
- writing a proof I cannot check and pushing it to see what CI says is exactly what this repository exists to prevent, so the count is reported instead
