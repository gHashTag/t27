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


def harness(*args: str, pre: str = "") -> tuple[int, str, list[str]]:
    """Run the tool with an empty TMPDIR; return (exit, stdout+stderr, leftovers)."""
    with tempfile.TemporaryDirectory(prefix="tri-harness-test-") as td:
        tmp = Path(td) / "tmp"
        tmp.mkdir()
        env = dict(os.environ, TMPDIR=str(tmp))
        if pre:
            cmd = [sys.executable, "-c",
                   pre + "\nimport runpy,sys\nsys.argv=['harness']+sys.argv[1:]\n"
                   f"runpy.run_path({TOOL!r}, run_name='__main__')", *args]
        else:
            cmd = [sys.executable, TOOL, *args]
        r = subprocess.run(cmd, capture_output=True, text=True, env=env, timeout=180)
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
