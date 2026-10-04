#!/usr/bin/env python3
"""`tri harness` measures what a phone would see, and says when it could not.

Each fixture below is a defect the Queen board loop shipped or nearly shipped
(cron 8782e5f8, 2026-10-04) or a trap the harness itself fell into on its first
run:

  broken    a 550px bar at 375 (S3), a 42px close mark and a 12px textarea (S11),
            next to a sentence link (exempt) and a contained wide scroller (not
            overflow, never named). All three at 375, none at 1024.
  clean     the same page done right: 0 findings at either width.
  nometa    no viewport meta: laid out at 980 on a phone.
  framed    a clean top page whose small button sits in an iframe, plus a 0x0
            frame -- app.t27.ai/game/browser draws its body in an iframe and the
            first run measured only the tab bar above it.
  delayed   a control that appears after load: the settle wait is what sees it.
  hostile   a page that breaks getComputedStyle: a broken measurement is exit 2,
            never 1 ("findings") and never 0 ("clean").
  gated     a page that draws a 30px control only when signed in -- by a cookie,
            localStorage or sessionStorage, chosen by `?want=`. app.t27.ai keeps
            its sign-in in sessionStorage, which Playwright's storage state does
            not carry, so the sessionStorage case is the one that matters.
  signin    a page that signs itself in on load, for --save-state.

The fixtures are served over HTTP from 127.0.0.1 so a 404 is a real 404. Every run
gets an empty TMPDIR that must be empty again afterwards: the browser profile is
temporary and must not outlive the run. Section 8 drives the browser through a
wrapper that writes into its TMPDIR the way Google Chrome on GitHub's runner did
on this test's first run there (`com.google.Chrome.chrome_chrome_url_fetcher_.*`)
-- the Mac's BrowserOS writes nothing, so without the wrapper the leak could only
be seen on CI.

Needs the `playwright` Python package and a Chromium-family browser on disk (CI:
the runner's /usr/bin/google-chrome). Without them this test FAILS rather than
skips -- the harness's own exit 2 is checked separately below.

Set TRI_HARNESS to run this against another copy of the tool (mutation runs).
"""
from __future__ import annotations

import http.server
import json
import os
import subprocess
import sys
import tempfile
import threading
from functools import partial
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
TOOL = os.environ.get("TRI_HARNESS") or str(REPO / "scripts/tri_loop/harness.py")

META = '<meta name="viewport" content="width=device-width,initial-scale=1">'
HEAD = "<!doctype html><html><head>{meta}<style>body{{margin:0;font:16px sans-serif}}" \
       "button{{padding:0}}</style>{extra}</head><body>{body}</body></html>"

# a stored sign-in that must never appear in anything the tool prints
SECRET = "good-s3cret-8782e5f8"

SENTENCE = '<p>Read <a href="#rules">the rules</a> first.</p>'
SCROLLER = ('<div class="scroller" style="overflow-x:auto;width:200px">'
            '<div class="wide" style="width:2000px">wide</div></div>')

FIXTURES = {
    "broken.html": HEAD.format(meta=META, extra="", body=(
        SENTENCE
        + '<div class="bar" style="width:550px;height:20px">live bar</div>'
        + '<button class="close" style="width:42px;height:42px">x</button>'
        + '<textarea class="lane" style="font-size:12px;height:60px;width:200px"></textarea>'
        + SCROLLER)),
    "clean.html": HEAD.format(meta=META, extra="", body=(
        SENTENCE
        + '<a class="skip" href="#main" style="position:absolute;width:1px;height:1px;'
          'overflow:hidden;clip:rect(0 0 0 0)">skip</a>'
        + '<div class="bar" style="max-width:100%;height:20px">live bar</div>'
        + '<button class="close" style="width:48px;height:48px">x</button>'
        + '<textarea class="lane" style="font-size:16px;height:60px;width:200px"></textarea>'
        + SCROLLER)),
    "nometa.html": HEAD.format(meta="", extra="", body=(
        '<button class="close" style="width:48px;height:48px">x</button>')),
    "inner.html": HEAD.format(meta=META, extra="", body=(
        '<button class="tiny" style="width:30px;height:30px">+</button>')),
    "framed.html": HEAD.format(meta=META, extra="", body=(
        '<button class="close" style="width:48px;height:48px">x</button>'
        '<iframe class="hive" src="inner.html" style="border:0;width:100%;height:200px"></iframe>'
        '<iframe class="pixel" src="clean.html" style="border:0;width:0;height:0"></iframe>')),
    "delayed.html": HEAD.format(meta=META, extra="", body=(
        "<script>setTimeout(function(){var b=document.createElement('button');"
        "b.className='late';b.textContent='+';b.style.cssText='width:30px;height:30px';"
        "document.body.appendChild(b)},700)</script>")),
    "hostile.html": HEAD.format(meta=META, extra=(
        "<script>window.getComputedStyle=function(){throw new Error('no styles')}</script>"),
        body='<button style="width:48px;height:48px">x</button>'),
    "gated.html": HEAD.format(meta=META, extra="", body=(
        "<script>var want=new URLSearchParams(location.search).get('want')||'any';"
        "var ok=function(v){return !!v&&v.indexOf('good')===0};"
        "var c=(document.cookie.match(/(?:^|; )sid=([^;]*)/)||[])[1];"
        "var hit={cookie:ok(c),local:ok(localStorage.getItem('tok')),"
        "session:ok(sessionStorage.getItem('tok'))};"
        "var yes=want==='any'?(hit.cookie||hit.local||hit.session):hit[want];"
        "var b=document.createElement('button');"
        "if(yes){b.className='signed';b.textContent='me';b.style.cssText='width:30px;height:30px'}"
        "else{b.className='signin';b.textContent='Sign in';"
        "b.style.cssText='width:48px;height:48px'}"
        "document.body.appendChild(b)</script>")),
    "signin.html": HEAD.format(meta=META, extra="", body=(
        f"<script>var s='{SECRET}';document.cookie='sid='+s+'; path=/';"
        "localStorage.setItem('tok',s);sessionStorage.setItem('tok',s)</script>"
        '<button class="signed" style="width:48px;height:48px">me</button>')),
}

passed = failed = 0


def check(ok: bool, what: str, detail: str = "") -> None:
    global passed, failed
    if ok:
        passed += 1
        print(f"  ok    {what}")
    else:
        failed += 1
        print(f"  FAIL  {what}")
        if detail:
            for line in detail.rstrip().splitlines()[-25:]:
                print(f"          {line}")


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args) -> None:  # the run's own output is the evidence
        pass


def leftovers(d: Path) -> list[str]:
    return sorted(str(p.relative_to(d)) for p in d.rglob("*"))


def harness(*args: str, pre: str = "", stdin: str = "",
            env_add: dict | None = None) -> tuple[int, str, list[str]]:
    """Run the tool with an empty TMPDIR; return (exit, stdout+stderr, leftovers)."""
    with tempfile.TemporaryDirectory(prefix="tri-harness-test-") as td:
        tmp = Path(td) / "tmp"
        tmp.mkdir()
        env = dict(os.environ, TMPDIR=str(tmp), **(env_add or {}))
        if pre:
            cmd = [sys.executable, "-c",
                   pre + "\nimport runpy,sys\nsys.argv=['harness']+sys.argv[1:]\n"
                   f"runpy.run_path({TOOL!r}, run_name='__main__')", *args]
        else:
            cmd = [sys.executable, TOOL, *args]
        r = subprocess.run(cmd, capture_output=True, text=True, env=env, timeout=180,
                           input=stdin)
        return r.returncode, r.stdout + r.stderr, leftovers(tmp)


def main() -> int:
    site = Path(tempfile.mkdtemp(prefix="tri-harness-site-"))
    for name, html in FIXTURES.items():
        (site / name).write_text(html)
    handler = partial(Quiet, directory=str(site))
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{srv.server_address[1]}/"

    try:
        print("0. the TMPDIR check can see a leftover (one-byte control)")
        with tempfile.TemporaryDirectory() as td:
            (Path(td) / "x").write_bytes(b"x")
            check(leftovers(Path(td)) == ["x"], "a one-byte file is reported")

        print("1. broken: all three defects at 375, none at 1024")
        code, out, left = harness(base + "broken.html", "--json")
        check(code == 1, "exit 1 (findings)", out)
        try:
            rep = json.loads(out)
        except ValueError:
            rep = {"widths": []}
            check(False, "--json prints JSON", out)
        by = {r["width"]: r for r in rep.get("widths", [])}
        check(set(by) == {375, 1024}, "measured at 375 and 1024 by default", out)
        f375 = {f["kind"]: f for fr in by.get(375, {}).get("frames", []) for f in fr["findings"]}
        f1024 = [f for fr in by.get(1024, {}).get("frames", []) for f in fr["findings"]]
        check(set(f375) == {"overflow", "small-target", "field-zoom"},
              "375: overflow, small-target, field-zoom", json.dumps(f375, indent=1))
        check(f1024 == [], "1024: nothing (550 fits; targets and fields need a coarse pointer)",
              json.dumps(f1024, indent=1))
        ov = [i["selector"] for i in f375.get("overflow", {}).get("items", [])]
        check(any("div.bar" in s for s in ov), "overflow names the 550px bar", str(ov))
        check(not any("wide" in s or "scroller" in s for s in ov),
              "overflow never names content inside its own scroller", str(ov))
        m375 = by.get(375, {}).get("frames", [{}])[0].get("measured", {})
        check(m375.get("clientWidth") == 375 and m375.get("scrollWidth") == 550,
              "clientWidth 375, scrollWidth 550 (innerWidth would read 550 and hide it)",
              json.dumps(m375))
        small = [(i["selector"], i["width"]) for i in f375.get("small-target", {}).get("items", [])]
        check(small == [("body > button.close", 42)], "the 42px close mark, and only it",
              str(small))
        check(m375.get("inlineExempt") == 1, "the sentence link is exempt and counted",
              json.dumps(m375))
        zoom = [(i["selector"], i["fontSize"]) for i in f375.get("field-zoom", {}).get("items", [])]
        check(zoom == [("body > textarea.lane", 12)], "the 12px textarea", str(zoom))
        check(left == [], "TMPDIR empty afterwards", str(left))
        real = rep.get("browser", {}).get("path")

        print("2. broken, as text: the same verdict a person reads")
        code, out, left = harness(base + "broken.html")
        check(code == 1 and "RESULT: FINDINGS -- 3 at 375; 0 at 1024" in out,
              "RESULT line counts 3 at 375 and 0 at 1024", out)
        check("NOT ESTABLISHED: first load only" in out, "prints what it did not establish", out)
        check(left == [], "TMPDIR empty afterwards", str(left))

        print("3. clean: nothing at either width")
        code, out, left = harness(base + "clean.html")
        check(code == 0 and "RESULT: clean -- 0 at 375; 0 at 1024" in out, "exit 0, clean", out)
        check("1px skipped: 1" in out, "the screen-reader-only link is skipped and counted", out)
        check(left == [], "TMPDIR empty afterwards", str(left))

        print("4. nometa: laid out at 980 on a phone")
        code, out, _ = harness(base + "nometa.html", "--width", "375")
        check(code == 1 and "FINDING  viewport-meta" in out and "laid out at 980px" in out,
              "viewport-meta finding names 980", out)

        print("5. framed: the small button in a visible iframe is found; a 0x0 frame skipped")
        code, out, _ = harness(base + "framed.html", "--width", "375", "--json")
        try:
            rep = json.loads(out)
            fr = rep["widths"][0]["frames"]
            hidden = rep["widths"][0]["hidden_frames"]
        except (ValueError, KeyError, IndexError):
            fr, hidden = [], -1
        check(code == 1, "exit 1", out)
        check(len(fr) == 2 and fr[0]["findings"] == [], "two frames measured; the top is clean",
              out)
        inner = [i["selector"] for f in (fr[1]["findings"] if len(fr) > 1 else [])
                 for i in f.get("items", [])]
        check(len(fr) > 1 and fr[1]["url"].endswith("/inner.html")
              and inner == ["body > button.tiny"], "frame 1 is inner.html and names button.tiny",
              out)
        check(hidden == 1, "the 0x0 frame is skipped and counted", out)

        print("6. delayed: the settle wait is what sees a control drawn after load")
        code, out, _ = harness(base + "delayed.html", "--width", "375")
        check(code == 1 and "button.late" in out, "default settle: the late button is found", out)
        code, out, _ = harness(base + "delayed.html", "--width", "375", "--settle", "0")
        check(code == 0, "--settle 0 misses it (why the default is not 0)", out)
        code, out, _ = harness(base + "delayed.html", "--width", "375", "--settle", "0",
                               "--wait-for", "button.late")
        check(code == 1 and "button.late" in out, "--wait-for finds it without the settle", out)

        print("7. could not measure: exit 2, never 0 or 1")
        code, out, _ = harness(base + "missing.html", "--width", "375")
        check(code == 2 and "answered HTTP 404" in out, "a 404 is not measured", out)
        code, out, left = harness(base + "hostile.html", "--width", "375")
        check(code == 2 and "the measurement itself failed" in out,
              "a broken measurement is exit 2", out)
        check(left == [], "TMPDIR empty after a failed measurement", str(left))
        code, out, _ = harness(base + "clean.html", "--width", "375", "--settle", "0",
                               "--wait-for", "#never", "--timeout", "2")
        check(code == 2 and "never became visible" in out, "--wait-for that never appears", out)
        code, out, _ = harness(base + "clean.html", "--browser", "/nonexistent/chrome")
        check(code == 2 and "no Chromium-family browser" in out and "--browser" in out,
              "an unusable --browser", out)
        code, out, _ = harness(base + "clean.html", pre="import sys; sys.modules['playwright']=None")
        check(code == 2 and "pip install playwright" in out and "not a clean result" in out,
              "no playwright package", out)
        code, out, _ = harness(base + "clean.html", "--width", "abc")
        check(code == 2, "a --width that is not a number", out)

        print("8. a browser that writes into TMPDIR (Google Chrome on GitHub's runner)")
        with tempfile.TemporaryDirectory(prefix="tri-harness-leaky-") as td:
            log = Path(td) / "saw"
            leaky = Path(td) / "leaky-browser"
            leaky.write_text(
                "#!/bin/sh\n"
                'd="$TMPDIR/com.google.Chrome.chrome_chrome_url_fetcher_.test"\n'
                'mkdir -p "$d" && printf x > "$d/fbdd96f4"\n'
                f'echo "$TMPDIR" >> {str(log)!r}\n'
                f'exec {str(real)!r} "$@"\n')
            leaky.chmod(0o755)
            code, out, left = harness(base + "clean.html", "--width", "375",
                                      "--browser", str(leaky))
            saw = log.read_text().split() if log.exists() else []
            check(code == 0 and bool(saw), "the wrapper ran and the page measured clean", out)
            check(left == [], "TMPDIR empty although the browser wrote into its own",
                  str(left))
            check(bool(saw) and all(Path(d).name.startswith("tri-harness-")
                                    and not Path(d).exists() for d in saw),
                  "the browser's TMPDIR was private and is gone", str(saw))

        port = srv.server_address[1]
        origin = f"http://127.0.0.1:{port}"
        # the same page from a second origin (localhost is not 127.0.0.1 to a browser)
        frame = '<iframe src="http://{}:%d/gated.html?want=session" ' \
                'style="border:0;width:100%%;height:120px"></iframe>' % port
        (site / "two.html").write_text(HEAD.format(meta=META, extra="", body=(
            frame.format("localhost") + frame.format("127.0.0.1"))))

        def cookie(value: str, domain: str = "127.0.0.1", expires: float = -1) -> dict:
            return {"name": "sid", "value": value, "domain": domain, "path": "/",
                    "expires": expires, "httpOnly": False, "secure": False,
                    "sameSite": "Lax"}

        def store(d: Path, name: str, body, mode: int = 0o600) -> str:
            f = d / name
            f.write_text(body if isinstance(body, str) else json.dumps(body))
            f.chmod(mode)
            return str(f)

        print("9. --storage-state: measured signed in, and only when it can be shown")
        with tempfile.TemporaryDirectory(prefix="tri-harness-state-") as sd:
            d = Path(sd)
            jar = store(d, "cookie.json", {"cookies": [cookie(SECRET)], "origins": []})
            local = store(d, "local.json", {"cookies": [], "origins": [
                {"origin": origin, "localStorage": [{"name": "tok", "value": SECRET}]}]})
            sess = store(d, "session.json", {"cookies": [], "origins": [
                {"origin": origin, "localStorage": [],
                 "sessionStorage": [{"name": "tok", "value": SECRET}]}]})
            stale = store(d, "stale.json", {"cookies": [cookie("stale")], "origins": []})
            before = {f: (Path(f).read_bytes(), Path(f).stat().st_mtime_ns)
                      for f in (jar, local, sess, stale)}
            printed = []

            code, out, _ = harness(base + "gated.html", "--width", "375")
            check(code == 0 and "button.signin" not in out,
                  "no state: the signed-out page, clean", out)
            code, out, left = harness(base + "gated.html?want=cookie", "--width", "375",
                                      "--storage-state", jar, "--wait-for", "button.signed")
            printed.append(out)
            check(code == 1 and "button.signed" in out, "a cookie state: the signed-in control",
                  out)
            check("signed in: proven by 'button.signed' visible at every width" in out
                  and "1 cookie(s)" in out, "says what proved the sign-in and counts it", out)
            check(left == [], "TMPDIR empty afterwards", str(left))
            code, out, _ = harness(base + "gated.html?want=local", "--width", "375",
                                   "--storage-state", local, "--wait-for", "button.signed")
            printed.append(out)
            check(code == 1 and "button.signed" in out, "a localStorage state", out)
            code, out, _ = harness(base + "gated.html?want=session", "--storage-state", sess,
                                   "--wait-for", "button.signed")
            printed.append(out)
            check(code == 1 and "RESULT: FINDINGS -- 1 at 375; 0 at 1024" in out,
                  "a sessionStorage state, at both widths (Playwright's state alone has none)",
                  out)
            code, out, _ = harness(base + "two.html", "--width", "375",
                                   "--storage-state", sess, "--json")
            printed.append(out)
            try:
                fr = json.loads(out)["widths"][0]["frames"]
            except (ValueError, KeyError, IndexError):
                fr = []
            kinds = [(f["url"].split("/")[2].split(":")[0],
                      [i["selector"] for x in f["findings"] for i in x.get("items", [])])
                     for f in fr[1:]]
            check(kinds == [("localhost", []), ("127.0.0.1", ["body > button.signed"])],
                  "sessionStorage goes only to its own origin: the localhost frame is "
                  "signed out, the 127.0.0.1 frame signed in", out)
            code, out, _ = harness(base + "gated.html?want=session", "--width", "375",
                                   "--storage-state", sess, "--settle", "0")
            printed.append(out)
            check(code == 1 and "signed in: NOT established" in out
                  and "still signed in" in out.split("NOT ESTABLISHED:")[-1],
                  "without --wait-for the sign-in is reported as not established", out)

            print("   a session the page no longer accepts")
            code, out, _ = harness(base + "gated.html?want=cookie", "--width", "375",
                                   "--storage-state", stale)
            check(code == 0 and "signed in: NOT established" in out,
                  "a stale cookie measures the signed-out page and says it is not proven",
                  out)
            code, out, _ = harness(base + "gated.html?want=cookie", "--width", "375",
                                   "--storage-state", stale, "--wait-for", "button.signed",
                                   "--timeout", "3")
            check(code == 2 and "never became visible" in out and "signed out here" in out,
                  "with --wait-for it is exit 2, not a clean signed-out run", out)

            print("   refused before a browser starts")
            refusals = [
                ("readable by others", store(d, "open.json", {"cookies": [cookie(SECRET)]},
                                             mode=0o644), "chmod 600"),
                ("another site's", store(d, "other.json",
                                         {"cookies": [cookie(SECRET, "example.org")]}),
                 "holds nothing live for 127.0.0.1"),
                ("expired", store(d, "old.json",
                                  {"cookies": [cookie(SECRET, expires=1000.0)]}),
                 "have expired"),
                ("not JSON", store(d, "junk.json", "not json " + SECRET),
                 "not a JSON storage state"),
                ("not a state", store(d, "shape.json", {"cookies": [{"value": SECRET}]}),
                 "is not a storage state"),
                ("missing", str(d / "absent.json"), "cannot read"),
            ]
            for what, f, says in refusals:
                code, out, left = harness(base + "gated.html", "--storage-state", f)
                printed.append(out)
                check(code == 2 and says in out and "Nothing was measured" in out
                      and left == [], f"{what} state: exit 2, {says!r}", out)
            code, out, _ = harness((Path(site) / "gated.html").as_uri(), "--storage-state", jar)
            check(code == 2 and "needs an http(s) URL" in out, "a file: URL", out)

            check(all(SECRET not in o for o in printed),
                  f"the stored value never appears in output ({len(printed)} runs)",
                  next((o for o in printed if SECRET in o), ""))
            after = {f: (Path(f).read_bytes(), Path(f).stat().st_mtime_ns) for f in before}
            check(after == before, "every state file is unchanged: bytes and mtime")

        print("10. --save-state: the person signs in, Enter writes the file 600")
        with tempfile.TemporaryDirectory(prefix="tri-harness-save-") as sd:
            d = Path(sd)
            saved = d / "app.json"
            show = {"TRI_HARNESS_HEADLESS": "1"}
            code, out, left = harness(base + "signin.html", "--save-state", str(saved),
                                      stdin="\n", env_add=show)
            check(code == 0 and saved.exists() and "values not printed" in out,
                  "saved after Enter", out)
            check(saved.exists() and (saved.stat().st_mode & 0o777) == 0o600,
                  "mode 600", oct(saved.stat().st_mode) if saved.exists() else "absent")
            check(SECRET not in out, "the saved value is not printed", out)
            check(left == [], "TMPDIR empty afterwards", str(left))
            try:
                got = json.loads(saved.read_text())
                ss = [o.get("sessionStorage") for o in got["origins"] if o["origin"] == origin]
            except (OSError, ValueError, KeyError):
                got, ss = {}, []
            check(ss == [[{"name": "tok", "value": SECRET}]],
                  "sessionStorage is saved -- Playwright's own state leaves it out",
                  json.dumps(ss))
            code, out, _ = harness(base + "gated.html?want=session", "--width", "375",
                                   "--storage-state", str(saved), "--wait-for", "button.signed")
            check(code == 1 and "button.signed" in out and SECRET not in out,
                  "round trip: the saved file measures the signed-in page", out)
            keep = saved.read_bytes()
            code, out, _ = harness(base + "signin.html", "--save-state", str(saved),
                                   stdin="\n", env_add=show)
            check(code == 2 and "never overwrites" in out and saved.read_bytes() == keep,
                  "an existing file is never overwritten", out)
            code, out, _ = harness(base + "signin.html", "--save-state", str(d / "eof.json"),
                                   stdin="", env_add=show)
            check(code == 2 and not (d / "eof.json").exists(),
                  "no Enter (stdin closed): nothing written", out)
            code, out, _ = harness(base + "clean.html", "--save-state", str(d / "none.json"),
                                   stdin="\n", env_add=show)
            check(code == 2 and "holds nothing for 127.0.0.1" in out
                  and not (d / "none.json").exists(), "a page with no sign-in: nothing written",
                  out)
    finally:
        srv.shutdown()
        for p in site.iterdir():
            p.unlink()
        site.rmdir()

    print()
    print(f"{passed} passed, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
