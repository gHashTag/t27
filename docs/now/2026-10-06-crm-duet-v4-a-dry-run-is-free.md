# NOW -- crm-duet v4: a dry run is free (2026-10-06)

## crm-duet v4: a dry run is free (Closes #6896)

- specs/automation/crm-duet.t27 VERSION 4: DRY_RUN_SPENDS = false, DRY_RUN_HIDES_PAID_TOOLS, DRY_RUN_REFUSES_PAID_CALL; paid_tool_offered(seller_turn, dry_run) is false on every turn of a dry run; paid_call_refused(seller_turn, dry_run, rewrite).
- Why: owner, 2026-10-06 (translated): "a dry run must be free, change the spec". v3 kept only the story reel out of a dry run; the full dry run duet-mu027xqr made one paid call. Host: gHashTag/999-multibots-telegraf#3794.
- t27c test-report 20/20 pass; negative control: paid_tool_offered ignoring dry_run -> FAIL 2 (the dry-run tests). Seal re-saved with t27c built from this branch, seal --verify MATCH.

## crm-duet v5: a paid tool is one with a price, not one on a list (review of #6897)

- VERSION 5: tool_is_paid(price) is price > 0; borrowed_price(own, borrowed) makes a tool that runs a priced tool priced; priced_tool_offered(price, seller_turn, dry_run) refuses every priced tool on every turn of a dry run. PAID_TOOLS = 6 removed (PAID_TOOL_IS_PRICED, PAID_TOOLS_HAND_LIST = false): the host derives the set from its price table.
- Why: v4 hid only the six tools v1 named; lipsync_generate, story_reel, split_reel and crm_voice_clone charge and were still offered in a dry run (reviewer, t27#6897).
- story_offered now calls paid_tool_offered: the duplicate-bodies gate grouped the two identical bodies.
- t27c test-report 21/21; negative control: priced_tool_offered ignoring dry_run -> FAIL 1 (the new test). Seal re-saved, seal --verify MATCH. dupe_scan: no crm-duet group.
