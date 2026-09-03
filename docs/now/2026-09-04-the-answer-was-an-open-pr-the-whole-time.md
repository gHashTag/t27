# NOW -- The answer was an open PR the whole time (2026-09-04)

## `tri topic` asks four sources at once whether someone is already on it

- Twice in one session I started work another session had shipped or had open:
  #3049 (`gates quiet`, `gates empty`) covered two of the four lenses a 29-agent
  fan-out of mine was scanning, and #3056 (`skill refs`) is the tool for a
  dangling cross-reference I had just written a paragraph about.
- The first was an **open pull request** the entire time. The second was a
  merged commit. Both were free to find before starting; what was missing was
  one command instead of four.
- `tri topic <keywords>` reads open PRs, open issues, the last N commits on the
  base branch, and every SKILL.md section title. Rows are ordered by how many
  DISTINCT keywords they carry -- occurrences would rank a title repeating one
  word above one carrying two.
- Verified retroactively: both collisions come out at the top of their query.
- It prints the DISTRIBUTION rather than applying a threshold. `468 of 694` on a
  three-word query is not a result, and a cutoff would need a number nobody has
  measured -- the same mistake that produced a three-state column with no
  correct members earlier today.
- It refuses when `gh` cannot answer. "Nobody else is working on this" and "I
  could not ask" are the same empty list.
- Four mutants, four kills, each by exactly one test: counting occurrences
  instead of distinct keywords, dropping the empty-keyword filter (`contains("")`
  is true of everything), keeping zero-hit rows, and letting a failed `gh`
  return an empty string.
