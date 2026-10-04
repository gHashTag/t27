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

SIGNED IN
---------
Without a stored session every width is a fresh, signed-out visitor, and a page
that draws its real controls only after sign-in -- the Queen board on app.t27.ai
-- is measured as its sign-in screen. Two flags close that; neither ever sees a
password:

    tri harness --save-state ~/.config/tri/app.json https://app.t27.ai/game/browser
    tri harness --storage-state ~/.config/tri/app.json URL --wait-for SELECTOR

--save-state opens a VISIBLE browser with a temporary profile at URL and
measures nothing. You sign in there, in that window; when the signed-in page
shows, Enter in the terminal writes the session to FILE with mode 600. It never
overwrites a file, and it writes nothing if the session holds nothing for URL's
host. It saves Playwright's storage state (cookies, localStorage, IndexedDB) AND
sessionStorage, which Playwright's state leaves out: app.t27.ai keeps its sign-in
in sessionStorage (gHashTag/trinity apps/website/src/lib/appSessionIdentity.ts),
so a Playwright-only state measures the signed-out page and says nothing.

--storage-state loads FILE read-only into each width's fresh context. Its
sessionStorage is put in before the page's own scripts run, and only into the
origin it was saved from. It refuses (exit 2) a FILE anyone but its owner can
read, and a FILE that holds nothing live for URL's host: a state for another site,
or one whose cookies have all expired, would measure the signed-out page under
the signed-in name. Values are never printed -- the report counts them.

Loading a session is not being signed in: a token can be revoked or expire on
the server, and a site that rotates its token on use can sign the second width
out. `--wait-for` a selector only the signed-in page shows is the proof, checked
at every width; without it the report says the sign-in was not established. The
page runs as you: whatever it does on load for a signed-in visitor, it does in
your name.

NOT ESTABLISHED
---------------
First load only: no menu opened, no field focused, nothing typed or scrolled, so
a defect that appears after an interaction is not seen. Chromium's mobile
emulation, not iOS Safari. Content behind a sign-in (unless --storage-state), or
hidden when the page settles, is not measured. A clean run says these four
checks found nothing on this load; it does not say the page works on a phone.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import sys
import tempfile
import time
from urllib.parse import urlsplit

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


def not_established(a) -> str:
    if not a.storage_state:
        return NOT_ESTABLISHED
    said = NOT_ESTABLISHED.replace("content behind a sign-in or hidden",
                                   "content hidden")
    if not a.wait_for:
        said += ("; that the stored session was still signed in -- pass --wait-for "
                 "a selector only the signed-in page shows")
    return said


# Put a saved sessionStorage back before the page's own scripts read it, in the
# origin it came from and nowhere else. A key the page already holds is left alone.
SESSION_JS = r"""
(() => {
  const stored = __SESSION__;
  const own = stored[location.origin];
  if (!own) return;
  try {
    for (const [k, v] of own) if (sessionStorage.getItem(k) === null) sessionStorage.setItem(k, v);
  } catch (e) { /* an opaque origin has no sessionStorage */ }
})();
"""

SESSION_READ_JS = "() => [location.origin, Object.entries(sessionStorage)]"


class Refused(Exception):
    def __init__(self, msg: str, hint: str) -> None:
        super().__init__(msg)
        self.msg, self.hint = msg, hint


def host_of(url: str) -> tuple[str, str]:
    u = urlsplit(url)
    return u.scheme.lower(), (u.hostname or "").lower()


def domain_matches(host: str, domain: str) -> bool:
    d = domain.lstrip(".").lower()
    return bool(d) and (host == d or host.endswith("." + d))


def coverage(state: dict, host: str, now: float) -> dict:
    """What a state holds for one host: counts only, never values."""
    live = expired = 0
    for c in state.get("cookies", []):
        if domain_matches(host, c.get("domain", "")):
            exp = c.get("expires", -1)
            if exp is None or exp < 0 or exp > now:
                live += 1
            else:
                expired += 1
    ls = ss = idb = 0
    for o in state.get("origins", []):
        if (urlsplit(o.get("origin", "")).hostname or "").lower() == host:
            ls += len(o.get("localStorage") or [])
            ss += len(o.get("sessionStorage") or [])
            idb += len(o.get("indexedDB") or [])
    return {"cookies": live, "expired_cookies": expired, "localStorage": ls,
            "sessionStorage": ss, "indexedDB": idb}


def held(cov: dict) -> int:
    return cov["cookies"] + cov["localStorage"] + cov["sessionStorage"] + cov["indexedDB"]


def pairs_ok(xs) -> bool:
    return isinstance(xs, list) and all(
        isinstance(x, dict) and isinstance(x.get("name"), str)
        and isinstance(x.get("value"), str) for x in xs)


def load_state(path: str, url: str, now: float) -> tuple[dict, dict, dict]:
    """FILE -> (Playwright state, {origin: [[k, v]]} sessionStorage, coverage).
    Raises Refused. Reads FILE once and never writes it; no value reaches a message."""
    make = f"make it with: tri harness --save-state {path} URL"
    try:
        st = os.stat(path)
    except OSError as e:
        raise Refused(f"cannot read {path}: {e.strerror}", make) from None
    if os.name == "posix" and st.st_mode & 0o077:
        raise Refused(f"{path} can be read by others (mode {stat.S_IMODE(st.st_mode):o}), "
                      "and it holds a sign-in", f"chmod 600 {path}")
    scheme, host = host_of(url)
    if scheme not in ("http", "https") or not host:
        raise Refused(f"a stored session needs an http(s) URL, not {scheme or url}:",
                      "file: pages have no sign-in to load")
    try:
        with open(path, "rb") as f:
            raw = json.loads(f.read())
    except (OSError, ValueError):
        raise Refused(f"{path} is not a JSON storage state", make) from None
    cookies, origins = (raw.get("cookies", []), raw.get("origins", [])) \
        if isinstance(raw, dict) else (None, None)
    shape_ok = (isinstance(cookies, list) and isinstance(origins, list)
                and all(isinstance(c, dict) and isinstance(c.get("name"), str)
                        and isinstance(c.get("value"), str)
                        and isinstance(c.get("domain"), str) for c in cookies)
                and all(isinstance(o, dict) and isinstance(o.get("origin"), str)
                        and pairs_ok(o.get("localStorage", []))
                        and pairs_ok(o.get("sessionStorage", [])) for o in origins))
    if not shape_ok:
        raise Refused(f"{path} is not a storage state: wants {{cookies: [{{name, value, "
                      "domain}]}, origins: [{origin, localStorage, sessionStorage}]}", make)
    cov = coverage(raw, host, now)
    if not held(cov):
        why = (f"; its {cov['expired_cookies']} cookie(s) for {host} have expired"
               if cov["expired_cookies"] else "")
        raise Refused(f"{path} holds nothing live for {host}{why}",
                      f"sign in again: tri harness --save-state NEWFILE {url}")
    pw = {"cookies": cookies,
          "origins": [{"origin": o["origin"], "localStorage": o.get("localStorage", []),
                       **({"indexedDB": o["indexedDB"]} if "indexedDB" in o else {})}
                      for o in origins]}
    session = {o["origin"]: [[x["name"], x["value"]] for x in o["sessionStorage"]]
               for o in origins if o.get("sessionStorage")}
    return pw, session, cov

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
    who = ap.add_mutually_exclusive_group()
    who.add_argument("--storage-state", metavar="FILE",
                     help="measure signed in: load FILE (from --save-state) read-only into "
                          "every width; pair with --wait-for as the proof")
    who.add_argument("--save-state", metavar="FILE",
                     help="open URL in a visible browser, you sign in, Enter saves the "
                          "session to FILE (mode 600); measures nothing")
    a = ap.parse_args(argv)

    try:
        widths = [int(w) for w in a.width.split(",") if w.strip()]
    except ValueError:
        print(f"tri harness: --width wants numbers, got {a.width!r}", file=sys.stderr)
        return 2
    if not widths or any(w < 200 or w > 4000 for w in widths):
        print("tri harness: each --width must be between 200 and 4000", file=sys.stderr)
        return 2

    # refused before a browser starts: a bad state must not become a signed-out run
    pw_state, session, cov = None, {}, None
    if a.storage_state:
        try:
            pw_state, session, cov = load_state(a.storage_state, a.url, time.time())
        except Refused as r:
            return could_not(r.msg, r.hint, a.json)

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
              "not_established": not_established(a)}
    if cov is not None:
        report["session"] = {"file": a.storage_state, "host": host_of(a.url)[1],
                             **{k: v for k, v in cov.items() if k != "expired_cookies"},
                             "values_printed": False, "proven_by": a.wait_for}
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
            if a.save_state:
                return save_state(p, exe, private, a, PwError)
            try:
                browser = p.chromium.launch(executable_path=exe, headless=True,
                                            env=dict(os.environ, TMPDIR=private))
            except PwError as e:
                return could_not(f"{exe} did not start: {str(e).splitlines()[0]}",
                                 "pass another --browser PATH", a.json)
            return_code = measure(browser, a, js, widths, report, PwError,
                                  pw_state, session)
        finally:
            shutil.rmtree(private, ignore_errors=True)
        if return_code is not None:
            return return_code

    return render(report, a, how, pw_version)


def save_state(p, exe: str, private: str, a, PwError) -> int:
    """--save-state: a person signs in in a visible window; Enter writes FILE 600."""
    path = a.save_state
    if os.path.lexists(path):
        return could_not(f"{path} already exists; tri harness never overwrites a sign-in",
                         "remove it yourself, or pick another path", a.json)
    parent = os.path.dirname(os.path.abspath(path))
    if not os.path.isdir(parent):
        return could_not(f"{parent} is not a directory", "create it first", a.json)
    scheme, host = host_of(a.url)
    if scheme not in ("http", "https") or not host:
        return could_not(f"a sign-in needs an http(s) URL, not {scheme or a.url}:",
                         "pass the page you sign in at", a.json)
    # the test's only door: CI has no screen to show a window on
    headless = os.environ.get("TRI_HARNESS_HEADLESS") == "1"
    try:
        browser = p.chromium.launch(executable_path=exe, headless=headless,
                                    env=dict(os.environ, TMPDIR=private))
    except PwError as e:
        return could_not(f"{exe} did not start: {str(e).splitlines()[0]}",
                         "pass another --browser PATH", a.json)
    try:
        ctx = browser.new_context(no_viewport=not headless)
        page = ctx.new_page()
        try:
            page.goto(a.url, wait_until="load", timeout=int(a.timeout * 1000))
        except PwError as e:
            return could_not(f"{a.url} did not load: {str(e).splitlines()[0]}",
                             "check the URL; nothing was saved", a.json)
        say = lambda s: print(s, file=sys.stderr, flush=True)  # noqa: E731
        say(f"tri harness --save-state: a browser with a temporary profile is open at {a.url}")
        say("  Sign in there. Nothing typed in that window reaches this terminal.")
        say("  When the signed-in page shows, press Enter here. Ctrl-C saves nothing.")
        try:
            if not sys.stdin.readline():
                return could_not("stdin closed before Enter", "nothing was saved", a.json)
        except KeyboardInterrupt:
            return could_not("cancelled", "nothing was saved", a.json)
        try:
            try:
                state = ctx.storage_state(indexed_db=True)
            except TypeError:  # Playwright before 1.51 has no IndexedDB in its state
                state = ctx.storage_state()
            got: dict[str, dict] = {}
            for pg in ctx.pages:
                for fr in pg.frames:
                    try:
                        origin, items = fr.evaluate(SESSION_READ_JS)
                    except PwError:
                        continue
                    if origin and origin != "null" and items:
                        got.setdefault(origin, {}).update(dict(items))
        except PwError as e:
            return could_not(f"the browser closed before Enter: {str(e).splitlines()[0]}",
                             "nothing was saved; keep the window open until Enter", a.json)
    finally:
        browser.close()
    by_origin = {o["origin"]: o for o in state.get("origins", [])}
    for origin, items in got.items():
        o = by_origin.setdefault(origin, {"origin": origin, "localStorage": []})
        o["sessionStorage"] = [{"name": k, "value": v} for k, v in items.items()]
    state["origins"] = list(by_origin.values())
    cov = coverage(state, host, time.time())
    if not held(cov):
        return could_not(f"the session holds nothing for {host}: not signed in, or the "
                         "sign-in lives where this cannot read", "nothing was saved", a.json)
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "w") as f:
        json.dump(state, f)
    counts = {k: v for k, v in cov.items() if k != "expired_cookies"}
    if a.json:
        print(json.dumps({"result": "saved", "file": path, "host": host, **counts,
                          "values_printed": False}, indent=2))
    else:
        print(f"saved {path} (mode 600) for {host}: {counts['cookies']} cookie(s), "
              f"{counts['localStorage']} localStorage, {counts['sessionStorage']} "
              f"sessionStorage, {counts['indexedDB']} IndexedDB -- values not printed")
        print("  It is a sign-in: whoever reads it is you until it expires. Keep it out "
              "of git and off shared disks.")
        print(f"  Measure with: tri harness --storage-state {path} URL --wait-for SELECTOR")
    return 0


def measure(browser, a, js, widths, report, PwError, pw_state=None,
            session=None) -> int | None:
    """Fill report["widths"]; an int is an early could-not-run exit."""
    seed = (SESSION_JS.replace("__SESSION__", json.dumps(session))
            if session else None)
    try:
        for w in widths:
            ctx = browser.new_context(**profile(w),
                                      **({"storage_state": pw_state} if pw_state else {}))
            try:
                if seed:
                    ctx.add_init_script(script=seed)
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
                        hint = ("the stored session may be signed out here -- expired, "
                                "revoked, or rotated by an earlier width; or the selector "
                                "is wrong" if a.storage_state
                                else "check the selector against the page")
                        return could_not(f"{a.wait_for!r} never became visible at {w}px",
                                         hint, a.json)
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
    s = report.get("session")
    if s:
        print(f"session: {s['file']} -- for {s['host']}: {s['cookies']} cookie(s), "
              f"{s['localStorage']} localStorage, {s['sessionStorage']} sessionStorage, "
              f"{s['indexedDB']} IndexedDB; values not printed")
        print(f"signed in: proven by {s['proven_by']!r} visible at every width"
              if s["proven_by"] else
              "signed in: NOT established -- the session was loaded; nothing showed it "
              "was still signed in (pass --wait-for)")
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
    print(f"NOT ESTABLISHED: {report['not_established']}")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
