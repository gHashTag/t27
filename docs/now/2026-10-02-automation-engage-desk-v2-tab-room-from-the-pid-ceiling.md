# NOW -- automation/engage-desk v2: tab room from the PID ceiling (2026-10-02)

## automation/engage-desk v2: tab room from the PID ceiling, Facebook and Telegram readers (Closes #5583)

- engage-desk VERSION 2: PARALLEL_TABS_MAX 3 -> 8, bounded by tabs_room(open_pages) under PAGE_CEILING 14, which is the pod's PID budget (PIDS_MAX 1000 - PIDS_MARGIN 100 - PIDS_BROWSER_BASE 250) over PIDS_PER_TAB 44, measured on the owner's pod; the desk's tabs live in off-screen windows (DESK_WINDOW_LEFT 6000 > POD_SCREEN_WIDTH 1280), are closed after each read (DESK_TABS_CLOSED) and are skipped by the tab sweep (SWEEP_SKIPS_DESK_TABS); READER_NETWORKS 5 -> 7 (Facebook, Telegram); fns tabs_room, signed_in (Habr by the site, others by the login cookie), telegram_may_comment (discussion group, no write ban, never joins: TELEGRAM_JOINS false); 21 tests
