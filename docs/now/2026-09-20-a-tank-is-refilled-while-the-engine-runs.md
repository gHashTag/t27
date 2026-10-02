# NOW -- A tank is refilled while the engine runs (2026-09-20)

## The feeders now top the queue up whatever the lanes are doing (Closes #4310)

- `queue_idle()` asked one question -- are there free lanes? -- and at ten of ten it answered no and the feeder printed `swarm busy - nothing added`. Ten of ten is when the queue is emptied fastest, and it is exactly the state the pusher's new `fuel-runway` rule dispatches a feeder for. The loop built an hour earlier had a hole in the middle of it.
- `--runway N` is the floor: top the queue up to about N dispatchable issues whatever the lanes are doing. Both feeder workflows pass `--runway 24`, which is a little over two rounds for ten lanes.
- Dispatchable is an estimate and the log says so: open issues carrying a `## Boundary`, minus the tick's `claimed` and `completed` counts. The tick caps the issue lists it prints, so the exact set cannot be subtracted -- only the counts. Measured live while writing this: `about 121 dispatchable issue(s) against a floor of 24 (121 carry a boundary, 0 claimed, 0 completed)`, so nothing was added, which is the right answer.
- One change serves both feeders: `feed_untested.py` imports `queue_idle` from `feed_empty_bodies.py` rather than carrying its own copy.
- What this does NOT do: it does not raise the ceiling on how much work exists. When 221 uncovered specs run out, the runway floor will ask for issues no generator can produce, and the log will say `0 uncovered` rather than pretending. That is the next pool's problem, not this one's.
