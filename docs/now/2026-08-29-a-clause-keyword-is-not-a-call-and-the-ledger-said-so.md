# NOW -- A clause keyword is not a call, and the ledger said so (2026-08-29)

## A clause keyword is not a call, and the ledger said so (Refs #2754)

- rung 12 seeded a column so a body opening with a call can lower: 23926 -> 23644 tokens, 76 -> 75 specs
- the corpus total fell and no acceptance column moved, while TWO specs got worse by 120 tokens -- the per-entry ratchet named both
- cause: given (exp, mant) = f(15) is an Ident followed by ( and the arm matched a clause keyword; before the seed it could not reach a block's first token, so nothing had tested it
- my first fix made the total worse (24046) and the spec was still broken -- a disproved theory in one build
