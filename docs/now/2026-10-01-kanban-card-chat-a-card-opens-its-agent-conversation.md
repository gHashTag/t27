# NOW -- kanban-card-chat: a card on the game's kanban opens its agent conversation (2026-10-01)

## A card opens its conversation with the agent, and every agent continues the same thread (Refs #5388)

- specs/automation/kanban-card-chat.t27 (KIND "automation", REPO 999-multibots-telegraf) is the contract for the owner's request of 2026-10-01: a tap on any card of app.t27.ai/game/kanban opens the person's conversation with the agent about THAT task, with its history, so the task can be carried on from the card.
- The thread is `task:gHashTag/<repo>#<n>`, one per (person, task) in the host's agent_messages; a bare number belongs to the repository the board publishes (it moved from trios to t27 on its own -- measured 2026-10-01: 866 cards, repo gHashTag/t27, 245 of them titled only `#<n>`). Keyed by repository AND number, never by number alone: the close-up defect recorded in the Queen-views entry today is exactly a join by number alone.
- A2A: `metadata.task` or a `contextId` equal to the thread key continues the same conversation; the last A2A_HISTORY_TURNS turns reach the model and the exchange is recorded.
- UI contract: the frame opens a card only through the same-origin `open-card` envelope; the sheet opens at half with the composer pinned; GitHub is a secondary link, never the tap; touch targets at least 44 px.
- test-report 10/10. Negative control: `assert(RUN_LIVE)` FAILs as it must. String constants are pinned by the host test, not here: the Zig backend refuses `==` on `[]const u8` (the first draft was BLOCKED by that, not failing).
- Not measured here: the host code and the deployed board; RUN_LIVE stays false until the host PR is merged by the owner.
