## I said master was green after reading two workflows of sixty-three

Twice in one night a report of mine ended with a line like

    master зелёный: перепись PASS, cli-tri и Untrusted Input Gate — success

Both readings were true. The sentence was not. The repository has **63 active
workflows**, and I had named two — the two I had happened to break and repair
that evening, which is exactly why they were the ones in front of me.

Measured properly, latest run on master for every active workflow:

    success        40
    failure        11
    never ran      12

Among the eleven was `Issue Gate`, which supplies `check-linked-issue` — one of
the four contexts required to merge.

Two separate errors, and the second is the one worth keeping.

**The sample was the ones I had touched.** A gate you just repaired is the most
available evidence and the least representative: its greenness is a statement
about your own afternoon, not about the branch.

**The obvious wider read is still a window, not a population.**
`gh run list --branch master --limit 100` returns the last hundred RUNS, and a
hundred runs held only **22 distinct workflows** — a third of them. The other 41
had not run recently enough to appear, and a workflow that has not run is not a
workflow that passed. The population lives in
`/actions/workflows`, and the per-workflow question has to be asked once each:

    gh api 'repos/OWNER/REPO/actions/workflows?per_page=100' | jq '.workflows[] | select(.state=="active")'
    # then, per id:
    gh api "repos/OWNER/REPO/actions/workflows/$id/runs?branch=master&per_page=1"

That read also separates a third answer the window cannot express: **12 workflows
have never run on master at all.** They are neither green nor red, and rolling
them into either number is the failure this file is full of.

The correction was not self-generated. It came from noticing a neighbour's issue
titled *"Every workflow red on master"* while looking for something else — a
title that could not both be true and leave my sentence standing. Two claims that
cannot both hold are the cheapest instrument there is, and the only reason this
one fired is that I read a list I did not need.
