# NOW -- Prove over a fixed sample, and the distinction lived in the tools (2026-08-24)

## Prove over a fixed sample, and the distinction lived in the tools (Closes #2161)

- Three steps named Prove X bit-exact run on fixed samples: 14 topologies from a list, 600 operand pairs drawn from 48 values, 80 seeded training steps. The fourth, verify_exhaustive, genuinely enumerates and its first line says where that stops — wherever the space is small.
- I checked for the failure I expected first: an unseeded sample. It is seeded, and two consecutive runs are byte-identical. That is a strength with a shadow — the sample never moves, so a disagreement outside it cannot be found here however often CI runs.
- The distinction was already in the repository, in the tools, written carefully by an author who understood it. It had never reached the step names, and the step name is what a green check shows.
- My own scope note was invisible to tri claims because I wrote it in my own words. The fix was to use the repository's vocabulary, not to widen the detector: an established vocabulary is worth more than any single statement made in it.
