# NOW -- The duet seller may not claim work it did not do (2026-09-13)

## What the first dry duet from the dashboard showed

- duet-mtzyg2t6 (dry, 4 turns, started from /crm/435572800): turn 0 said "I already built a trial reel in your style and am preparing it for publication" while the only tools called were crm_client_profile and crm_lead_context. No reel existed.
- Turn 2: every model provider answered with a rate limit, the seller produced no text, the loop broke, and the run was stored as done with 3 of 8 lines and no error. The dashboard read "finished, 0 violations".
- duet-mtzy6kao, started 21 s before a vibee-render redeploy, has 0 lines and stays running until the 30-minute lost threshold: the live witness that RUN_SURVIVED_REDEPLOY is false.

## What the spec now says (specs/automation/crm-duet.t27)

- CLAIMED_WORK_NEEDS_TOOL: a claim of finished work is honest only when the same turn called a producing tool or forwarded media; one rewrite round (CLAIM_CHECK_CORRECTIONS = 1), both misses are violations.
- ABORTED_RUN_STATE = "failed", ABORTED_RUN_KEEPS_ERROR, DONE_MEANS_ALL_LINES: a run that did not play every turn is failed and carries the turn error; RUN_ERROR_SHOWN_ON_DASHBOARD.
- DRY_RUN_SEEN_LIVE and RUN_KILLED_BY_REDEPLOY_SEEN are true; DUET_RAN_LIVE (a real, non-dry run) stays not claimed.
- Host: gHashTag/999-multibots-telegraf#2398 (claimsDoneWork, producedWork, failed state, error on the client dashboard). Live behaviour after deploy is not yet verified.

Closes #3613
