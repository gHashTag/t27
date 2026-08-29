# NOW -- The guard named one member of the class it guards (2026-08-29)

## The guard named one member of the class it guards (Refs #2804)

- secret-scan blocks ONE developer home spelling; there are two, and the unguarded one is in 33 files against the guarded one's 5 -- 28 of the 33 executable, including bootstrap/src/service.rs
- found by triaging the FPGA rows of tri orphaned: a hardcoded absolute checkout root the compiler joins onto
- debt pinned per file rather than fixed: 33 files, 51 occurrences; new fails, growing fails, shrinking fails so slack cannot be banked
- seen failing on purpose: a planted occurrence gives FAIL new, and absence of the baseline is refused rather than treated as amnesty
