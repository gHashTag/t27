# NOW -- Price what you added, and the price will find a defect (2026-09-05)

## Price what you added (Refs #3310)

- Section 591. Three gates went into the barrier in one pass and none was priced.
- The measurement found a defect no control had: the push hook does not read stdin, so it
  cannot tell a push from a deletion, and `git push --delete` was refused while the
  branch survived on the remote.
- It also found a case where being FAITHFUL to the CI gate was wrong: an empty range is
  refused by the CI script, but that case cannot arise in CI and arises constantly locally.
- The number was the opposite of the expectation. The barrier is 329 ms and got about three
  times cheaper, because correcting the conflict gate to read the index -- done purely for
  correctness -- took 876 ms down to 93.
- A correctness fix can be a performance fix, when the wrong operand was also the larger
  one: 7951 tracked files read to judge the 3 that were staged.
