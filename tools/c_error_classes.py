#!/usr/bin/env python3
"""What is actually wrong with the generated C, ranked.

Three passes were spent shrinking one error class from 1729 to 522 while a
family TEN TIMES its size sat uncounted, because the first question of each
pass was "what is left of what I was doing" rather than "what is biggest now".
This answers the second one.

TWO THINGS IT INSISTS ON, both learned the hard way:

  * `-ferror-limit=0`. Clang stops at twenty errors per file by default and
    141 corpus files reach it, so every total measured without this is a
    FLOOR -- 3849 reported where the real count was 15188 (#3448).
  * the compiler's identity, printed beside the numbers. The same command
    gives 20 on Apple clang and 50 on gcc for the same input; a diagnostic
    count measures the code, the instrument AND the machine (#3450).

It is a READER, not a gate, and deliberately not wired into CI: it rebuilds the
whole corpus, which is minutes, and nothing here is a pass/fail claim. Run it
at the start of a pass to decide what the pass is about.

Usage:
  tools/c_error_classes.py                ranked classes
  tools/c_error_classes.py --undeclared   split the undeclared symbols by cause
  tools/c_error_classes.py --self-check   negative control

Exit codes:
  0  the report printed
  2  COULD NOT RUN (no t27c, no cc, no specs)
"""

import collections
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# A DIAGNOSTIC, not the word. Clang echoes the offending source line under each
# message, so a spec that contains the text `error: ` in a STRING LITERAL is
# counted as an error by a bare `error:` search: 9 phantom errors in this
# corpus, from three lines of one spec. Anchoring on `file:line:col: error: `
# is what separates what the compiler SAID from what it QUOTED.
DIAG = re.compile(r"^\S+:\d+:\d+: error: (.*)$", re.M)


def t27c() -> str:
    for rel in ("target/release/t27c", "bootstrap/target/release/t27c"):
        p = os.path.join(ROOT, rel)
        if os.path.isfile(p) and os.access(p, os.X_OK):
            return p
    env = os.environ.get("TRI_T27C", "")
    if env and os.access(env, os.X_OK):
        return env
    print("c_error_classes: t27c not built. Exit 2 = COULD NOT RUN.", file=sys.stderr)
    print("  cargo build --release -p t27c, or set TRI_T27C.", file=sys.stderr)
    sys.exit(2)


def cc_uncap_flag() -> str:
    """The flag that ACTUALLY removes the cap, per vendor -- gcc rejects
    `-ferror-limit` and clang ACCEPTS `-fmax-errors=0` while ignoring it."""
    v = subprocess.run(["cc", "--version"], capture_output=True, text=True).stdout
    return "-ferror-limit=0" if "clang" in v.lower() else "-fmax-errors=0"


def diagnose(binary: str, out_dir: str) -> str:
    specs = sorted(
        os.path.join(r, f)
        for r, _, fs in os.walk(os.path.join(ROOT, "specs"))
        for f in fs
        if f.endswith(".t27")
    )
    if not specs:
        print("c_error_classes: no specs. Exit 2.", file=sys.stderr)
        sys.exit(2)
    flag = cc_uncap_flag()
    chunks = []
    for s in specs:
        h = os.path.join(out_dir, os.path.relpath(s, ROOT).replace("/", "_") + ".h")
        with open(h, "w") as fh:
            subprocess.run([binary, "gen-c", s], stdout=fh, stderr=subprocess.DEVNULL)
        if os.path.getsize(h) == 0:
            continue
        r = subprocess.run(
            ["cc", "-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter",
             flag, "-fsyntax-only", "-x", "c", h],
            capture_output=True, text=True,
        )
        chunks.append(f"### {os.path.basename(h)}\n" + r.stdout + r.stderr)
    return "\n".join(chunks)


def undeclared_split(text: str) -> None:
    """Why is each undeclared function undeclared? The causes need different
    repairs, and the totals do not separate them."""
    names = collections.Counter(re.findall(r"call to undeclared function '(\w+)'", text))
    fns, mods, files = set(), set(), set()
    for r, _, fs in os.walk(os.path.join(ROOT, "specs")):
        for f in fs:
            if not f.endswith(".t27"):
                continue
            files.add(f[:-4])
            for line in open(os.path.join(r, f), encoding="utf-8", errors="replace"):
                m = re.match(r"\s*(?:pub\s+)?fn\s+(\w+)\s*\(", line)
                if m:
                    fns.add(m.group(1))
                m = re.match(r"\s*module\s+([\w\-]+)", line)
                if m:
                    mods.add(m.group(1).replace("-", "_"))
    cats, ex = collections.Counter(), collections.defaultdict(list)
    for n, c in names.items():
        if n in fns:
            k = "A  declared as `fn` in some spec, not reaching this file"
        elif n in files or n in mods:
            k = "B  the name of a MODULE or spec FILE, called as a function"
        else:
            k = "C  a name nothing in the tree declares"
        cats[k] += c
        ex[k].append((n, c))
    total = sum(names.values()) or 1
    print(f"\nundeclared functions: {total} errors across {len(names)} names")
    for k in sorted(cats):
        print(f"  {cats[k]:5} ({100 * cats[k] // total:2}%)  {k}")
        print("        " + ", ".join(f"{n}({v})" for n, v in sorted(ex[k], key=lambda t: -t[1])[:6]))


def self_check(binary: str) -> int:
    """A planted error must be counted, and a clean file must count zero.
    Without both, a report of "0" and a report that could not run look alike."""
    ok = True
    with tempfile.TemporaryDirectory() as d:
        bad = os.path.join(d, "bad.h")
        open(bad, "w").write("int f(void){ return undefined_zzz; }\n")
        r = subprocess.run(
            ["cc", "-std=c11", cc_uncap_flag(), "-fsyntax-only", "-x", "c", bad],
            capture_output=True, text=True,
        )
        n = len(DIAG.findall(r.stdout + r.stderr))
        print(f"  planted 1 error -> counted {n} {'PASS' if n >= 1 else 'FAIL'}")
        ok &= n >= 1
        good = os.path.join(d, "good.h")
        open(good, "w").write("int f(void){ return 0; }\n")
        r = subprocess.run(
            ["cc", "-std=c11", cc_uncap_flag(), "-fsyntax-only", "-x", "c", good],
            capture_output=True, text=True,
        )
        n = len(DIAG.findall(r.stdout + r.stderr))
        print(f"  clean file      -> counted {n} {'PASS' if n == 0 else 'FAIL'}")
        ok &= n == 0
        # The case that found the defect: a file that is CLEAN except that it
        # contains the text `error: ` inside a string. A bare `error:` search
        # counts the compiler's echo of that line and reports 1.
        # The error must be ON the quoted line, or clang never echoes it and
        # the control cannot fail. The first version of this fixture put the
        # string on its own clean line and passed for the wrong reason.
        quoted = os.path.join(d, "quoted.h")
        open(quoted, "w").write(
            'const char *f(void){ return ("error: " zzz_undeclared); }\n'
        )
        r = subprocess.run(
            ["cc", "-std=c11", cc_uncap_flag(), "-fsyntax-only", "-x", "c", quoted],
            capture_output=True, text=True,
        )
        n = len(DIAG.findall(r.stdout + r.stderr))
        raw = len(re.findall(r"error:", r.stdout + r.stderr))
        print(f"  1 error on a line quoting \"error: \" -> counted {n}, "
              f"unanchored counts {raw} "
              f"{'PASS' if n == 1 and raw > n else 'FAIL'}")
        ok &= n == 1 and raw > n
    return 0 if ok else 2


def main() -> int:
    binary = t27c()
    if subprocess.run(["cc", "--version"], capture_output=True).returncode != 0:
        print("c_error_classes: no cc. Exit 2 = COULD NOT RUN.", file=sys.stderr)
        return 2
    if "--self-check" in sys.argv:
        return self_check(binary)
    with tempfile.TemporaryDirectory() as d:
        text = diagnose(binary, d)
    v = subprocess.run(["cc", "--version"], capture_output=True, text=True).stdout
    print(f"instrument: {v.strip().splitlines()[0][:64]}  {cc_uncap_flag()}")
    files = text.count("### ")
    errors = len(DIAG.findall(text))
    clean = sum(1 for c in text.split("### ")[1:] if not DIAG.search(c))
    print(f"translation units {files}   errors {errors}   compiling {clean}\n")
    classes = collections.Counter(
        re.sub(r"\d+", "N", m.split("[")[0]) for m in DIAG.findall(text)
    )
    for msg, n in classes.most_common(12):
        print(f"  {n:5}  {msg.strip()[:70]}")
    if "--undeclared" in sys.argv:
        undeclared_split(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
