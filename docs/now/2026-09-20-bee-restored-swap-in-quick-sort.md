# NOW -- A bee restored the function quick_sort lost (2026-09-20)

## swap() is back in specs/tri/sort/quick_sort.t27, with a test (Closes #4286)

- `quick_sort.t27` was edited into parsing at some point and lost `swap`, which its own `sort_range` needs. The bee that took #4286 wrote it back: four lines of body and a `test "swap function swaps two elements"` beside it.
- This entry exists because a pull request must add one, and the bee that wrote the code has no way to know that: its brief names the boundary file and the acceptance criteria, and `docs/now/` is neither. The publisher adds it, names the branch it came from, and says so rather than pretending a bee wrote it.
- This is the first bee branch published since 2026-09-17. 401 `queen-*` branches sit on the remote and the last bee pull request was three days old; of the forty most recent branches, 24 carry real commits, 9 carry only a salvage commit and 7 are empty. Nothing in the swarm opens a pull request -- the Queen skips issues that are pull requests and never creates one -- so every branch a bee pushed has been invisible.
