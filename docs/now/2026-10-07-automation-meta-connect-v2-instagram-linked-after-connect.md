# NOW -- automation/meta-connect v2: an Instagram linked after Connect is found again (2026-10-07)

## automation/meta-connect v2: re-read the Page's Instagram account (Closes #7346)

- specs/automation/meta-connect.t27 v2: IG_RELINK_EVERY_MS (10 min), IG_RELINK_NEEDS_CONSENT=false, IG_PUBLISH_RETRIES=1; should_relink, retry_after_relink; 14 tests, sealed
- host: 999-multibots-telegraf render src/agent/meta-connect.ts (relinkInstagram, postByApi, the status reads)
