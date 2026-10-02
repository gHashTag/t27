# NOW -- The Queen only manages: a reviewer bee gates every merge (2026-10-02)

## The publisher stops arming auto-merge; the scheduled merger wants `bee-reviewed` (Closes #5525)

- Owner's rule of 2026-10-02, overriding "the Queen's accept IS the merge" (2026-10-01, #5422): the Queen assigns, judges, accepts and sends back; she does not merge. A merge happens only after a reviewer bee has reviewed and verified the pull request.
- `tools/queen/publish.py` still opens a pull request for a head she accepted, but no longer runs `gh pr merge --auto`. Its reconcile pass now only disarms auto-merge on open `queen-*` pull requests, whatever her verdict. `--self-test` parses the module's own syntax tree and fails if any `gh pr merge` argv is not `--disable-auto` (20 shapes; a mutation that re-arms `--auto --squash` turns it red).
- `.github/workflows/auto-merge-ready-prs.yml` now skips any pull request without the `bee-reviewed` label, or whose label predates the head commit. The Queen's verdict is not read there. The existing gates stay: L1 issue reference, an APPROVED review, every check green.
- At the time of this change no open pull request in this repository had auto-merge armed (checked with `gh pr list --state open --json autoMergeRequest`), so nothing needed disarming by hand.
- Not established: that the label is applied only by reviewer bees. GitHub cannot tell who is a bee; anyone with triage rights can apply the label.
