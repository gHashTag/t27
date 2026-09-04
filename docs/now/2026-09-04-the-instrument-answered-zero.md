# NOW -- The instrument answered zero (2026-09-04)

## `assert true` is 2241 more tests that cannot fail, and a grep said there were none

- The ratchet from #3144 pinned 1813 spec tests whose body is only comments. It missed a second,
  larger shape: **2241** whose only statement is `assert true`. Same ceiling now: **4054** in 33
  files.
- **`git grep -cE '^\s*assert true\s*$'` returns 0.** `-E` is POSIX ERE and does not know `\s`.
  The same population is **2247 lines** under `-E '^[[:space:]]*assert true$'` and under
  `-P '^\s*assert true\s*$'`, and two independent body-walkers then agree on **2241 tests**.
  An instrument that answers zero is not evidence of an empty population.
- This is the third time this trap has fired here, and the doctrine I wrote names the wrong thing:
  it says `\b` is not a word boundary in `git grep -E`. The rule is broader -- **`-E` knows no
  Perl escape at all**, `\s` and `\d` included. A rule stated about one escape does not carry.
- Counting the two shapes apart would suggest they are different problems. They are the same
  problem spelled differently: a file that swaps one for the other has not improved, so they share
  one ceiling.
- **Mutation earned a fixture again.** Deleting the `;?` from `assert\s+true\s*;?` left every
  self-check passing -- no case used the semicolon form, though 65 lines in the corpus do. With
  the fixture, that mutant dies. Three mutants tried, three dead.
- The ratchet failed loudly before re-blessing (`0 -> 1`, `64 -> 144` across 29 files), which is
  its designed behaviour and the demonstration that the ceiling is real.

Refs #3141
