# NOW -- automation/blog-share v3: the owner's queue comes first (2026-10-04)

## automation/blog-share v3: the owner's queue comes first; a switched-off network is not asked about (Closes #5916)

- offerable / newest_goes_next: the next question is the first queued article the feed carries and nobody was asked about; the newest such only when no queued one is left -- the owner ranked the articles in their Blogger agent's skill ("the blog starts with its successes") and the desk still asked about the newest article, so the ranking reached the agent and not the desk
- the queue has one home: the owner's skill row, read by the host (999 render blog-share-wiring.ts); the spec holds the rule, not the list
- network_used: a network the owner switched off is neither named in the question nor posted to -- Reddit closed r/t27ai on 2026-10-03
- RUN_LIVE flips to true: on 2026-10-03 08:36 UTC the desk found its post of one-regional-clock-twelve-pull-requests on the owner's X, LinkedIn and Reddit profiles (blog_shares row, state "posted"); the queue order of v3 is not yet measured on a live tick
- test: 12/12 spec tests; negative controls `return in_feed;` (offerable), `return true;` (newest_goes_next) and `return true;` (network_used) each FAIL 1; host controls live in the 999 binding (999-multibots-telegraf#3572)
