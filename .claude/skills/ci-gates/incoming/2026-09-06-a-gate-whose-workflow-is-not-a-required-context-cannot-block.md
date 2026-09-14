## A gate whose workflow is not a required context cannot block a merge

I published, in the body of #3332, that "the pre-commit hook has been failing for every author
since a workflow edit landed unblessed". I generalised it from a single observation and never
measured it.

Replayed `census pin --gate` over the last twelve first-parent commits of `master`, with the tool
itself unchanged across the window (`cli/tri/src/census.rs`: 0 commits, so today's binary is not an
anachronistic ruler). **Exactly one commit fails** -- `3429f9c6` -- and the next commit blesses it
nineteen minutes later. One commit, not an era. Corrected at source, and verified by re-fetching
the body rather than trusting the write.

The replay was worth more than the claim it replaced:

- `tri census pin --gate` runs in `cli-tri.yml:161`. The four required contexts are `check` (a job
  inside `check-now-freshness.yml`, not a workflow of that name), `check-linked-issue`,
  `check-now-freshness` and `validate`. **`cli-tri` is not among them**, so a red census cannot
  block a merge.
- A GitHub squash-merge **never runs a local hook**, so the pre-commit census guard binds local
  commits only.
- `3429f9c6` merged red for both reasons at once. That is also why the pin oscillated
  `235 -> 234 -> 235` in under an hour: one commit blessed a reduction the next reverted.

Before calling a gate "required", read the ruleset -- `master` here is under a ruleset, not classic
protection, so `branches/master/protection` returns 404 and proves nothing. Changing the ruleset is
the owner's decision, so this is reported, not fixed.
