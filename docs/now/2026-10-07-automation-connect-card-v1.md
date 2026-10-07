# NOW -- automation/connect-card v1: a Telegram card to connect Threads and LinkedIn (2026-10-07)

## automation/connect-card v1: one-tap consent card, person-only links (Closes #7480)

- specs/automation/connect-card.t27: one-time links that live LINK_TTL_MS, redeemed only with the owner's signed launch data (never the agent key), a network is carded once a week until connected, no button for a network whose app is not configured, the card carries no token and posts nothing; 8 tests, sealed
- byte-identical to 999-multibots-telegraf apps/vibee-editor/render/src/spec/connect-card.t27 (999 PR #3834); host render src/agent/connect-card.ts
