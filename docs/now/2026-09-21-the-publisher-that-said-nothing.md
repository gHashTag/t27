# NOW -- The publisher that said nothing (2026-09-21)

## queen-publish reads stdout, retries, and says why a gh call failed (Closes #4514)

- From 2026-09-20 19:02 every scheduled run of queen-publish exited 2 with "`gh issue list` returned nothing". Sixteen hours, no bee pull request opened. The same command answered in ten seconds from a laptop, and a local dry run found five branches ready to publish.
- `gh_json` parsed stdout and stderr joined, and on any failure returned an empty list without a word. A warning on stderr, a 502 on a thousand issue bodies and an expired token all read as "nothing".
- Now: stdout only, three attempts with backoff, and every failure logged with its exit code and stderr.
- What this does NOT establish: which of those causes it was. The next run's log will say.
