# NOW -- tri census explain (2026-10-04)

## What was read

- `tri census pin --gate` prints the FIRST differing line and stops. On #5787 that line was `files read 22 -> 23` while the cause (`0 do not -> 1 do not`: a helper spelling "lower bound" in lowercase) sat further down, and the PR read "open" for six cron ticks while `cli-tri` was red on it.
- The commit that last wrote `tools/census/shell.txt` on master (769f32521) is 1057 changed files away from the tree. Which of them moved a number is not something a reader can see.
- Every pinned census finds its root with `git rev-parse --show-toplevel`, and `gates quiet` asks whether each path a step names exists -- so a census can be run against a copy of the tree, but only a WHOLE copy reads the same.

## What changed

- `tri census explain [--max N]` (`cli/tri/src/census.rs`), read-only. For each pinned census whose output differs from its ledger:
  - every moved line, rows matched by their words with numbers folded (`run: steps 283->284`), where the gate shows one;
  - the commit that last wrote the ledger; the files changed since then (`git diff` against it, plus untracked, minus `tools/census/`);
  - a scratch copy of the tree (tracked and untracked files, no ignored ones) with its own `git init`, which must reproduce today's reading byte for byte or nothing is attributed;
  - each changed file put back to that commit ALONE, the census run again, and the lines it moves printed;
  - all tried files put back together, and whether that reproduces the ledger.
  - Exit 0 nothing moved, 1 something moved. The copy is removed before exit: `process::exit` runs no destructors, and the first build would have left a copy of the tree behind on every run with something to say.
- `scripts/ci/test_a_moved_census_names_its_file.py`, a step in `cli-tri`: scratch repositories blessed by the binary under test; clean, moved (two movers, two bystanders, one census moved by only one of them), `--max 1`, and a ledger never committed. Every run must leave the repository byte-identical and TMPDIR empty; the comparison is shown to see a one-byte edit. Four mutations turned it red: put-back a no-op; no drop before exit; only the first moved line; a tried file not restored.
- Census: shell `run:` steps 283 -> 284 and runner bash 262 -> 263, the new step.

## Measured

- On this branch before the bless, `explain` named `.github/workflows/cli-tri.yml` as the only file of 1057 that moves `shell`, with exactly the two lines the gate would have shown one of; all put back together reproduced the ledger. 1057 files in 101 s (about 95 ms each); the default `--max 600` is about a minute.

## Not established

- A change that moves a number only together with another reads as moving nothing alone; the together line is where that shows, not which pair.
- The census code is this build. Putting back a census's own source changes what it reads, not how it counts.
- Files beyond `--max` are counted, not tried.
- `gates shell` reads 0 steps for a step written `- run: x` (the run key first in the list item). No workflow in the repository uses that shape today; a new one would not move the census, and `explain` cannot see a move the census does not make.

Closes #5825
Refs #5786
