# NOW — a mutant escaped into a commit (2026-08-23)

The five-operator run exceeds ten minutes, so I backgrounded it. A timeout killed an earlier one. The loop writes a mutant, runs the control, restores — and a kill lands between the first and the third.

- **A boundary mutant stayed in `gft_backprop_microcode.py`. `git add -A` staged it.** It went into a commit, a push, and an open pull request — a deliberately broken line, in the file whose control I had just written, in a repository whose whole subject is gates that cannot fail.
- The command's docstring already said the restore is recoverable with `git checkout tools/`. True, and useless: **you have to know an interrupt happened.** The dirty-tree guard could not help — it refuses to *start* dirty, and by then the mutant was already staged.
- **Two failures, and the second shipped it:** staging everything and trusting nothing else moved. During a mutation run the tree is *transiently* dirty by design, so `git add -A` in that window commits whatever the loop is holding.

**Fixed both ways.** A marker under `target/` — already ignored, so it can never be the dirt it warns about — is written before each gate and removed on success; a later run refuses to start and prints the recovery commands. And the habit: during mutation work, stage named files, never `-A`.

**The demonstration failed on its first attempt, correctly.** With the background run still holding a file mutated, the dirty-tree guard fired before the marker check — the older guard doing its job, and proof the two cover different moments rather than the same one.
