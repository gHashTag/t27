# NOW -- crm-duet v4: a dry run is free (2026-10-06)

## crm-duet v4: a dry run is free (Closes #6896)

- specs/automation/crm-duet.t27 VERSION 4: DRY_RUN_SPENDS = false, DRY_RUN_HIDES_PAID_TOOLS, DRY_RUN_REFUSES_PAID_CALL; paid_tool_offered(seller_turn, dry_run) is false on every turn of a dry run; paid_call_refused(seller_turn, dry_run, rewrite).
- Why: owner, 2026-10-06 (translated): "a dry run must be free, change the spec". v3 kept only the story reel out of a dry run; the full dry run duet-mu027xqr made one paid call. Host: gHashTag/999-multibots-telegraf#3794.
- t27c test-report 20/20 pass; negative control: paid_tool_offered ignoring dry_run -> FAIL 2 (the dry-run tests). Seal re-saved with t27c built from this branch, seal --verify MATCH.
