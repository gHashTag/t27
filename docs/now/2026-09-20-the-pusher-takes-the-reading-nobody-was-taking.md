# NOW -- The pusher takes the reading nobody was taking (2026-09-20)

## A scheduled reading of what can stop the swarm, compared with the last one, with a rule per stall and the command that would falsify it (Closes #4282)

- Three stalls, one shape. 2026-09-17: the feeder fired twice in twelve hours instead of 36, because the machine slept. 2026-09-19: the batch merge had failed every run for two days in three seconds on a model id, writing no log. 2026-09-20: 71 issues were held by attempts that had spent their retries, 0 of 8 lanes ran, and the tick refused 673 candidates with "nothing to choose". Each time the evidence existed and the operator found out by asking.
- `tools/queen/pusher.py` reads the Queen's status and board, the open issues and bee pull requests, what merged in the last six hours, and the Actions queue; it compares that with the previous reading and applies rules named for the stalls that produced them: idle-with-fuel, out-of-fuel, work-parked, prs-conflicted, nothing-lands, ci-queue, not-evolving.
- Its memory is its own issue. The reading is written back as JSON inside the issue body and read from there next time, so the history is where anyone can read it and no state file is committed to master.
- Every rule carries the command that would falsify it, and the self-test exercises eleven shapes - including three a moving system must NOT fire, because a watchman that cannot be shown to cry wolf is one nobody believes when it does.
- Verified against the live system before landing: it read 8 bees running, 0 issues claimed, 673 open issues, 0 queued workflow runs, and fired exactly one rule - "nothing has merged in six hours while 19 bee pull requests are open", which was true and is the conflicted-branch backlog.
- What it does not do: judge code, merge, or close anyone's work. It measures, compares, and files.
