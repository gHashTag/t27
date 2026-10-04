# NOW -- tri harness, signed in (2026-10-04)

## What was read

- `tri harness` (#5827) measured every width as a fresh, signed-out visitor. On `app.t27.ai/game/browser`, signed out, the BROWSER view is one sentence and one button -- the chat and the browser are behind the sign-in, so the page the Queen board loop works on was the one part it could not see.
- app.t27.ai keeps its sign-in in `sessionStorage`: `trinity.app.session.access` and `trinity.app.session.expires-at` (gHashTag/trinity `apps/website/src/lib/appSessionIdentity.ts`, read on main). Playwright's storage state (1.58 here) carries cookies, localStorage and IndexedDB, and no sessionStorage. A `--storage-state` that only handed the file to Playwright would have measured the signed-out page and printed it under the signed-in name.

## What changed

- `tri harness --save-state FILE URL` opens a visible browser with a temporary profile at URL and measures nothing. The person signs in there; Enter in the terminal writes FILE with mode 600 (`O_CREAT|O_EXCL`), holding Playwright's state plus each open frame's sessionStorage by origin. It never overwrites a file, and it writes nothing when stdin closes before Enter or when the session holds nothing for URL's host. `TRI_HARNESS_HEADLESS=1` is the test's door: CI has no screen.
- `tri harness --storage-state FILE URL` loads FILE read-only into each width's fresh context; its sessionStorage goes in through an init script before the page's own scripts, guarded by `location.origin`, and a key the page already holds is left alone. Refused with exit 2 before a browser starts: a FILE readable by group or others (`chmod 600`), not JSON, not a storage state, an `http(s)`-less URL, or a FILE holding nothing live for URL's host -- another site's, or every cookie expired.
- Values are never printed. The report counts cookies, localStorage, sessionStorage and IndexedDB entries for the host. With `--wait-for SELECTOR` it says `signed in: proven by SELECTOR visible at every width`; without it, `signed in: NOT established`, and NOT ESTABLISHED gains "that the stored session was still signed in". A `--wait-for` that never shows under a stored session hints that the session may be signed out there (expired, revoked, or rotated by an earlier width).
- `scripts/ci/test_a_page_is_measured_at_both_widths.py`: sections 9 and 10, 28 new checks (64 in all). Two new fixtures: `gated` draws a 30px control only when signed in by a cookie, localStorage or sessionStorage (`?want=`), and `signin` signs itself in on load. They check:
  - each store signs the page in;
  - sessionStorage reaches its own origin and not `localhost` beside `127.0.0.1`;
  - a stale cookie measures the signed-out page and says so, and is exit 2 under `--wait-for`;
  - six refusals;
  - the stored value appears in none of 11 runs' output;
  - every state file keeps its bytes and mtime;
  - save -> mode 600 -> sessionStorage present -> round trip signed in;
  - no overwrite, no write on EOF, no write for a page without a sign-in.

## Measured

- The test passes locally on BrowserOS: 64 passed, 0 failed, in 151 s (35 s before these sections).
- Nine mutations of `harness.py` each turned it red. Each is listed with the number of checks it failed:
  - no sessionStorage init script: 4;
  - the init script not guarded by origin: 1, the `localhost` frame signed in;
  - no mode check: 1;
  - no host-coverage check: 2, another site's state and an expired one;
  - `--save-state` without sessionStorage: 2, the saved file and the round trip;
  - the not-JSON refusal echoing the file: 1, the secret appeared in output;
  - the state file touched on load: 1, mtime;
  - `--save-state` overwriting: 1;
  - no "still signed in" note without `--wait-for`: 1.
- The headed `--save-state` path ran once on this Mac against a local page. BrowserOS opened a visible window for six seconds, and Enter saved `1 cookie(s), 0 localStorage, 1 sessionStorage` to a mode-600 file. CI drives this path headless only.

## Not established

- **Never run against app.t27.ai signed in.** Making the file means a person signing in, and the loop does not type passwords. The owner's step: `tri harness --save-state ~/.config/tri/app.json https://app.t27.ai/game/browser`, then `--storage-state` with `--wait-for` on a control only the signed-in BROWSER view draws.
- Whether app.t27.ai's sign-in survives being loaded into two contexts in turn: a token that rotates on use would sign the second width out. `--wait-for` would catch it at that width; it has not been tried.
- The page runs as the signed-in person: whatever it does on load in their name (presence, read marks), a measurement does too.

Closes #5848
Refs #5827
