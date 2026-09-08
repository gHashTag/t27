# NOW -- zero of thirteen: the trinity coq proofs have never compiled (2026-09-06)

## zero of thirteen: the trinity coq proofs have never compiled (Refs #3328)

- First full reading, from the all-files report: 0 of 13 compile. The twelve after CorePhi.v all fail at their Require Import line, so they are cascades -- the real state of files 2-13 is still unknown, and CorePhi.v is what gates it.
- CorePhi.v is not one missing lemma. lra is used eleven times and Lra is never imported. apply phi_quadratic; ring applies an equation whose conclusion cannot unify with the goal, and that idiom repeats five times. Rlt_lt_1 and sqrt_lt_cancel do not exist. field at line 32 cannot prove the quadratic because sqrt 5 is opaque to it.
- Every statement is TRUE and is kept byte-identical; only the proofs are replaced, plus two helper lemmas (sqrt5_sq, sqrt5_nonneg) that carry the one fact ring and field cannot discover. Nothing is removed, so no downstream file can break on a missing name.
