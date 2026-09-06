# NOW -- master was green in two workflows of sixty-three (2026-09-05)

## master was green in two of sixty-three (Refs #3316)

- Twice in one night a report of mine ended with `master зелёный`, naming two workflows.
  Both readings were true. The sentence was not.
- Measured for every active workflow, latest run on master: **40 success, 11 failure,
  12 that have never run there at all.**
- Among the eleven is `Issue Gate`, which supplies `check-linked-issue` -- one of the four
  contexts required to merge.
- The sample was the two workflows I had broken and repaired that evening. A gate you just
  repaired is the most available evidence and the least representative.
- The obvious wider read is still a window: `gh run list --branch master --limit 100`
  returned only **22 distinct workflows** of the 63. A workflow that has not run recently
  is not a workflow that passed, and the window cannot say so.
- The population lives in `/actions/workflows`, asked once per id. That read also expresses
  the third answer -- never ran -- which no count of runs can.
- The correction was not self-generated. A neighbour's issue titled "Every workflow red on
  master" could not both be true and leave my sentence standing. Two claims that cannot
  both hold are the cheapest instrument there is.
- Spooled with `tri skill add` rather than appending a number, so it cannot race another
  branch for one.
