# NOW -- The type-conflict gate lives where specs change (2026-08-29)

## The type-conflict gate lives where specs change (Refs #2774)

- tri types ratchet runs in corpus-ratchet.yml, not cli-tri.yml: the latter filters on cli/** and a new conflicting struct arrives in specs/**
- a gate whose paths filter excludes the change it exists to catch is a gate that never runs -- the shape this repo has been bitten by before
