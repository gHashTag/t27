# NOW -- Two of my own commands disagreed about the same population (2026-08-30)

## Two of my own commands disagreed about the same population (Refs #2864)

- I opened this iteration to bridge tri unparsed and tri prose -- the census abstains on some specs and prose was supposed to answer six of them. It answers ZERO: the earlier prose repairs closed them, and my own options list carried a stale claim.
- What was there instead: prose report said '107 specs that do not parse' where unparsed report said 76. The gap is 21 fixtures, broken ON PURPOSE, and 10 specs that parse and fail at a later stage -- the same two rules I added to report, then to locate, and which never travelled to the third sibling.
- Third occurrence of one lesson, so the cure is structural: the scope now lives in ONE function, parse_failures, used by both. Disagreement is impossible rather than merely tested for. Both report 76 parse failures and 21 fixtures.
- No compiler change; FROZEN_HASH does not move. All gates green, corpus ratchet CLEAN.
