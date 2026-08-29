# NOW -- The conflicted type names are pinned by identity, not by count (2026-08-29)

## The conflicted type names are pinned by identity, not by count (Refs #2774)

- tri types ratchet: 79 names with more than one definition, pinned as a SET -- a new conflict fails and a resolved one fails until it is blessed away
- identity and not a count because a count cannot see a swap: one resolved while another appears leaves the total unchanged and the ledger wrong
- seen failing three ways on purpose, including the swap at a constant 79
- absence is not amnesty: with no ledger the command exits 1 rather than passing quietly
