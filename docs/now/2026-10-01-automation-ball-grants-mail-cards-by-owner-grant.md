# NOW -- the ball board's mail cards, to others only by the owner's grant (2026-10-01)

## specs/automation/ball-grants.t27 -- one exception to MAIL_OWNER_ONLY, and what it never opens

- Owner's ask: every task from the mail visible to the owner when signed in on
  /game/kanban, and to others only if the owner allowed it -- with privacy and
  security sorted out.
- A grant is one row: a viewer by telegram id (a username can change hands), a
  client as linked with `ball_link` or `*` for every linked client, and an
  expiry. Only the owner grants; the owner needs no grant and cannot get one.
  Unsized is 30 days; nothing outlives 90.
- What a grant never opens: the github column (private repos stay the
  owner's), a matter no link names (it has no client to be granted), the
  counterparties' addresses, the owner's links (applied on the server), counts
  of cards that were not sent, and anything over the t27.ai game token.
- Refusal still comes before the read: a caller with no live grant never causes
  the mail digest to be selected.
- Every viewer read stamps `last_read_at`; `ball_grant list` shows the owner who
  looked and when.
- Negative controls: dropping the unlinked check fails "a grant covers its
  client..."; dropping `!viewer_is_owner` fails "only the owner grants...".
- Host: 999-multibots-telegraf render `src/agent/ball-grants.ts`, gate in
  `src/agent/ball-board.ts`, bound by `src/spec/ball-grants.test.ts`.
  RUN_LIVE = false.
