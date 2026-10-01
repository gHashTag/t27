# NOW -- kanban-card-chat: a card on the game's kanban opens its agent conversation (2026-10-01)

## A card opens its conversation with the agent, and every agent continues the same thread (Refs #5388)

- specs/automation/kanban-card-chat.t27 (KIND "automation", REPO 999-multibots-telegraf) is the contract for the owner's request of 2026-10-01: a tap on any card of app.t27.ai/game/kanban opens the person's conversation with the agent about THAT task, with its history, so the task can be carried on from the card.
- The thread is `task:gHashTag/<repo>#<n>`, one per (person, task) in the host's agent_messages; a bare number belongs to the repository the board publishes (it moved from trios to t27 on its own -- measured 2026-10-01: 866 cards, repo gHashTag/t27, 245 of them titled only `#<n>`). Keyed by repository AND number, never by number alone: the close-up defect recorded in the Queen-views entry today is exactly a join by number alone.
- A2A: `metadata.task` or a `contextId` equal to the thread key continues the same conversation; the last A2A_HISTORY_TURNS turns reach the model and the exchange is recorded.
- UI contract: the frame opens a card only through the same-origin `open-card` envelope; the sheet opens at half with the composer pinned; GitHub is a secondary link, never the tap; touch targets at least 44 px.
- test-report 10/10. Negative control: `assert(RUN_LIVE)` FAILs as it must. String constants are pinned by the host test, not here: the Zig backend refuses `==` on `[]const u8` (the first draft was BLOCKED by that, not failing).
- Not measured here: the host code and the deployed board; RUN_LIVE stays false until the host PR is merged by the owner.

## v2: the sheet says the task's work, and the agent's actions wait for a tap (Refs #5388)

- The strip under the title lists the pull requests that name the issue (cross-repository) and the worst state of their checks (`checks_rank`: one failure outranks everything; neutral and skipped count as passed). Read by the person's own client from GitHub's public API, cached STATUS_CACHE_S; the server holds no GitHub token.
- `status_state`: a read that did not happen (the public limit, a network error) is STATUS_UNMEASURED, never STATUS_NO_PR, and the strip says when it can measure again.
- The agent proposes, the person decides: the card chat gets the approve/decline proposal cards (CARDS_ON_KANBAN), `action_runs` only on the tap. MOVE_OFFERED false -- the column is the Queen's verdict and no door could honour a move; AGENT_OPENS_PR false -- no token to do it.
- test-report 14/14. Negative controls: `failed > 9` in `checks_rank` and `return proposed` in `action_runs` each FAIL their test.

## v3: a long timeline is read at both ends; an open costs at most four reads (Refs #5388)

- Measured on a live issue (999-multibots-telegraf#999: 149 timeline events, 58 linked PRs): one page of TIMELINE_PAGE said "+42" for 55, and since the timeline runs oldest first, a fresh open PR on page two would not have been shown at all. Now the first page and the last are read (`pages_read`); when pages lie between, "more on GitHub" is a lower bound (`more_is_lower_bound`, MIDDLE_SAYS_AT_LEAST).
- Check runs are read by the PR's head ref (CHECKS_ONE_READ; `commits/refs/pull/<n>/head/check-runs` returns the same head sha as `pulls/<n>`, checked live). READS_PER_OPEN_MAX = TIMELINE_PAGES_MAX + CHECKS_PRS_MAX = 4, so PUBLIC_READS_HOUR allows at least OPENS_PER_HOUR_MIN uncached opens; v2 cost five.
- test-report 16/16. Negative control: TIMELINE_PAGES_MAX = 1 FAILs both new tests.

## v4: the server reads for everyone; a move is one card through a Queen door that does not exist yet (Refs #5388)

- STATUS_DOOR `/api/card-status`: with GITHUB_READ_TOKEN set, the server answers a card's status with one read (SERVER_READS_PER_OPEN; the newest SERVER_REFS_LAST cross-references with their checks) into a cache shared by every person. Without the token the door says "not configured" and the client reads for itself (`status_source`); a person is still required (STATUS_DOOR_NEEDS_PERSON), and the query is never a mutation (SERVER_READS_ONLY). `refs_partial`: when GitHub holds more references than were returned, the count is a floor.
- S15, the move: read on the Queen's deployed server, a column is computed from the registry, then the dispatch verdict, then the open issue; no route sets one, and the only write that could (PUT /queen/registry) replaces the whole task array. The contract for a one-card door (MOVE_DOOR `/queen/move`, absent): the request names the column the person saw (MOVE_SAYS_FROM), the target is only backlog or dropped (`move_target_ok`), and the Queen answers with a reason either way. MOVE_OFFERED stays equal to `move_offered(MOVE_DOOR_EXISTS, ...)` = false.
- test-report 19/19. Negative controls: `status_source` always SERVER and `refs_partial` as `>=` each FAIL their test; COL_DONE as a person's target FAILs the move test.

## v5: the person's way to the door, the thread live, the nightly look (Refs #5388)

- S16. The Queen's `/queen/move` (BrowserOS#519) admits only the deployment's own credential, so no browser can reach it. MOVE_PROXY `/api/card-move` on render: a hive keeper only (MOVE_NEEDS_KEEPER), the token from MOVE_TOKEN_ENV never reaches a browser (MOVE_TOKEN_TO_BROWSER = false), two taps (MOVE_CONFIRM_TAPS; an armed button disarms after MOVE_ARM_S). `move_allowed(keeper, token, door)` keeps the button off while MOVE_DOOR_EXISTS is false. `move_choice`: backlog and blocked offer "not doing", dropped offers "back to backlog" (the Queen refuses, with her reason, when the drop was her own verdict), running/review/done offer nothing (MOVE_NONE).
- S17. LIVE_DOOR `/api/agent/history/live`: one person-scoped stream per open sheet, resuming after the last id the sheet holds (`live_fresh`), a server recheck every LIVE_RECHECK_S for turns written by another replica, a ping under the proxy's 30 s idle cut, and an end after LIVE_STREAM_MAX_S so the browser reconnects.
- S18. The nightly `tri card-check`: drift lives in ONE issue under the EPIC (`drift_action`: open, comment, close by agreement, nothing).
- test-report 22/22. Negative controls: `move_allowed` without the keeper, `move_choice` offering a drop from running, `live_fresh` as `>=`, and a drift issue that never closes each FAIL exactly their own test.
