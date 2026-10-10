# NOW -- spec: browser-viewer-boot v1 -- the live-browser window's boot clocks, language and sound (2026-10-07)

## spec: browser-viewer-boot v1 -- the live-browser window's boot clocks, language and sound (Closes #7348)

- New specs/automation/browser-viewer-boot.t27 for 999-multibots-telegraf render src/browser/skin.ts: fast fallback 8000 ms to frames mode, final give-up 45000 ms, frames-mode memory per session cleared on play, lang=ru|en whitelist, inline and muted unless sound=1
- Laws as fn (fast_before_give_up, mode_at, remembered_after, lang_kept, muted); test-report 8/8 pass, 0 vacuous; negative control BOOT_FAST_WATCH_MS 8000 -> 50000 fails 3 of 8
- The 999 host imports every value from t27c gen-ts and runs each spec test against the injected viewer scripts
