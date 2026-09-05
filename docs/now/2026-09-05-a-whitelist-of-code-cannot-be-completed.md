# NOW -- A whitelist of code cannot be completed (2026-09-05)

## A whitelist of code cannot be completed (Refs #3292)

- The guard's first form asked "does the diff contain a file with a code extension", and
  the list grew twice before it was even proposed: {rs, py, t27, zig}, then {c, h, v, sv,
  svh} once 164 hand-written `.v` files were noticed.
- An adversarial pass then named four more categories that exist here, each of which would
  have been a false accusation in a REQUIRED context: 14 `.xdc` and 4 `.tcl`, which are the
  actual deliverable of timing work under `fix(verilog)`; 43 `.toml`, where a build
  breakage genuinely lives; 72 `.lean` formalising the compiler's own lowering; and every
  extensionless path -- `Makefile`, `Dockerfile`, `scripts/tri` -- which `rsplit_once('.')`
  answers None for, so no whitelist entry can ever match it.
- The list was never going to close, and each omission accuses somebody.
- So the question is inverted. #3264's entire diff was ONE file, a `docs/now` note. What
  it lacked was not a particular extension; it was anything at all besides prose.
- Prose and records are small and closed: under `docs/`, under `.trinity/seals/`, or `.md`.
  Substance is everything else. Refuse when a compiler-scope title has nothing but prose.
- Re-measured over the same 498 commits: 1 refusal, the same commit, and the refusal set
  is byte-identical to the whitelist version's. Two readers, 0 disagreements, 0 could-not-run.
- Proven end to end on real diffs: `.xdc`, `Cargo.toml`, `Makefile` and `.lean` pass;
  `docs/now/*.md` and `.trinity/seals/*.json` are refused.
- The probe that proved it was wrong the first time. The script resolves its repository
  from its own path, so running the copy in MY tree measured MY tree, not the scratch
  worktree -- and reported the defect case as passing. Copying the script into the tree
  under test fixed it.
