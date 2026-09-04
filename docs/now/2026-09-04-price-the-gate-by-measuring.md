# NOW -- Price the gate by measuring what moves the number (2026-09-04)

## The worry was imagined; the measurement refutes it

§518 left a question: should *"a change that moves a census must say so"* be a gate, a snapshot, or
a habit? The stated worry was that an always-on gate over ~10 numbers reddens constantly and gets
muted. **Measured over the 39 most recent transitions on master, with one fixed instrument so tree
drift is separated from tool drift:**

    transitions                        39
    moved at least one census           8   (20%)
    of those, the commit SAID so        4
    per census   fetches 4   shell 4   quiet 1

**Every one of the 8 had edited that census's own subject** -- fetches 4/4 in `cli/tri/src`, shell
4/4 and quiet 1/1 in `.github/workflows`. Not one moved as a side effect. That is **structural**:
each census's population IS a directory, so it cannot move unless a file there changes. The 8/8
confirms the structure rather than proving it.

So the re-bless falls only on commits already working in that area. **The tax does not exist, and
finding that out cost one loop over `git checkout`.**

## `tri census pin`

Three pure, deterministic censuses (`fetches`, `quiet`, `shell` -- 1.1s, 0.1s, 0.1s, byte-identical
across three runs, no dates or absolute paths) are pinned as **whole output** under `tools/census/`.

- **The output, not numbers parsed out of it.** Parsing a tool's own report to check the tool is the
  re-implementation trap one layer up. A byte comparison cannot have that bug, and the failure
  prints the diff.
- **Exclusions are measured and enforced.** `dead` and `unmeasured` read the GitHub API -- their
  answers move when the *world* moves, so pinning them would redden on somebody else's push. A test
  refuses their addition; adding `dead` turns it red.
- **Absence is not amnesty.** A missing ledger fails with its own message, not the moved-census one.

**Historical control, which is the whole argument.** Bless at the parent of the commit whose move
was silent, then check that commit out: `PASS` becomes
`FAIL: fetches moved / was 56 / now 59`. It would have caught the miss that made 45 of 50 red
workflows invisible.

## Two things running it caught

- **The trigger was narrower than the subject.** `cli-tri.yml` fired on `cli/**` and not
  `.github/workflows/**`, which is the subject of two of the three pinned censuses. Left alone, a
  workflow-only commit moves `shell`, the gate does not run, the ledger goes stale, and **the next
  `cli/**` commit fails blaming an author who changed nothing.** Misattribution is a correctness
  bug, not a cost. Priced before widening: 24 of the last 200 commits touch `.github/workflows/`,
  2 of those already touch `cli/`, so it adds this job on **22 of 200** (11%).
- **The rule bit its author on arrival.** Adding the gate's own step moved `run: steps` **229 ->
  230**, so this commit re-blesses its own ledger. The ledger's byte length did **not** change --
  only the diff showed it. Third time this session a size match was weak evidence, and the first
  time I blessed before checking and had to redo the reading.

Refs #3176
