# NOW -- The seller asks before it renders (2026-09-14)

## What the first full dry duet showed

- duet-mu027xqr (dry, 4 turns, 00:00 +07, on the #2400 build): state done, 8 of 8 lines, 1 paid call, 0 violations, 1 voice flag. FULL_DRY_RUN_SEEN_LIVE = true. RETRY_SEEN_LIVE stays false: no provider limit this time.
- Line 0 opened with a 70-second paid reel_render and "the same style you liked" -- nothing had been shown or liked. The buyer caught it ("we have not agreed a single sample yet"); the seller took the words back on line 2. The brief said "first 1-2 lines are discovery only"; the model did not hold it.
- Line 2 "ya ne videl" (masculine), line 4 "ya zadavala ... sama ... ne videla" (feminine): the seller's voice drifted between lines.
- Line 6 "bez obeshchanij i bez predskazanij" was flagged for "predskaz": the only flag of the run was a false positive. The plan-6 hook "Ne sluchajno." reads as a verdict about the reader.

## What the spec now says

- crm-duet.t27 DISCOVERY_SELLER_TURNS = 2, DISCOVERY_HIDES_PAID_TOOLS, DISCOVERY_REFUSES_PAID_CALL: on the first two seller turns the paid tools are not offered to the model, and a call that arrives anyway is refused by the dispatcher -- no handler runs, nothing is spent. fn paid_tool_offered.
- FAMILIARITY_MARKERS_RU, FAMILIARITY_CORRECTIONS = 1, FAMILIARITY_MISS_IS_VIOLATION: a line that assumes shared history on a discovery turn gets one rewrite, as the claim check.
- SELLER_GENDER = "m", SELLER_GENDER_IN_BRIEF, GENDER_SLIP_IS_FLAG_ONLY: the seller writes for the owner; a feminine first-person past form is reported, not rewritten.
- CLAIM_NEGATION_EXEMPT, NEGATION_WINDOW_WORDS = 2, PRESSURE_NEGATION_EXEMPT = false: a claim word within two words after a negation is not a hit; pressure words and prices always are. fn claim_hit_counts. HOOK_NE_SLUCHAJNO_FORBIDDEN_IN_BRIEF.

## Not claimed

- DISCOVERY_GATE_SEEN_LIVE = false: a run on the new build with no paid call on lines 0 and 2 is the evidence. CLAIM_CHECK_SEEN_LIVE, RETRY_SEEN_LIVE, RESERVE_CONFIGURED_LIVE, DUET_RAN_LIVE stay false.

## Bookkeeping

- Two test blocks added to crm-duet.t27: published_figures pin 12756 -> 12758.
- Host change: 999-multibots-telegraf #2401 (chat.ts denyTools, tools.ts toOpenAITools deny, crm-duet-tool.ts gate and checks, leela-canon.ts negation window). Closes #3617.
