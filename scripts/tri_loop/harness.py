#!/usr/bin/env python3
r"""tri harness -- a page measured at 375px and 1024px: overflow, tap targets, field zoom.

Every layout defect the Queen board loop shipped and then caught (cron 8782e5f8,
2026-10-04) was invisible to the contracts and visible to a browser in seconds:

  S3   the live bar was 550px wide on a 375px screen (`scrollWidth` 550);
  S11  a lone close mark measured 42px across, a textarea was set at 12px --
       iOS zooms into any text field under 16px the moment it is focused.

Each of those was found by a hand-written harness: a throwaway Playwright script,
a viewport, a few `getBoundingClientRect` calls, deleted before the commit. The
recipe lived in a skill as six bullet points. This is the recipe as one command,
so the next pass runs it instead of re-typing it.

    tri harness https://app.t27.ai/game/browser
    tri harness file:///tmp/page.html --width 375,768,1024
    tri harness URL --wait-for '#root main' --json

WHAT IT MEASURES, PER WIDTH
---------------------------
A width under 768 is a phone: touch, coarse pointer, device scale 2, height 812.
768 and over is a desktop: mouse, fine pointer, height 768. Each width gets a
fresh browser context -- no cookies, storage or cache carried between them.

  viewport-meta  no `<meta name="viewport">`: a phone lays the page out at 980px
                 and shrinks it, so every other number at that width is a 980px
                 number. Checked on a coarse pointer only.
  overflow       `scrollWidth` wider than `clientWidth`: the page scrolls sideways.
                 Names up to five elements that push past the edge. Content
                 inside its own horizontal scroller is contained, not overflow,
                 and is never named. NOT `innerWidth`: on a phone Chromium grows
                 `innerWidth` to the content (a 550px bar at 375 reads
                 innerWidth 550, scrollWidth 550 -- "no overflow"), while
                 `clientWidth` stays 375. Measured on the first run of this tool.
  small-target   a link, button, field or ARIA control under 44x44 CSS px on a
                 coarse pointer (Apple's minimum; WCAG 2.5.5). A link inside a
                 sentence is exempt (WCAG 2.5.8) and COUNTED, not hidden;
                 elements of 1px or less (screen-reader-only) are skipped and
                 counted too.
  field-zoom     a text field, select or contenteditable under 16px on a coarse
                 pointer.

EVERY VISIBLE FRAME
-------------------
app.t27.ai/game/browser draws its whole body inside a same-origin iframe
(`/queen/`). The first run of this tool measured only the top document -- the
tab bar -- and called the page clean while a screenshot showed two controls it
had never seen. So each visible frame is measured as its own document and
printed under its URL; frames of 1px or less are skipped and counted.

Exit 0: nothing found at any width. Exit 1: at least one finding. Exit 2: could
not measure -- no Playwright, no browser, the URL did not load or answered an
HTTP error, or a visible frame refused the measurement. An error page measured
and called clean is the lie this refuses.

WHICH BROWSER
-------------
`--browser PATH`, else `$TRI_HARNESS_BROWSER`, else Playwright's own Chromium if
it was downloaded, else the first of Google Chrome, BrowserOS, google-chrome,
chromium found on disk. The one used is printed. It always runs headless with a
temporary profile that Playwright deletes on close: a person's own profile,
cookies and sign-ins are never opened. This tool downloads nothing -- but the
browser may fetch its own components: Google Chrome on GitHub's runner left
`com.google.Chrome.chrome_chrome_url_fetcher_.*/<hash>` in TMPDIR after every
run (the CI test's empty-TMPDIR check caught it on its first run there). So the
browser gets a private TMPDIR of its own, removed when this exits.

NOT ESTABLISHED
---------------
First load only: no menu opened, no field focused, nothing typed or scrolled, so
a defect that appears after an interaction is not seen. Chromium's mobile
emulation, not iOS Safari. Content behind a sign-in, or hidden when the page
settles, is not measured. A clean run says these four checks found nothing on
this load; it does not say the page works on a phone.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import tempfile

MOBILE_BELOW = 768

CANDIDATES = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/BrowserOS.app/Contents/MacOS/BrowserOS",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
]

NOT_ESTABLISHED = (
    "first load only -- no menu opened, no field focused, nothing typed or "
    "scrolled; Chromium mobile emulation, not iOS Safari; content behind a "
    "sign-in or hidden when the page settles is not measured"
)

# One pass over the page, in the page. Returns plain data; Python decides nothing
# the page could not see. MIN_TARGET and MIN_FIELD are substituted below so the
# numbers live in one place.
AUDIT_JS = r"""
() => {
  const MIN_TARGET = __MIN_TARGET__, MIN_FIELD = __MIN_FIELD__;
  const root = document.documentElement;
  const W = root.clientWidth;
  const coarse = matchMedia('(pointer: coarse)').matches;

  const label = (el) => {
    const t = (el.getAttribute('aria-label') || el.getAttribute('title') ||
               el.getAttribute('placeholder') || el.textContent || el.value || '')
      .replace(/\s+/g, ' ').trim();
    return t.length > 40 ? t.slice(0, 39) + '~' : t;
  };
  const step = (el) => {
    if (el.id) return el.tagName.toLowerCase() + '#' + CSS.escape(el.id);
    let s = el.tagName.toLowerCase();
    const cls = [...el.classList].slice(0, 2);
    if (cls.length) s += '.' + cls.map((c) => CSS.escape(c)).join('.');
    const p = el.parentElement;
    if (p) {
      const same = [...p.children].filter((c) => c.tagName === el.tagName);
      if (same.length > 1) s += ':nth-of-type(' + (same.indexOf(el) + 1) + ')';
    }
    return s;
  };
  const path = (el) => {
    const parts = [];
    for (let e = el; e && e !== root && parts.length < 3; e = e.parentElement) {
      parts.unshift(step(e));
      if (e.id) break;
    }
    return parts.join(' > ');
  };
  const shown = (el) => {
    if (el.closest('[hidden],[inert],[aria-hidden="true"]')) return false;
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden' ||
        cs.visibility === 'collapse' || cs.pointerEvents === 'none') return false;
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  };
  const describe = (el, extra) => Object.assign(
    { selector: path(el), text: label(el) }, extra);

  const out = {
    clientWidth: W, innerWidth: window.innerWidth, coarse, scrollWidth: root.scrollWidth,
    viewportMeta: !!document.querySelector('meta[name="viewport"]'),
    overflow: [], small: [], smallChecked: 0, inlineExempt: 0, tinySkipped: 0,
    fields: [], fieldsChecked: 0,
  };

  // overflow: name the outermost elements that reach past the right edge (or
  // start left of zero), skipping anything clipped by its own scroller or fixed.
  if (root.scrollWidth > W) {
    const contained = (el) => {
      for (let e = el.parentElement; e && e !== document.body && e !== root;
           e = e.parentElement) {
        const cs = getComputedStyle(e);
        if (cs.position === 'fixed') return true;
        if (['auto', 'scroll', 'hidden', 'clip'].includes(cs.overflowX)) return true;
      }
      return getComputedStyle(el).position === 'fixed';
    };
    const pokes = (el) => {
      const r = el.getBoundingClientRect();
      return r.width > 0 && (r.right > W + 1 || r.left < -1);
    };
    const all = [...document.body.querySelectorAll('*')]
      .filter((el) => pokes(el) && !contained(el));
    const outer = all.filter((el) => !el.parentElement || !pokes(el.parentElement) ||
                                     el.parentElement === document.body);
    outer.sort((a, b) => b.getBoundingClientRect().right - a.getBoundingClientRect().right);
    out.overflowCount = outer.length;
    out.overflow = outer.slice(0, 5).map((el) => {
      const r = el.getBoundingClientRect();
      return describe(el, { left: Math.round(r.left), right: Math.round(r.right),
                            width: Math.round(r.width) });
    });
  }

  if (coarse) {
    const CONTROLS = 'a[href],button,input:not([type="hidden"]),select,textarea,' +
      'summary,[role="button"],[role="link"],[role="tab"],[role="checkbox"],' +
      '[role="radio"],[role="switch"],[role="menuitem"],[role="option"],[tabindex]:not([tabindex="-1"])';
    const inSentence = (el) => {
      if (el.tagName !== 'A' || getComputedStyle(el).display !== 'inline') return false;
      const p = el.parentElement;
      return !!p && [...p.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
    };
    for (const el of document.querySelectorAll(CONTROLS)) {
      if (!shown(el)) continue;
      let box = el.getBoundingClientRect();
      if (box.width <= 1 || box.height <= 1) { out.tinySkipped++; continue; }
      if (inSentence(el)) { out.inlineExempt++; continue; }
      // a checkbox or radio inside its label is tapped through the label
      const lab = (el.type === 'checkbox' || el.type === 'radio') && el.closest('label');
      if (lab) box = lab.getBoundingClientRect();
      out.smallChecked++;
      if (box.width + 0.5 < MIN_TARGET || box.height + 0.5 < MIN_TARGET) {
        out.small.push(describe(el, { width: Math.round(box.width * 10) / 10,
                                      height: Math.round(box.height * 10) / 10 }));
      }
    }

    const TEXTISH = ['', 'text', 'search', 'email', 'url', 'tel', 'password',
                     'number', 'date', 'datetime-local', 'month', 'time', 'week'];
    const FIELDS = 'input,textarea,select,[contenteditable]:not([contenteditable="false"])';
    for (const el of document.querySelectorAll(FIELDS)) {
      if (el.tagName === 'INPUT' &&
          !TEXTISH.includes((el.getAttribute('type') || '').toLowerCase())) continue;
      if (!shown(el)) continue;
      out.fieldsChecked++;
      const px = parseFloat(getComputedStyle(el).fontSize);
      if (px + 0.01 < MIN_FIELD) out.fields.push(describe(el, { fontSize: px }));
    }
  }
  return out;
}
"""

MIN_TARGET = 44
MIN_FIELD = 16


def find_browser(flag: str | None, bundled: str | None) -> tuple[str | None, str]:
    """(path, how it was chosen). path None when nothing usable exists."""
    if flag:
        return (flag if os.access(flag, os.X_OK) else None), "--browser"
    env = os.environ.get("TRI_HARNESS_BROWSER")
    if env:
        return (env if os.access(env, os.X_OK) else None), "$TRI_HARNESS_BROWSER"
    if bundled and os.access(bundled, os.X_OK):
        return bundled, "Playwright's own Chromium"
    for c in CANDIDATES:
        if os.access(c, os.X_OK):
            return c, "first found on disk"
    for name in ("google-chrome", "chromium", "chromium-browser"):
        hit = shutil.which(name)
        if hit:
            return hit, "first found on PATH"
    return None, "nothing found"


def profile(width: int) -> dict:
    if width < MOBILE_BELOW:
        return {"viewport": {"width": width, "height": 812}, "is_mobile": True,
                "has_touch": True, "device_scale_factor": 2}
    return {"viewport": {"width": width, "height": 768}, "is_mobile": False,
            "has_touch": False, "device_scale_factor": 1}


MEASURED = ("clientWidth", "innerWidth", "scrollWidth", "coarse", "viewportMeta",
            "smallChecked", "inlineExempt", "tinySkipped", "fieldsChecked")


def findings(m: dict, top: bool = True) -> list[dict]:
    """One frame's measurement -> its findings. An iframe takes its width from its
    frame, so only the top document can lack a viewport meta that matters."""
    out = []
    if top and m["coarse"] and not m["viewportMeta"]:
        out.append({"kind": "viewport-meta",
                    "says": f"no <meta name=\"viewport\">: laid out at {m['clientWidth']}px "
                            "and shrunk -- every number at this width is a desktop number"})
    if m["scrollWidth"] > m["clientWidth"]:
        out.append({"kind": "overflow",
                    "says": f"scrollWidth {m['scrollWidth']} > clientWidth {m['clientWidth']}: "
                            f"the page scrolls sideways ({m.get('overflowCount', 0)} "
                            "element(s) reach past the edge)",
                    "items": m["overflow"]})
    if m["small"]:
        out.append({"kind": "small-target",
                    "says": f"{len(m['small'])} of {m['smallChecked']} controls under "
                            f"{MIN_TARGET}x{MIN_TARGET} (inline links exempt: "
                            f"{m['inlineExempt']}, 1px skipped: {m['tinySkipped']})",
                    "items": m["small"]})
    if m["fields"]:
        out.append({"kind": "field-zoom",
                    "says": f"{len(m['fields'])} of {m['fieldsChecked']} text fields under "
                            f"{MIN_FIELD}px -- iOS zooms in on focus",
                    "items": m["fields"]})
    return out


def item_line(it: dict) -> str:
    where = it["selector"] + (f' "{it["text"]}"' if it.get("text") else "")
    if "fontSize" in it:
        return f"{where}  {it['fontSize']:g}px"
    if "right" in it:
        return f"{where}  {it['width']}px wide, x {it['left']}..{it['right']}"
    return f"{where}  {it['width']:g}x{it['height']:g}"


def could_not(msg: str, hint: str, as_json: bool) -> int:
    if as_json:
        print(json.dumps({"result": "could-not-run", "why": msg, "hint": hint}, indent=2))
    else:
        print(f"tri harness: COULD NOT RUN -- {msg}")
        print(f"  {hint}")
        print("  Nothing was measured. This is not a clean result.")
    return 2


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(prog="tri harness", description=__doc__.split("\n")[1])
    ap.add_argument("url")
    ap.add_argument("--width", default="375,1024",
                    help="comma-separated CSS widths; under 768 is a phone (default 375,1024)")
    ap.add_argument("--browser", help="Chromium-family executable to drive")
    ap.add_argument("--wait-for", help="CSS selector that must be visible before measuring")
    ap.add_argument("--settle", type=int, default=1500,
                    help="ms to wait after load for a client-rendered page (default 1500)")
    ap.add_argument("--timeout", type=float, default=30.0, help="seconds per page load")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args(argv)

    try:
        widths = [int(w) for w in a.width.split(",") if w.strip()]
    except ValueError:
        print(f"tri harness: --width wants numbers, got {a.width!r}", file=sys.stderr)
        return 2
    if not widths or any(w < 200 or w > 4000 for w in widths):
        print("tri harness: each --width must be between 200 and 4000", file=sys.stderr)
        return 2

    try:
        from playwright.sync_api import Error as PwError
        from playwright.sync_api import sync_playwright
        from importlib.metadata import version
        pw_version = version("playwright")
    except Exception:
        return could_not("the Python package `playwright` is not importable",
                         "pip install playwright  (the package only: the browser is "
                         "found on disk; tri harness downloads nothing)", a.json)

    js = (AUDIT_JS.replace("__MIN_TARGET__", str(MIN_TARGET))
                  .replace("__MIN_FIELD__", str(MIN_FIELD)))
    report = {"url": a.url, "playwright": pw_version, "widths": [],
              "not_established": NOT_ESTABLISHED}
    with sync_playwright() as p:
        try:
            bundled = p.chromium.executable_path
        except Exception:
            bundled = None
        exe, how = find_browser(a.browser, bundled)
        if not exe:
            return could_not(f"no Chromium-family browser ({how})",
                             "pass --browser PATH or set TRI_HARNESS_BROWSER", a.json)
        report["browser"] = {"path": exe, "chosen_by": how}
        # whatever the browser writes to TMPDIR lands here and goes with it
        private = tempfile.mkdtemp(prefix="tri-harness-")
        try:
            try:
                browser = p.chromium.launch(executable_path=exe, headless=True,
                                            env=dict(os.environ, TMPDIR=private))
            except PwError as e:
                return could_not(f"{exe} did not start: {str(e).splitlines()[0]}",
                                 "pass another --browser PATH", a.json)
            return_code = measure(browser, a, js, widths, report, PwError)
        finally:
            shutil.rmtree(private, ignore_errors=True)
        if return_code is not None:
            return return_code

    return render(report, a, how, pw_version)


def measure(browser, a, js, widths, report, PwError) -> int | None:
    """Fill report["widths"]; an int is an early could-not-run exit."""
    try:
        for w in widths:
            ctx = browser.new_context(**profile(w))
            try:
                page = ctx.new_page()
                try:
                    resp = page.goto(a.url, wait_until="load",
                                     timeout=int(a.timeout * 1000))
                except PwError as e:
                    return could_not(f"{a.url} did not load at {w}px: "
                                     f"{str(e).splitlines()[0]}",
                                     "check the URL; --timeout raises the limit", a.json)
                if resp is not None and resp.status >= 400:
                    return could_not(f"{a.url} answered HTTP {resp.status}",
                                     "an error page is not the page; fix the URL", a.json)
                if a.wait_for:
                    try:
                        page.wait_for_selector(a.wait_for, state="visible",
                                               timeout=int(a.timeout * 1000))
                    except PwError:
                        return could_not(f"{a.wait_for!r} never became visible at {w}px",
                                         "check the selector against the page", a.json)
                if a.settle > 0:
                    page.wait_for_timeout(a.settle)
                frames, hidden, unmeasured = [], 0, []
                for i, fr in enumerate(page.frames):
                    if i:
                        # a frame nobody can see (0x0 analytics, display:none) is
                        # skipped and COUNTED; a visible one is measured like the top
                        try:
                            box = fr.frame_element().bounding_box()
                        except PwError:
                            box = None
                        if not box or box["width"] < 2 or box["height"] < 2:
                            hidden += 1
                            continue
                    try:
                        m = fr.evaluate(js)
                    except PwError as e:
                        if not i:
                            # exit 1 means "findings"; a broken measurement is not one
                            return could_not(f"the measurement itself failed at {w}px: "
                                             f"{str(e).splitlines()[0]}",
                                             "a defect in tri harness, not in the page",
                                             a.json)
                        unmeasured.append({"frame": i, "url": fr.url,
                                           "why": str(e).splitlines()[0]})
                        continue
                    frames.append({"frame": i, "url": fr.url,
                                   "measured": {k: m[k] for k in MEASURED},
                                   "findings": findings(m, top=not i)})
            finally:
                ctx.close()
            prof = profile(w)
            report["widths"].append({
                "width": w, "height": prof["viewport"]["height"],
                "kind": "phone" if prof["is_mobile"] else "desktop",
                "frames": frames, "hidden_frames": hidden, "unmeasured_frames": unmeasured,
            })
    finally:
        browser.close()
    return None


def render(report: dict, a, how: str, pw_version: str) -> int:
    total = sum(len(f["findings"]) for r in report["widths"] for f in r["frames"])
    gaps = sum(len(r["unmeasured_frames"]) for r in report["widths"])
    report["result"] = "findings" if total else ("incomplete" if gaps else "clean")
    code = 1 if total else (2 if gaps else 0)
    if a.json:
        print(json.dumps(report, indent=2, ensure_ascii=False))
        return code

    print(f"tri harness -- {a.url}")
    print(f"browser: {report['browser']['path']} ({how}), Playwright {pw_version}, "
          "headless, temporary profile")
    for r in report["widths"]:
        coarse = r["frames"][0]["measured"]["coarse"]
        ptr = "touch, coarse pointer" if coarse else "mouse, fine pointer"
        print()
        print(f"{r['width']}x{r['height']} {r['kind']} ({ptr}): {len(r['frames'])} frame(s) "
              f"measured, {r['hidden_frames']} hidden frame(s) skipped")
        for f in r["frames"]:
            m = f["measured"]
            print(f"  frame {f['frame']}  {f['url']}")
            print(f"    clientWidth {m['clientWidth']}, scrollWidth {m['scrollWidth']}, "
                  f"innerWidth {m['innerWidth']}")
            kinds = {x["kind"]: x for x in f["findings"]}
            rows = [("viewport-meta", "present" if m["viewportMeta"] else "absent",
                     m["coarse"] and not f["frame"]),
                    ("overflow", f"none (scrollWidth {m['scrollWidth']} <= clientWidth "
                                 f"{m['clientWidth']})", True),
                    ("small-target", f"{m['smallChecked']} controls >= {MIN_TARGET}x{MIN_TARGET} "
                                     f"(inline links exempt: {m['inlineExempt']}, 1px skipped: "
                                     f"{m['tinySkipped']})", m["coarse"]),
                    ("field-zoom", f"{m['fieldsChecked']} text fields >= {MIN_FIELD}px",
                     m["coarse"])]
            for kind, ok_text, checked in rows:
                if kind in kinds:
                    x = kinds[kind]
                    print(f"    FINDING  {kind}: {x['says']}")
                    for it in x.get("items", []):
                        print(f"               {item_line(it)}")
                elif checked:
                    print(f"    ok       {kind}: {ok_text}")
                elif kind == "viewport-meta" and f["frame"]:
                    print(f"    --       {kind}: an iframe takes its frame's width")
                else:
                    print(f"    --       {kind}: not checked on a fine pointer")
        for u in r["unmeasured_frames"]:
            print(f"  NOT MEASURED  frame {u['frame']}  {u['url']}: {u['why']}")
    print()
    per = "; ".join(f"{sum(len(f['findings']) for f in r['frames'])} at {r['width']}"
                    for r in report["widths"])
    word = {"findings": "FINDINGS", "incomplete": "INCOMPLETE -- a visible frame was not "
            "measured, so this is not a clean result", "clean": "clean"}[report["result"]]
    print(f"RESULT: {word} -- {per}")
    print(f"NOT ESTABLISHED: {NOT_ESTABLISHED}")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
