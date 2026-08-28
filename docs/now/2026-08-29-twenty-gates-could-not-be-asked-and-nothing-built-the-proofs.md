# NOW -- Twenty gates could not be asked, and nothing built the proofs (2026-08-29)

## Twenty gates could not be asked, and nothing built the proofs (Refs #2747)

- tri gates unmeasured says 28 of 58 have no default-branch run in 30 days; twenty had no workflow_dispatch either, so the reading could not be taken on purpose
- all twenty now have it -- a manual trigger causes no runs by itself, it only makes the question askable
- proofs/lean4 gets the first job that compiles it; deliberately not gating master until a dispatch says whether the 250 theorems still build
- five sorry sit in the tree, one commented REQUIRES lake build -- held by a down-only ratchet, since a sorry compiles and a green build says nothing about them
