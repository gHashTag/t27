## A preview of the required checks must read the set, not remember it

`tri gates preview` was written on 2026-09-03 to ask "the four contexts that can block a
merge", and each of its rows was careful: it ran the gate's own implementation, and a row that
could not run said UNAVAILABLE. What it never asked was WHICH contexts block. It wrote four
names into the source, and the header that printed them was a claim about repository
settings that no file in the tree can read.

The ruleset moved. On 2026-09-06 it required `check`, `check-now-freshness`, `validate` and
`check-linked-issue`; its last edit, 2026-09-19 15:06 UTC -- 21 seconds after #4277 merged the
parse ratchet -- left `validate`, `check-linked-issue` and `parse-ratchet`. For two weeks after
that the preview:

- exited 1 with "a required context would refuse this change" on every branch that had no
  docs/now entry yet, over two contexts that could not refuse anything;
- never asked `parse-ratchet`, the one context that could block every merge.

Every row was right about its own subject. The set was the stale part, and nothing could see it
go stale, because the set lives where no diff reaches. docs/BRANCH-PROTECTION.md had the same
defect for longer: five required workflows, of which two posted a required context.

**A list of required checks in the tree is a cache of settings.** Read the settings on every use
(`gh api repos/<repo>/rules/branches/master`), fall back to a ledger that a command regenerates
from them, and print any difference between the two -- or keep no list at all. The same applies
one level down: a reader of a required context must be checked against the job that posts it,
because GitHub matches a required check by NAME. The repair checks two things on every run: that
exactly the reader's workflow posts the context, and that the job's `run:` steps equal the steps
the reader runs. A renamed job, a second workflow taking the name, or an added step each turn the
row UNAVAILABLE instead of leaving a PASS that answers some other question.

And when proving that classic branch protection is off, `branches/master/protection` answering
404 is not evidence: it answers 404 to anyone without admin rights. `branches/master` is
readable by everyone and says `"protection": {"enabled": false}`.
