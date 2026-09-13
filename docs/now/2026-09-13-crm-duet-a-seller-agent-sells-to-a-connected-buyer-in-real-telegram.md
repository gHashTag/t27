# NOW -- crm-duet: a seller agent sells to a connected buyer in real Telegram (2026-09-13)

## What landed

- `specs/automation/crm-duet.t27` (module `automation::crm_duet`, 5 tests): the owner's seller agent writes from his session (144022504) to a connected seller playing a NEW BUYER (435572800, @playom); a persona model answers from HER session. Content is for her real game Leela Chakra (72 planes, entry with a six, ruleset classic, t27.ai/leela, @leela_chakra_ai_bot).
- Why the seller opens: the production business bot answers any DM to the owner but pauses for OWNER_TAKEOVER_MS (30 min) after an owner-typed message; a run that opens from the owner's session keeps it silent, so the buyer is answered once.
- Gates: owner-only tool, buyer must be a connected seller (row in tg_sessions), buyer may not be the owner, background run with a status tool, dry_run sends nothing, session strings never returned.
- Coverage: every seller tool call recorded name -> ok/fail; five paid generations counted; the 1-per-reply / 3-per-run cap is SOFT (`PAID_CAP_IS_HARD = false`) -- prompt plus report, not code.
- Host: 999-multibots-telegraf PR #2362 (`crm-duet-tool.ts`, 12 unit tests with doubles).

## Not claimed

- `DUET_RAN_LIVE = false`: whether a run succeeded live is shown by `crm_duet_status` of a real run, not by the spec.
