# NOW -- The gap was the sample, not the protocol (2026-09-04)

## Two protocols on the SAME 24 agree 22 times

- Last pass reported 83% accusatory in a sample of 24 against 69% across the other 81, and offered
  two explanations: an unrepresentative sample, or batched judges being more conservative than solo
  ones. I also said the comparison could be made from the existing journals. **That was wrong** --
  the two runs judged **disjoint** sets, overlap zero, so no verdict-by-verdict comparison existed.
- Run properly: the same 24, same doctrine text, only batched-versus-solo changed.

      solo   20 ABSENT_AND_CLAIMED, 3 renamed, 1 not-a-deliverable   (83%)
      batch  19 ABSENT_AND_CLAIMED, 2 renamed, 3 not-a-deliverable   (79%)

  **They disagree on 2 names of 24 -- 92% agreement.** Batching costs about one verdict in
  twenty-four. It does not explain a fourteen-point gap.
- So the 83/69 difference is **sampling**, not protocol: the random 24 happened to hold more
  genuine cases than the other 81. My leading hypothesis was the weaker one.
- Both runs are complete measurements -- the batched one returned 24 of 24 and says so in its own
  result, a guard added after a previous fan-out reported 4 of 81 without noticing (see 503).

## `tri merge-losses`

- Two merges across 415 took one parent's file whole and discarded the other side: **649 lines** of
  Lean and **454 of `bootstrap/src/main.rs`**. The second took three Coq proofs with it --
  `gamma_phi3.v` 32 lines -> 12, `dl_bounds.v` 32 -> 15, `l5_identity.v` 27 -> 18 -- **none of them
  in `coq/_CoqProject`**, so nothing compiles them and nothing could have noticed.
- The obvious check returns **1197 hits and 2,595,752 lines**: ordinary divergence. The structural
  one needs no threshold and returns two.
- First implementation took over ten minutes; a file resolved "ours" is exactly one the merge did
  not change relative to P1, so two name-only diffs replace three `rev-parse` calls per file. 67s.

Refs #3150
