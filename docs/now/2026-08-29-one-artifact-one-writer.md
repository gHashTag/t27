# NOW -- One artifact, one writer (2026-08-29)

## One artifact, one writer (Refs #2754)

- the status dashboard was republished three times by the parallel session in one iteration; every merge attempt was stale before it finished
- the publisher correctly refuses a write not built on the current version, which makes the race unwinnable rather than merely wasteful
- rule: the session that publishes owns it, the other contributes through the repo -- and force:true is not the answer, since what it discards is work
