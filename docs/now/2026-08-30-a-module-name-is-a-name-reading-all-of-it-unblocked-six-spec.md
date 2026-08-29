# NOW -- A module name is a name: reading all of it unblocked six specs (2026-08-30)

## A module name is a name: reading all of it unblocked six specs (Refs #2864)

- The module-name parser read one segment and stopped, so 'module github::auth {' left the parser looking at a colon at module level. Nine specs declared a path-qualified module and none parsed. Rewritten as a loop over ::-separated segments, each of which may still be hyphenated.
- Both colons are required before either is consumed: a single ':' after a module name is not a path, and swallowing it would turn a real error into a stranger one further down.
- Controls: 621 specs parsed before, 627 after, ZERO regressions. Seal drift went 537 -> 543, exactly the six new specs, so no previously-parsing spec's output changed. Bootstrap test ratchet: no new failures.
- Hollow seals 187 -> 175, generate 626 -> 632, debt ledger 90 -> 84. FROZEN_HASH updated in the same commit per M5.
- New: tri unparsed report [--list] -- the work queue by construct. Its own first three readings were wrong: it counted 21 deliberately-broken fixtures as debt, lost 15 upstream statements into 'not decided', and dropped 4 rows through a bare continue.
