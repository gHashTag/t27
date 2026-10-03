# NOW -- tri harness (2026-10-04)

## What was read

- Three layout defects the Queen board loop caught on 2026-10-04 -- a 550px live bar at 375px (S3), a 42px close mark and a 12px textarea (S11) -- were each found by a Playwright script written by hand for one pull request and deleted before the commit. None of the contracts could see them.
- `scripts/tri` already dispatches `tri <name>` to `scripts/tri_loop/<name>.py` with no compiler, and `loop-tools-gate.yml` already runs `scripts/ci/test_*.py`.

## What changed

- `tri harness URL [--width 375,1024] [--browser PATH] [--wait-for SEL] [--settle MS] [--timeout S] [--json]` (`scripts/tri_loop/harness.py`). Per width a fresh browser context (under 768 is a phone: touch, coarse pointer, scale 2), and in every visible frame:
  - `viewport-meta` (top document, coarse pointer): absent means the phone lays out at 980;
  - `overflow`: `scrollWidth > clientWidth`, naming up to five elements that push past the edge, never content inside its own scroller;
  - `small-target` (coarse pointer): controls under 44x44, sentence links exempt and counted, 1px screen-reader-only elements skipped and counted;
  - `field-zoom` (coarse pointer): text fields, selects and contenteditables under 16px.
  - Exit 0 clean, 1 findings, 2 could not measure: no `playwright` package, no browser, the load failed or answered HTTP >= 400, `--wait-for` never appeared, a visible frame refused the measurement, or the measurement itself threw.
  - The browser: `--browser`, `$TRI_HARNESS_BROWSER`, Playwright's own Chromium if downloaded, else Google Chrome / BrowserOS / google-chrome / chromium on disk, printed. Headless, temporary profile, and a private TMPDIR of its own removed on exit. `tri harness` downloads nothing; the browser may fetch its own components (see Measured).
- `scripts/ci/test_a_page_is_measured_at_both_widths.py`, a new `harness` job in `loop-tools-gate` (installs the `playwright` package only and drives the runner's Google Chrome). Seven fixture pages served over HTTP from 127.0.0.1, plus a wrapper browser that writes into its TMPDIR; 36 checks; every run must leave its TMPDIR empty, and the check is shown to see one byte. Thirteen mutations each turned it red: `innerWidth` for `clientWidth`; the top frame only; no sentence-link exemption; 12px for 16px; checks on a fine pointer; scrollers not treated as containing; a 404 measured; a failed top-frame measurement not reported as one; a temporary directory left behind; a 0x0 frame measured; 1px elements not skipped; 40px for 44px; the browser launched without a private TMPDIR.
- `harness.py` added to `scripts/ci/loop-tools-tracked.sh` (present, tracked, routed).
- Census: shell `jobs` 85 -> 86, `run:` steps 283 -> 284, runner bash 262 -> 263 -- the new job and its one step. `tri census explain --max 1100` named `.github/workflows/loop-tools-gate.yml` as the only file of 1058 changed since 769f32521 that moves it.

## Measured

- Two defects in the tool itself, each found on its first runs and each now a mutation the test catches:
  - under phone emulation Chromium grows `innerWidth` to the content: a 550px bar at 375 read `innerWidth 550, scrollWidth 550`, "no overflow". `clientWidth` stays 375.
  - `app.t27.ai/game/browser` draws its body inside a same-origin iframe (`/queen/?lang=en#/queen?tab=browser...`). Measuring the top document alone counted the tab bar and called the page clean while a screenshot showed two controls it had never seen.
- `tri harness 'https://app.t27.ai/game/browser?spec=specs%2Fdemos%2Fhello_world.t27' --settle 4000`, BrowserOS headless: clean at 375 and 1024 in both frames -- 9 controls in the top document and 2 in the iframe at 375, none under 44x44, no text field on the page. Signed out, the BROWSER view is a sign-in sentence and one button; the chat and the browser itself are not on the page to measure.
- The CI test takes about 35 s locally.
- The `harness` job's first run on GitHub (98e421f62) passed every functional check on the runner's Google Chrome and failed one: `TMPDIR empty afterwards`. Chrome left `com.google.Chrome.chrome_chrome_url_fetcher_.CHvS5y/fbdd96f4...` behind -- a component download of the browser's own, which BrowserOS on the Mac never made, so the local run could not see it. The browser now gets a private TMPDIR, and section 8 of the test reproduces the leak on any machine with a wrapper that writes the same file; without the fix it is red here too.

## Not established

- First load only: no menu opened, no field focused, nothing typed or scrolled.
- Chromium's mobile emulation, not iOS Safari.
- Anything behind a sign-in -- which on `/game/browser` is the whole BROWSER view.
- What Google Chrome's url_fetcher downloads, and whether it reaches the network on every run: only its leftover was seen.
- The `Corpus ratchet` check is red on this PR and on master's last four pushes (62c105ed2, d995a31ad, 17f09865a, 188884e89): four new conflicted type names, none from this change.

Closes #5827
Refs #5786
