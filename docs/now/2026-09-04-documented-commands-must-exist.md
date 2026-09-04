# NOW -- Documented commands must exist (2026-09-04)

## Documented commands must exist (Closes #3093)

- 83 distinct t27c subcommands are named in backticks across README, .claude/skills and docs; 16 of them are not subcommands, and t27c gen-zig -- named in the whitepaper -- cost a full corpus run because clap's exit 2 for an unknown subcommand is this repo's could-not-run code
- Seven live mentions corrected: gen-double-buffer(-ctrl) and gen-weight-prefetch(-ctrl) in README, parse-accounted to parse-complete in ci-gates, editcheck to edit-check in oracle-method, gen-zig to gen in the whitepaper and the nona-01 table
- LAW 8 in TECHNOLOGY-TREE named t27c validate-graph as its verification; no such command has ever existed, so the invariant is stated and unchecked (#3092)
- The gate calibrates its ruler before reading a single document, refuses an empty population, and excuses a mention the document itself declares unbuilt -- on the line or in the heading above it; four mutations kill it, including a stale binary, which exits 2 rather than blaming the docs
