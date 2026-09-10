# specs/ui -- the site's viewport contract as a declared spec

> **Where this lives.** `specs/ui/` in `gHashTag/t27` is the canonical home of the
> viewport contract of t27.ai -- edit it here. `gHashTag/trinity` keeps a byte-identical
> vendored copy under `apps/website/public/t27/files/specs/ui/`; its build reads that copy
> through the vendored compiler wasm (`t27_compiler.wasm`) and generates
> `src/lib/viewport.generated.ts` and `src/styles/viewport.generated.css` from it
> (`scripts/viewport-from-spec.mjs`). Nothing is parsed with a regular expression and no
> breakpoint is typed twice. The wasm's `typecheck.ok` is necessary, not sufficient (it
> stays true for `assert 1 > 2`), so the generator also evaluates every `test` block of the
> spec against the declared constants and refuses a spec whose own tests do not hold. The
> bootstrap compiler on `master` was not run against this file in the commit that added it.
> A checkout of this repository is referred to as `T27_ROOT` (an environment variable) or
> `git rev-parse --show-toplevel`; no file here names an absolute path.

## Files

| File | Module | Role |
|---|---|---|
| `viewport.t27` | `ui_viewport` | `KIND = "viewport"`: the four tiers by CSS viewport width, the compact-chrome threshold, the pane rule per tier, the touch-target minimum, the six-size QA matrix with the tier of each entry, the desktop reference width, and the Queen rail capacity table (described, not changed) |

## Fields of `viewport.t27`

| Field | Type | Meaning |
|---|---|---|
| `KIND`, `ID`, `NAME` | `str` | `"viewport"`, `"ui/viewport"`, a display name |
| `GENERATED` | `[2]str` | The two files the site derives from this spec (relative to `apps/website`) |
| `TIERS` | `[4]str` | `phone`, `tablet`, `desktop`, `wide` |
| `PHONE_MAX`, `TABLET_MAX`, `DESKTOP_MAX` | `u16` | Inclusive upper bounds in CSS px: `phone <= PHONE_MAX < tablet <= TABLET_MAX < desktop <= DESKTOP_MAX < wide` |
| `COARSE_POINTER_QUERY` | `str` | The media feature that says the pointer cannot hover |
| `HEADER_CHROME_MAX` | `u16` | Below this width the explorer header drops subtitle and note. Not a tier; it sits strictly inside the desktop tier so 1025-1099 renders as before |
| `PHONE_PANES`, `TABLET_PANES`, `DESKTOP_PANES` | `u8` | Panes visible at once: phone shows list **or** card (master-detail), the others both |
| `ONE_SCROLLER_PER_PANE`, `DOCUMENT_SCROLLS` | `bool` | One vertical scroller per visible pane; the document itself does not scroll |
| `TOUCH_TARGET_MIN_PX`, `BACK_CONTROL_MIN_PX` | `u8` | Smallest interactive box per side on phone and tablet; the phone card's way back to the list meets the same minimum |
| `VIEWPORTS`, `VIEWPORT_WIDTHS`, `VIEWPORT_HEIGHTS`, `VIEWPORT_TIERS` | `[6]str`, `[6]u16`, `[6]u16`, `[6]str` | The QA matrix every viewport contract iterates, written as labels and as integers so a test can index them, plus the tier of each entry |
| `DESKTOP_REFERENCE_WIDTH` | `u16` | The width at which the desktop layout must stay pixel-identical across a change |
| `RAIL_*` | `u8` / `u16` / `[6]u8` | The Queen rail's tile sizes, gaps and chrome heights read from `Queen.css` (file:line in the comments) and the capacity per matrix height they predict. Described, not changed: the first consumer is the explorers only |

## Rules

- **Tests inside the spec.** Every invariant the site relies on (monotone bounds, the 44 px
  minimum, every matrix entry in the tier the bounds say, the rail capacity arithmetic) is an
  `assert` in a `test` block of the same file. The site's generator evaluates them; a failing
  assert is a failed build, not a warning.
- **Evidence in the file.** The `SOURCE` comment names the measurement each number rests on
  (live t27.ai, the date, the viewport, what was observed) or the `Queen.css` line it was
  read from. A number without a source does not belong here.
- **Comments.** Top-level comments use `;`. Inside a `test` block use `//` only: the parser
  reads a `;` line inside a block as a statement, and the site's generator reports it.
- **English only, ASCII only** (LANG-EN, L3). UI copy is not in this spec; the site's
  Russian travels through its i18n bundles, which this spec does not name because it carries
  no user-visible text.
- **Do not edit the generated files by hand.** Change this spec, re-vendor it byte-identical,
  re-run the generator; the generated files carry the spec's sha256 in their header.
- **Not a Queen spec.** The rail and HUD numbers are recorded so a QA contract can report
  them; changing the rail is a separate round with its own approval.

## How the site uses it

```
T27_ROOT/specs/ui/viewport.t27
  -> trinity apps/website/public/t27/files/specs/ui/viewport.t27   (byte-identical copy)
  -> node scripts/viewport-from-spec.mjs                            (compiler wasm, asserts evaluated)
  -> src/lib/viewport.generated.ts     tiers, tierOf(width), tierQuery(tier), VIEWPORTS
  -> src/styles/viewport.generated.css :root custom properties (touch minimum, rail tiles)
  -> src/lib/useViewport.ts            { tier, width, height, coarsePointer } via matchMedia + resize
  -> qa/explorer-viewport-contract.mjs the six explorers at the six sizes, headless Chrome
```

`npm run check:viewport` fails when the committed generated files are not the ones the
vendored spec produces; `npm run check:explorer-viewport` runs the browser contract.
