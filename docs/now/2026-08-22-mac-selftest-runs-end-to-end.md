# NOW -- mac's full self-test runs end to end: 30 of 32 checks pass (2026-08-22)

## elaboration 5 -> 0; the two failures are #2410 proving itself (Refs #2413 #2410)

- Two type-registration fixes finish the elaboration arc (95 -> 0 across four
  increments): a test-block binding declared its reg but recorded no TYPE, so
  `r0.raw` on a struct-returning call flattened to the unbound `r0_raw` --
  both the assign-binding and the given/and let-binding paths now register
  the callee's return type when it is a lowerable struct.
- Two in-spec test repairs, each intent-preserving: the parallel-independence
  test dropped its incidental slices (two units, two multiplies, both
  nonzero); the "initially ready" test makes post-reset explicit -- in-spec
  initial blocks run in source order, the same sequencing class the vvp
  registry caught in its own TB.
- The remaining two failures (dot_product, matrix_vector) are ANNOTATED
  known-failures: slice params lower to scalars (#2410), and these tests are
  the live demonstration, kept on purpose. 30/32 checks pass; benches run;
  32/32 yosys smoke. M5 performed.
