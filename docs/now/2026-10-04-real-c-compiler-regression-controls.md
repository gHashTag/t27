# NOW -- Real C compiler regression controls (2026-10-04)

## Permanent controls (Closes #5936)

- Closes #5936. Accepted upstream #5930 repaired compiler-option selection and rejected-invocation handling; its temporary negative control was removed before publication. Preserve that upstream source and authorship.
- Add five permanent tests against the actual common::cc_check, error_count and get_cc_args APIs: valid C, invalid C, missing-include fatal errors, rejected options, and an isolated probe that must produce no object file. Temporary inputs are removed on success and panic; each probe runs in a separate child process with a fresh OnceLock.
- Exercise real Clang and pinned Linux GCC. Validate the controls with actual helper mutations: removing rejected-invocation protection, dropping fatal-error counting and restoring the object-producing probe must cause their respective tests to fail.
- Existing C fixtures, shared helper implementation, compiler, generated sources, seal hashes and ratchet ledgers/caps remain unchanged. This tests compiler-harness contracts, not radio operation or full model inference.
- Validation completed: Clang and Linux GCC each pass all 110 C tests across 19 targets. The full bootstrap passes 2797 tests with zero failures and two ignored tests across 118 targets; this uses the unchanged compiler source. All three helper mutations fail their controls on both Clang and GCC. Inherited tests may explicitly skip unavailable witnesses; no physical hardware claim.
