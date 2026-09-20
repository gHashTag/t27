# NOW -- The last link of the delivery chain (2026-09-20)

## The publisher arms auto-merge itself, and asks for a token that may open a pull request (Closes #4461)

- The publisher's first scheduled run found four bee branches with real work and published **none**: `gh pr create failed: GraphQL: GitHub Actions is not permitted to create or approve pull requests`. The repository setting is `can_approve_pull_request_reviews: false`, and turning it on would let every workflow in the repository open one. A named token for the one job that needs it is the smaller grant, so the job asks for `GH_AGENT_TOKEN` and falls back to `github.token` where that secret does not exist.
- And it now arms auto-merge on each pull request **at the moment it opens it**, while every check is still pending. GitHub refuses `--auto` on a pull request whose checks have already settled into an unstable state - "Pull request is in unstable status" - so the moment to ask is then, not on a later sweep. A refusal is not a failure of the publish: the pull request exists either way and the scheduled merger can still take it.
- Measured while writing this: six bee pull requests published by hand (#4455-#4460), all six for issues **this loop's own untested-function feeder created today** (#4331-#4337). The chain ran end to end for the first time: measure a gap -> file an issue -> a bee takes it -> it pushes a branch -> the publisher opens a pull request -> the gates judge it. #4455 merged.
- Eight more merged earlier from the three-day backlog (#4339-#4347).
- What is NOT established: that the work is good. The gates judge that, and they are the same gates every other change meets.
