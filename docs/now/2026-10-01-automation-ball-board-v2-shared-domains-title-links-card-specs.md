# NOW -- ball board v2: no link may claim strangers, the blog gets a door, every card names its spec (2026-10-01)

## specs/automation/ball-board.t27 v2 -- from the first live read (crm 6, mail 134, github 71)

- hh.ru was a counterparty on 50 unrelated mail matters. A domain link there would
  have filed all 50 under one client, so a domain link on a domain many strangers
  share (job boards, ATSs, mailbox providers, lists, submission systems) is now
  refused: `shared_domain`, `may_link`. An email link at the same domain is fine.
- Code work could reach a client only through a label or its repository, and the
  blog's PRs share their repository with everything else, named "blog: ...". The
  new `title` link claims work by the title's HEAD -- first word, lower-cased, cut
  at ':' or space -- so "fix(blog): typo" is not claimed by "blog". It is weighed
  after the label and before the repository.
- Owner's ask: every kanban card stands on a .t27 spec. `card_spec`: the path a
  title names wins; else the source's spec of record (CRM ->
  crm-client-workspace.t27, mail -> mail-push.t27); code work that names none has
  none, and the board COUNTS it instead of inventing a link.
- Found while testing: `t27c test-report` is BLOCKED on every spec that compares
  strings (`==` on `[]const u8` in the Zig backend) -- v1 of this spec and
  mail-push included. Their tests are exercised only by the host's binding tests.
- Claim unchanged: `RUN_LIVE = false`.

## v3 -- the agent's browser is a source, and only ever the caller's own

- Owner's ask: see on the board what the agent is doing in the AI browser, open
  that browser at any moment to help or steer it, and keep people safe giving
  their data.
- `SOURCE_BROWSER = "browser"`: the caller's own `browser_sessions` row
  (`BROWSER_OWN_ONLY`, `WHERE telegram_id = caller` -- the owner included), its
  open ask (input request or approval, `ASK_TTL_MIN = 10`) and its last
  non-probe journal step.
- `browser_ball`: an open ask or the person holding the wheel is ours; a live
  browser asking nothing is the agent's move, theirs; a stopped one is no card.
- Privacy laws, each a test: `browser_field_shown` names the three columns a
  card may carry (state, last_seen_at, person_wheel_until) and refuses tokens,
  endpoints and the pod by default; typed text is a length; probes are not work;
  the card opens `/game/browser`, the app's own view, never a `/live/` URL.
- Its spec of record is `specs/automation/browser-sign-in.t27`.

## v4 -- the caller's own meetings are a source (t27#5416)

- Owner's ask: the board is the control centre of the whole game; the meetings the
  agent books (`specs/automation/meeting-reminders.t27`) belong on it.
- `SOURCE_MEETING = "meeting"`: the caller's own `meetings` rows only
  (`MEETING_OWN_ONLY`), status `planned`, starting within `MEETING_AHEAD_H = 168`
  hours or ended within `MEETING_ASK_BACK_H = 72`.
- `meeting_ball`: ended and unanswered is due; ended and answered is no card's
  move; starting within `MEETING_SOON_H = 24` is ours; later is none.
- Privacy laws, each a test: `meeting_field_shown` names the five columns a card
  may carry (title, starts_at, ends_at, mode, status); the tap and calendar
  tokens, the call link and the place never reach a card
  (`MEETING_CARD_NO_TOKEN`).
- `t27c test-report` stays BLOCKED by #5162 (string `==`); the host's binding tests
  exercise every title. Claim unchanged: `RUN_LIVE = false`.

## v5 -- mail only machines or the owner wrote is nobody's move (t27#5419)

- Live board: 115 mail cards on us. jcrm says "we owe" whenever the newest message
  is inbound, and a machine's message is always inbound: DMARC reports, noreply
  notices and a publisher's sign-in mail sat on the board as replies owed, and so
  did the owner's own domain.
- `automated_marker` names the markers (noreply, donotreply, mailerdaemon,
  postmaster, bounce, dmarc, notification), looked for in the address with
  . - _ + removed. `nobody_to_answer`: every address is a machine's or
  `OWN_DOMAIN`; a matter with no address is not judged.
- `mail_ball_weighed`: nobody to answer is `none`, otherwise jcrm's decision
  stands. Such matters are counted (`AUTOMATED_IS_COUNTED`), never dropped.

## v6 -- a relay's no-reply speaks for a person (t27#5423)

- v5 read live: 53 matters machine-only, 48 of them from `noreply@hh.ru`. hh.ru
  relays an employer's message through its no-reply address, so v5 moved live
  conversations off "on us". That was the spec's mistake, not jcrm's.
- `relay_domain`: an address on `RELAY_DOMAIN` (or under it) is never a
  machine's. `RELAY_SEEN` records the 48.
