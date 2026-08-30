# NOW -- Each test gets its own scratch directory (2026-08-30)

## Each test gets its own scratch directory (Closes #2954)

- four binaries keyed the scratch dir by (pid, src.len()) and each test deleted the whole directory; one erased the spec another was mid-read of
- measured, not inferred: scaffold_c produced THREE directories for six tests; a freshness probe fired 8 runs out of 8 on three other files
- it passed the first time it was written -- a green run does not clear a race
- tri harness scratch (--gate, --self-check) guards the class; two false readings were found by counterexample, not by review
