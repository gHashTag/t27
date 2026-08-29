# NOW -- One name, two definitions, different scores (2026-08-29)

## Thirteen competitors defined up to four times (Refs #2822, #2807)

- `specs/igla/coder/benchmark.t27` defines 13 competitor functions 2-4 times each and no two copies are identical
- five state a different score for the same competitor: `estrtl_competitor` is `pass_at_1` 0.705 at line 307 and 0.0 at 698 and 2202
- in four of the five the scored copy comes FIRST, so a consumer taking the last definition reads four competitors as scoring zero -- in the file that compares IGLA CODER against the field
- `t27c parse` reads all of it, exits 0, and emits no diagnostic; the AST holds every copy
- the other eight differ only in a provenance string that gained an arXiv id, which is drift and not contradiction -- the check reports those and does not fail on them
- nothing published carries these numbers, which is why this is an issue and not a withdrawal
- `tri types redef` exits 1 on master today and is deliberately not in CI: a gate that is red on arrival teaches people to ignore it
