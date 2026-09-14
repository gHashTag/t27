#!/usr/bin/env python3
"""Does each compiler REPORT every error it finds, or stop early?

Every error total published for the generated C in this repository was a floor.
`cc -fsyntax-only | grep -c 'error:'` looks like a count and is not: clang's
default is `-ferror-limit=20`, 141 corpus files reach it, and the corpus emits
15133 errors rather than the 3849 reported (#3448). A repair that certainly
removed errors showed a delta of ZERO -- three files sitting at exactly 20
before and after -- which is how it was found.

So this asks the same question of every instrument, by planting a KNOWN number
of errors and counting what comes back.

Measured 2026-09-08, 50 planted:

    clang, default        20   CAPPED at -ferror-limit=20
    clang, -ferror-limit=0 50  complete
    rustc                  50  complete
    zig                    50  complete
    iverilog               50  complete (two diagnostic LINES per error)
    yosys                   1  ABORTS on the first error

yosys is not capped, it stops: any error COUNT from it is 0 or 1 and means
nothing. Only its pass/fail is a measurement.

Acceptance columns are safe, and that was checked rather than assumed: clang's
exit code is 1 with and without the limit on a failing file, and 0 on a clean
one. A count of FILES that compile is unaffected by a cap on diagnostics per
file.

THE FIXTURES ARE THE HARD PART, and two of the first five were wrong in ways
that looked like findings:

  * the Zig fixture named its functions `f16`, `f32` -- which shadow Zig
    primitives -- so it reported 2 errors of a different kind and read as a cap;
  * the Verilog fixture used implicitly declared identifiers, which yosys
    treats as a WARNING, so it reported 0 errors and read as silence.

Hence `--self-check`: every fixture must produce the expected KIND of
diagnostic before its count is believed.

Usage:
  tools/diagnostic_capacity.py               report, and fail if a cap appeared
  tools/diagnostic_capacity.py --self-check  fixtures only

Exit codes:
  0  every instrument reports as recorded below
  1  an instrument that reported completely now truncates
  2  COULD NOT RUN (an instrument is missing, or a fixture stopped working)
"""

import os
import re
import subprocess
import sys
import tempfile

PLANTED = 50

# name -> (fixture builder, argv builder, diagnostic regex, expected count,
#          note). `expected` is what was MEASURED, not what the manual claims.
# `None` means "report it, do not ratchet it": the default cap is a property of
# the vendor, not of this repository -- clang stops at 20, gcc does not stop at
# all -- so a runner switching compilers must not turn the gate red.
EXPECT = {
    "cc (default)": None,
    "cc (uncapped)": PLANTED,
    "rustc": PLANTED,
    "zig": PLANTED,
    "iverilog": PLANTED,
    "yosys": 1,
}


def c_uncap_flag() -> str:
    """The flag that ACTUALLY removes the per-file cap, per vendor.

    They are not interchangeable and the failure is silent in the worst
    direction: gcc rejects `-ferror-limit`, while clang ACCEPTS
    `-fmax-errors=0` and ignores it -- 20 errors of a planted 50, with no
    diagnostic about the flag. A flag that is accepted and ignored is worse
    than one that is refused, so the self-check below verifies the effect
    rather than the spelling.
    """
    v = run(["cc", "--version"]) or ""
    return "-ferror-limit=0" if "clang" in v.lower() else "-fmax-errors=0"


def fixtures(d: str) -> dict:
    c = os.path.join(d, "c50.c")
    with open(c, "w") as fh:
        fh.write("".join(f"int f{i}(void) {{ return undefined_{i}; }}\n" for i in range(PLANTED)))
    r = os.path.join(d, "r50.rs")
    with open(r, "w") as fh:
        fh.write("".join(f"pub fn f{i}() -> i32 {{ undefined_{i} }}\n" for i in range(PLANTED)))
    # NOT `f16`/`f32`: those shadow Zig primitives and the file fails for a
    # different reason before the undefined names are ever reached.
    z = os.path.join(d, "z50.zig")
    with open(z, "w") as fh:
        fh.write("".join(f"pub fn probe_{i}() i32 {{ return undefined_{i}; }}\n" for i in range(PLANTED)))
    v = os.path.join(d, "v50.v")
    with open(v, "w") as fh:
        fh.write("module m;\n" + "".join(f"  assign w{i} = undefined_{i};\n" for i in range(PLANTED)) + "endmodule\n")
    # yosys treats an implicit declaration as a WARNING, so the iverilog
    # fixture measures nothing there. A missing module is a real yosys error.
    y = os.path.join(d, "y50.v")
    with open(y, "w") as fh:
        fh.write("".join(f"module m{i}; nosuchmod_{i} u{i} (); endmodule\n" for i in range(PLANTED)))
    return {"c": c, "rs": r, "zig": z, "v": v, "y": y}


def run(argv, cwd=None) -> str:
    try:
        p = subprocess.run(argv, capture_output=True, text=True, cwd=cwd)
    except FileNotFoundError:
        return None
    return p.stdout + p.stderr


def counts(d: str, f: dict) -> dict:
    out = os.path.join(d, "out")
    os.makedirs(out, exist_ok=True)
    got = {}

    t = run(["cc", "-std=c11", "-fsyntax-only", f["c"]])
    got["cc (default)"] = None if t is None else len(re.findall(r"error:", t))
    t = run(["cc", "-std=c11", c_uncap_flag(), "-fsyntax-only", f["c"]])
    got["cc (uncapped)"] = None if t is None else len(re.findall(r"error:", t))

    t = run(["rustc", "--edition", "2021", "--crate-type", "lib", "--crate-name", "m",
             "-A", "warnings", "--emit=metadata", "-o", os.path.join(out, "m.rmeta"), f["rs"]])
    # `error: aborting due to N previous errors` is a summary, not an error.
    got["rustc"] = None if t is None else len(
        [l for l in t.splitlines() if l.startswith("error") and "aborting due to" not in l])

    t = run(["zig", "build-obj", f["zig"], "-femit-bin=" + os.path.join(out, "z.o")])
    got["zig"] = None if t is None else len(re.findall(r"error:", t))

    t = run(["iverilog", "-o", os.path.join(out, "v.vvp"), f["v"]])
    # TWO diagnostic lines per planted error ("Unable to bind" and "Unable to
    # elaborate"), so the unit is the SOURCE LINE cited, not the line printed.
    got["iverilog"] = None if t is None else len(
        set(re.findall(r"^[^\s:]+\.v:(\d+):\s*error:", t, re.M)))

    t = run(["yosys", "-q", "-p", f"read_verilog {f['y']}; hierarchy -check"])
    got["yosys"] = None if t is None else len(re.findall(r"(?m)^ERROR:", t))
    return got


def self_check(d: str, f: dict) -> int:
    """Each fixture must produce the KIND of diagnostic it was written for.

    Two of the first five did not, and both read as findings about the
    instrument: the Zig fixture tripped over primitive shadowing, and the
    Verilog one produced warnings where yosys needed errors.
    """
    ok = True
    present = 0
    checks = [
        # NOT the vendor's wording. This asserted clang's phrasing
        # ("use of undeclared identifier") and went red on a gcc runner, where
        # the same defect reads "'undefined_0' undeclared". The identifier is
        # the part every C compiler must name.
        ("cc", ["cc", "-std=c11", c_uncap_flag(), "-fsyntax-only", f["c"]], r"undefined_0"),
        ("rustc", ["rustc", "--edition", "2021", "--crate-type", "lib", "--crate-name", "m",
                   "-A", "warnings", "--emit=metadata", "-o", os.path.join(d, "m.rmeta"), f["rs"]],
         r"undefined_0"),
        ("zig", ["zig", "build-obj", f["zig"], "-femit-bin=" + os.path.join(d, "z.o")], r"undefined_0"),
        ("iverilog", ["iverilog", "-o", os.path.join(d, "v.vvp"), f["v"]], r"undefined_0"),
        ("yosys", ["yosys", "-q", "-p", f"read_verilog {f['y']}; hierarchy -check"], r"nosuchmod_0"),
    ]
    for name, argv, want in checks:
        t = run(argv)
        if t is None:
            # NOT a failure: a runner without zig or yosys can still check the
            # fixtures for the compilers it has, and the one that matters --
            # clang, the capped one -- is everywhere. Named, never dropped.
            print(f"  {name:10} NOT ON PATH, not checked")
            continue
        present += 1
        hit = re.search(want, t) is not None
        print(f"  {name:10} expects /{want}/ -> {'PASS' if hit else 'FAIL'}")
        ok &= hit
    # The flag must have an EFFECT, not merely be accepted. clang takes
    # `-fmax-errors=0` and ignores it; nothing in its output says so.
    t = run(["cc", "-std=c11", c_uncap_flag(), "-fsyntax-only", f["c"]])
    if t is not None:
        n = len(re.findall(r"error:", t))
        good = n >= PLANTED
        print(f"  {'cc uncap':10} {c_uncap_flag()} reports {n} of {PLANTED} -> {'PASS' if good else 'FAIL'}")
        ok &= good
    if present == 0:
        # A check that could not run has not passed.
        print("  no instrument on PATH. Exit 2 = COULD NOT RUN.", file=sys.stderr)
        return 2
    print(f"  checked {present} of {len(checks)} fixtures")
    return 0 if ok else 1


def main() -> int:
    with tempfile.TemporaryDirectory() as d:
        f = fixtures(d)
        if "--self-check" in sys.argv:
            return self_check(d, f)
        got = counts(d, f)
        # WHICH MACHINE. The same command reported 20 here and 50 on CI,
        # because one `cc` is Apple clang and the other is gcc -- and no report
        # said where it ran. A diagnostic count measures the code, the
        # instrument AND the machine; only the first was ever written down.
        print("instruments:")
        for tool, argv in (("cc", ["cc", "--version"]), ("rustc", ["rustc", "--version"]),
                           ("zig", ["zig", "version"]), ("iverilog", ["iverilog", "-V"]),
                           ("yosys", ["yosys", "-V"])):
            v = run(argv)
            line = v.strip().splitlines()[0][:64] if v and v.strip() else "not on PATH"
            print(f"  {tool:9} {line}")
        print(f"\nplanted {PLANTED} errors per language\n")
        bad = False
        absent = []
        checked = 0
        for name, want in EXPECT.items():
            n = got.get(name)
            if n is None:
                print(f"  {name:24}    -   NOT ON PATH, not checked")
                absent.append(name)
                continue
            checked += 1
            if want is None:
                cap = "TRUNCATES at %d" % n if n < PLANTED else "no default cap"
                print(f"  {name:24} {n:4}   {cap}   (reported, not ratcheted)")
                continue
            verdict = "complete" if n >= PLANTED else ("ABORTS on the first" if n <= 1 else "TRUNCATES")
            flag = "" if n == want else "   <-- CHANGED, recorded " + str(want)
            print(f"  {name:24} {n:4}   {verdict}{flag}")
            if n < want:
                bad = True
        print()
        print("Acceptance counts are unaffected: clang's exit code is 1 with and")
        print("without the limit on a failing file, and 0 on a clean one -- so a")
        print("count of FILES that compile cannot be censored by a per-file cap.")
        if absent:
            # NAMED, not silently dropped: a run that checked two instruments
            # and a run that checked six print different things.
            print(f"\nNOT CHECKED ({len(absent)} of {len(EXPECT)}): {', '.join(absent)}")
            print("Install them to widen this, or read the result as covering the rest.")
        if checked == 0:
            # A check that could not run has not passed.
            print("\nNo instrument was available. Exit 2 = COULD NOT RUN.", file=sys.stderr)
            return 2
        print(f"\nchecked {checked} of {len(EXPECT)}")
        return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
