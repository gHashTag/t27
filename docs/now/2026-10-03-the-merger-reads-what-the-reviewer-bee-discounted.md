# NOW -- The merger reads what the reviewer bee discounted (2026-10-03)

## Review jam: reviewer bee as a service, a merger that can pass, a publisher heading that passes `check` (Closes #5776)

- Measured: `auto-merge-ready-prs.yml` merged 0 pull requests since 2026-09-20; all 300 merges were by hand. Its gate demanded every check green while `spec-guards` is red on master itself.
- The merger now passes a red check only when it is not required by the ruleset, has concluded, and the bot's approval of THAT head carries `discounted-check: <name> -- <why>`. A required check that never posted now blocks; the old gate called such a pull request ready.
- The merger also runs on `pull_request_target: labeled` (`bee-reviewed`), so the bot's label merges at once; the cron stays as the backstop.
- `tools/bees/reviewer.py` is the reviewer bee as a launchd service: the runner gathers the facts (required checks, failing step, log tail, the same check on master), and a `claude -p` with Read/Grep/Glob only and no token judges. The runner then validates the verdict, re-reads the head, and approves as `t27-bees[bot]`.
- Dry runs on #5756 and #5664: about 37 s and $0.20 each. Both discounted `spec-guards` (red on master 1b12580ce) and both refused on `check`.
- The refusal was right. `tools/queen/publish.py` wrote `(published DATE)` as the heading's last parenthesis, which `tools/check_now_entry_shape.py` rejects, so every queen pull request was red on `check`. The heading is now `Published: <title> (DATE)`, and the publisher's self-test runs the real checker on its own entry.
- Not established: that the open queen pull requests are fixed. They still carry the old heading until they are republished or amended.
