# NOW -- dupe scan green: four copied bodies reuse their originals (2026-10-07)

## dupe scan green: four copied bodies reuse their originals (Closes #7355)

- WAL_deinit returns WAL_checkpoint(wal); getRecentHistory calls searchHistory with an empty query; is_blinking_success calls is_success_status.
- review_log.t27 imports str_contains from test_framework::runner; its str_starts_with had a contains-scan body and is now a real prefix check.
- tools/dupe_scan.py: 624 -> 615 duplicate bodies, 185 -> 181 groups, no new group; test-report FAIL 0 on all four specs, seals saved.
