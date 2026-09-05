# NOW -- the coq fix worked and uncovered the next defect (2026-09-06)

## the coq fix worked and uncovered the next defect (Refs #3316)

- OPAMROOT fixed the first error: opam now updates and reaches the solver. The job still fails, at a different place, which is what a repair looks like when a workflow has never run far enough to show its second problem.
- coq-interval 4.9.0 requires coq < 8.19 and the image is coqorg/coq:8.19. 7 of 13 proof files Require Import Interval.Tactic, so the dependency is real; only the pin can move.
- Unpinned rather than guessed at a version. No version of coq-interval has ever been observed resolving against 8.19 in this job because the job has never got this far. opam list --installed records what the solver picks, and that reading is what a pin should be restored from.
