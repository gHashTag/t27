# NOW -- A ratchet walking the wrong population (2026-08-30)

## Two lessons from the vacuous count (Refs #2747, #2890)

- 114 of 250 completeness theorems are about an empty module; `max_vacuous` counts 44
- its marks are all correct -- it counts 44 because it reads the MISMATCH ledger, and a model reaches that ledger only if it also disagrees with the Rust classifier
- the two questions were joined because one file happened to hold both answers; being vacuous has nothing to do with the classifier disagreeing
- worse than no ratchet: with none the question is open, with this one it has an answer and the answer is wrong by 70
- the check for any ratchet: what population does it walk, and is that the population its name describes?
- I recommended writing four real Lean models last pass and did not do it: `lean`/`lake`/`elan` are absent here, and a FAITHFUL model may make its theorem false, which would be the correct outcome
- so a red `lake build` would mean either "the theorem was false" or "you transcribed it wrong", and with no local build there is no way to choose
- pushing an artefact you cannot check to see what CI says is guessing with a longer feedback loop -- and in this medium the artefact is a proof
