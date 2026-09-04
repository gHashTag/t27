# NOW -- `git merge` does not answer with one bit (2026-09-04)

## What a leftover merge cost, and what separates refusal from conflict (Refs #2994)

- a merge left from an earlier iteration sat in the worktree. `git commit` concluded **that** merge, not the change its message described: the subject said "Merge origin/master, split the duplicate 504" and the commit had **one parent**
- every content check agreed the branch was current, and they were right: title sets differed by exactly the two added sections, and the diff with those removed left **three lines**. GitHub still said `DIRTY` -- **a pull request merges histories, and contents are not history**
- the repair script read `if git merge …; then push; else resolve_conflicts; fi` and ran the resolver against a merge that never started; it died in its own assertion and the death read as "bad file"
- **measured on a scratch repo:** genuine conflict `rc=1`, refusal with a live MERGE_HEAD `rc=128` -- and **unmerged paths is 1 in both**. The obvious guard cannot separate them; only the exit code can
- `tri merging` (new): MERGE_HEAD in flight / a subject claiming "Merge" with fewer than two parents / base not an ancestor. Exits 1 on any
- **declined as a history gate:** 3003 commits, 266 subjects beginning "Merge", **one** with a single parent. Squash merging erases the shape before master
- **the tool already existed:** the hand-written poll loop was a worse copy of `tri pr ready --wait --merge`. It counted checks as `(.conclusion // .state)`, and live `fpga-bitstream` had **both null** with `status: IN_PROGRESS` -- "nothing is failing" would have been true with a check still running
