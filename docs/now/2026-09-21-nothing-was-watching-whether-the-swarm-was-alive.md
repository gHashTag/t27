# NOW -- Nothing was watching whether the swarm was ALIVE (2026-09-21)

## Nine hours of 502, and no instrument read it (Closes #4494)

- The agent server crash-looped, spent Railway's `restartPolicyMaxRetries: 10` - a budget, not a promise - and the platform stopped restarting it. `/queen/status` then answered `{"status":"error","code":502,"message":"Application failed to respond"}` for **nine hours**, with twenty bees' worth of work stopped.
- Every instrument this loop built reads the swarm's OWN numbers. The pusher exits 2 when the status endpoint is silent, which turns one scheduled run red and does nothing else. Nothing asked the only question that matters when a service dies: **does it answer at all?**
- `tools/queen/watchdog.py` asks it every five minutes and is the only thing here allowed to restart the service. It refuses on one bad probe - a deploy answers 502 too, so two probes a minute apart must both fail - refuses without a token rather than pretending to have healed anything, and refuses to ask twice in a row, because a redeploy that did not help is not fixed by another. Its alarm issue closes itself when the swarm answers.
- The probe is tested against the platform's own 502 body, which is valid JSON and is not a swarm: a 200 carrying `workers` is the only yes.
- **The restart budget itself is raised**, from 10 to 10000 in `railway.json`, because a container that can be restarted into health should be.
- **And the leak that caused the crash loop is measured.** This boot: `removing 27 bee worktree(s) left by a previous container (25948 MiB free)` ... `worktrees cleared; 33692 MiB free now` - 7.7 GB of finished bees' worktrees. The emergency sweep kept every one of them because their branches carried commits the base did not, which was right until the branch reached origin; it now reads the remote and removes what is published.
- To have the watchdog restart the service by itself, three secrets are needed: `RAILWAY_TOKEN`, `RAILWAY_SERVICE_ID`, `RAILWAY_ENVIRONMENT_ID`. Without them it alarms and says plainly that it restarted nothing.
