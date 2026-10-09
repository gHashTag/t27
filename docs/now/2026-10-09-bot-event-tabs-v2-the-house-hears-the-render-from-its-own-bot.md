# NOW -- bot-event-tabs v2: the house hears the render from its own bot (2026-10-09)

## bot-event-tabs v2 (Closes #8139)

- specs/automation/bot-event-tabs.t27 VERSION 2: via_house_bot(for_house, house_token_known), tries_service_bot(via_house, house_delivered).
- Why: owner 2026-10-09 (translated) "why does the bot send in another bot? we have one main one now!!" -- the render's questions and reports left from TELEGRAM_BOT_TOKEN (@neuro_blogger_bot) instead of @t27ai_bot, crm-story-reel's HOUSE_DELIVERY_BOT. Host: gHashTag/999-multibots-telegraf telegram-sender.ts.
- t27c test-report 4/4 (8 runtime asserts in the new test; the two vacuous tests are constant-only, as on master); negative controls: via_house_bot without for_house FAIL 1, tries_service_bot always true FAIL 1. Seal re-saved.
