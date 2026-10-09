# NOW -- blog-share v5: a finished release is asked about at once (2026-10-09)

## blog-share v5: a finished release is asked about at once, posted after a yes (Closes #8083)

- specs/automation/blog-share.t27 VERSION 5: RELEASE_REPO gHashTag/t27, RELEASE_FRESH_MS = 259200000 (3 days), RELEASE_SKIPS_PACE; release_offerable (ours, not a draft or pre-release, pipeline done, young, never asked, newest of its tool), may_ask_release, release_first.
- Why: owner 2026-10-09 (translated) "set it up so that releases are published on X"; asked how, they chose at once but with their yes. Host: gHashTag/999-multibots-telegraf (blog-share desk).
- t27c test-report 14/14 pass (1 vacuous, the same as on master); negative controls: fresh edge '<=' -> '<' FAIL 1, newest_of_tool dropped FAIL 1, may_ask_release without the open-question guard FAIL 1. Seal re-saved.
