# NOW -- Two things that had to be true before a gate could be required (2026-09-07)

## Two things that had to be true before a gate could be required (Closes #3386)

- `tri census pin --gate` lives in `cli-tri.yml`, which is not one of the four required contexts -- so a red census cannot block a merge, and squash-merge never runs the local hook either.
- Making it required would have broken the repo twice over: the job was displayed as `build`, a name TWO other workflows also emit, and it was paths-filtered -- so 15 of the last 40 merged PRs (38%) would have hung forever on a context that never posts.
- Both fixed and priced: unique context `cli-tri`, no PR paths filter, +38% runs on a 2.9-minute job that is green 58 of its last 60. The ruleset is untouched -- that click is the owner's.
