# NOW -- .len() is usize, rebuilt on a master that had moved (2026-09-05)

## .len() is usize, rebuilt on a master that had moved (Refs #3249)

- #3257 went DIRTY in the squash cascade for the third time. Rebuilt on today's master as
  a new branch, because salvaging the old one needed a history rewrite, which is not
  available here.
- Extracting the change with `git diff -- bootstrap/src/compiler.rs` produced a patch that
  applied cleanly and did not build: the path filter dropped `bootstrap/stage0/FROZEN_HASH`,
  which the M5 ceremony requires in the same commit.
- The build made that harder than it needed to be. Forty lines of language-policy warnings
  about pre-existing documents sit above the panic that names the real cause, and the first
  diagnosis written down -- accumulated garbage in the worktree -- was wrong.
- The control that settled it was a clean checkout of origin/master in a second worktree,
  which built with zero errors. The difference was mine and the panic had already named it.
- Measured over 651 specs with two binaries of distinct hashes: master 333 OK / 248 FAIL /
  69 NOGEN, this branch 335 OK / 246 FAIL / 69 NOGEN.
- The two that rose are `igla/race/cordic_top.t27` and `igla/race/opcodes.t27`, and FAIL
  fell by exactly two. Named by set subtraction, not by `join`, which reported a zero delta
  against totals that disagreed with it.
- Seals re-sealed after the measurement and not before, as `tri seals drift` demands:
  25 drifted to 0.
