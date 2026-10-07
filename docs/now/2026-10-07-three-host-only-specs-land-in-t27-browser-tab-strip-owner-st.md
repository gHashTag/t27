# NOW -- Three host-only specs land in t27: browser-tab-strip, owner-studio, token-wallet (2026-10-07)

## Three host-only specs land in t27: browser-tab-strip, owner-studio, token-wallet (Closes #7463)

- specs/automation/browser-tab-strip.t27 (v2), owner-studio.t27 (v1) and token-wallet.t27 (v1) were vendored in 999-multibots-telegraf but never existed here, so 999's check-t27-specs --t27 failed t27-same for all three (999#3828). Copied byte for byte from 999 origin/main; Closes #7463, #7464, #7465.
- test-report: browser-tab-strip 10/10, owner-studio 2/2, token-wallet 6/6, FAIL 0. One negative control each went red (POLL_MS 2999, STUDIO_COUNT 4, MARKUP_PCT 199: FAIL 1 apiece).
- token-wallet's warning-days test reports 0 runtime asserts (const-folded), yet WARN_DAYS_FIRST=2 turns it red, so it discriminates. Seals saved. The census re-bless (quiet 181->182, shell 301->302) is master drift from #7399, the same one #7454 carries.
