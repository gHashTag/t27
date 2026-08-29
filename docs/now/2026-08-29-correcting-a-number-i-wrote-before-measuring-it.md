# NOW -- Correcting a number I wrote before measuring it (2026-08-29)

## Correcting a number I wrote before measuring it (Refs #2754)

- the previous commit message claims 2430 cargo tests passed; the measurement is 2429
- the new conformance case is a t27c subcommand, not a cargo test, so it never enters that total
- corrected by a follow-up commit rather than an amend: no force-push, per the rule I broke earlier today
