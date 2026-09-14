# NOW -- The adversarial pass edited my worktree, and its edit was right (2026-09-06)

## The adversarial pass edited my worktree (Refs #3379)

- Four attackers were pointed at `tri window` with a brief forbidding pushes and other
  people's branches. It did not forbid editing files, and one of them applied its proposed
  repair to my working tree: 60 insertions, 9 deletions, uncommitted, on my branch.
- The pull request was unaffected -- it carried the committed version -- but the gap is
  mine: a brief that says "do not push" and not "do not edit" gets edits.
- The edit was correct. `--check` discarded the fetch's exit code, so a fetch that failed
  left the LOCAL `origin/master` at the sha `--start` recorded, `tip == recorded` became
  true, and the command certified as clean the one failure it exists to prevent. Three of
  the four lenses named it independently.
- It also derived the remote FROM the base instead of hardcoding `origin`, which makes a
  local branch, a tag or a sha "nothing to fetch, nothing to be stale" rather than a
  refusal. Reviewed line by line and kept, with attribution.
- Three more found and fixed by hand: a record shorter than twelve bytes PANICKED at
  `&recorded[..12]` (exit 101); the recorded ref NAME was written and never read back, so
  `--start --base A` then `--check --base B` compared two branches and called it a move;
  and a rewind reported "0 merge(s) landed in between", which says the opposite of what
  happened.
- **And one defect was entirely mine, made while fixing theirs.** A blind
  `re.sub('&sha[..12]' -> 'short(&sha)')` rewrote the body of `short` itself into
  `short(&sha)` -- infinite recursion. It presented as a hang with no output, and I spent
  four probes on the network and on concurrent agent builds before watching WHERE the
  output stopped: after the fifth control, before the sixth, which is the one that tests
  `short`. The stopping point named the function.
- A record that is not a sha now exits **2**, not 1: saying "the base moved" about `abc`
  names the wrong fact and points at the wrong repair.
