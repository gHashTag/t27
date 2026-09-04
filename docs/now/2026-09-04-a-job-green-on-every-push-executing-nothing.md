# NOW -- A job green on every push, executing nothing (2026-09-04)

## A job green on every push, executing nothing (Closes #3116)

- notebook-sync.yml's sync-activity greps the last commit for a ROOT activity.md that has 0 commits in the whole history and 0 tracked files; the three steps behind it have never run
- sync.py sits three separators deep and used five .parent calls -- one level ABOVE the repository, which on a runner is /home/runner/work and not a git repository; four cwd=REPO_ROOT git calls ran there
- Fixed to four, with a marker guard that refuses rather than reading a directory nobody meant, and a checker that reads the assignment against the file's own depth
- The sweep was measured and declined: 41 assignments name a repo root, 33 are exactly right, and all 8 flags are artifacts -- seven parents[N], one TEST_ROOT. The class has one member
