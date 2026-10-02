# NOW -- Published figures follow the corpus (2026-10-02)

## Re-pin nine figures that drifted (Refs #5497)

- `published_figures.py --check` reported 9 of 10 figures drifted on master 769f3252. No matcher changed in the file's history; only the corpus moved.
- No pin had moved since d3224e69 (2026-09-16), and five had already drifted at that commit: the merge that resolved this file's conflict kept the older numbers. Since then specs/ grew from 946 to 1146 files, 200 of them under specs/port/, which did not exist then.
- Each pin now equals the value at 769f3252, and its note gives the value at d3224e69 and the split of the change between specs/port/ and the rest. The largest move: `x.len` field reads 680 -> 2197 (1075 at d3224e69; +492 outside specs/port/, +630 inside). `--check` and `--self-check` pass.
