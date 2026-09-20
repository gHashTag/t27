# NOW -- Measure delivery, not dispatch (2026-09-20)

## Two rules that would have caught a three-day outage on its first hour (Closes #4324)

- `nothing-lands` counted every merged pull request in the repository, so one operator merge suppressed it entirely. Measured today: **seventeen merged in twenty-four hours, none of them from a bee**, while the swarm ran at ninety percent utilisation and every instrument reported health. It now counts only branches named `queen-*`, and says the total beside it so the difference is visible.
- **`nothing-published`** is new: bees running, no bee pull request open, none merged in six hours. That is the exact shape of what happened -- 401 branches on the remote and nothing opening a pull request for any of them -- and no rule in this file could see it, because every number was about DISPATCH and none about DELIVERY.
- Both are report-only. The repair is `tools/queen/publish.py`, which is a separate change with its own refusals.
- Twenty-seven rule shapes now, ten of them ones a moving system must NOT fire -- including a swarm whose work lands, which must leave `nothing-published` silent.
- Measured live while writing this: `bee_merged_last_6h: 1` (the probe that landed), and the reading also fired `lanes-idle` and `fuel-runway` at once: **about 12 dispatchable issues for 20 lanes**, because 122 open issues carry a boundary, 67 are claimed by attempts that are not running and 43 are completed. The parked claims are the next thing to look at, and they are already named by `work-parked`.
