# NOW -- Corpus artifact-failure fixture reaches the real child path (2026-10-04)

## Child PID failure injection (Closes #5903)

- Closes #5903. The fixture previously blocked c.ext while corpus writes c-PID.ext, so it returned success without exercising the write guard.
- On Unix a std Rust launcher prepares exactly one PID-qualified directory and execs the real t27c. All four independent refusal checks, exit 2 and exclusive error labels remain enforced.
- The repaired fixture passes with the unchanged baseline compiler and the candidate. All seven corpus_unresolved tests pass on this laptop. Non-Unix behavior is retained and is not verified here.
- Full bootstrap testing separately exposes existing Lean classifier mismatches for nn_phi_rope and nn_sacred_attention, reproduced on the baseline. This patch changes no compiler, ledger, production behavior or test exclusion.
