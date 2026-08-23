#!/usr/bin/env python3
"""Do the copies of a duplicated function still agree with each other?

`tmul` is defined in **14** specs under specs/ternary/. `dot27` in 13, `quantize` in 9.
(Those said 9 and 7 until T79. They were the counts this gate COMPARED, printed
as the counts the tree DEFINES -- 6 of 36 spec-function pairs vanished silently
at the `if d:` join because the harness would not build against them.)
Nothing checked that the copies still compute the same thing, and copies drift.

Text comparison is the wrong instrument, and trying it first is what motivated this
file. Hashing the normalised source reported "2 variants of tmul, 3 of dot27, 3 of
quantize" -- all artefacts. `bitnet_mlp3.t27` writes its functions on one line, so a
regex ending at `\\n}` swallowed five following definitions; and even with balanced-brace
extraction, `if(ta==1)` and `if (ta == 1)` hash differently while computing the same
thing. Every one of those "divergences" was a finding about the comparison, not the
repository.

So this compares BEHAVIOUR: each spec is compiled to C, the named function is extracted
from *that spec's own* output, and the same FNV-1a digest is folded over a fixed domain.
Specs whose copies compute the same function produce the same digest, whatever their
formatting.

Result at the time of writing: one behaviour each, across all 14 / 13 / 9 specs. That is
a negative result, and it is the point -- the risk is real and now something watches it.

Usage:
  tools/check_duplicate_agreement.py               gate
  tools/check_duplicate_agreement.py --self-check  negative control

Exits non-zero if two specs define the same function with different behaviour.
"""
import glob
import os
import re
import subprocess
import sys
import tempfile

# T73: overridable so the negative control can point the WHOLE program at a
# planted tree and run it end to end. Nothing in the repository sets it; if it
# is set to a directory without a built t27c, the tool exits loudly rather than
# passing.
ROOT = os.environ.get("T27_DUP_ROOT") or os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))

# name -> (C signature regex, extra functions it needs, the enumeration body)
# T79: keyed by CASE, not by function name. `quantize` genuinely exists at two
# arities in this corpus, so it needs two harnesses while still counting as ONE
# source name -- which is why the first element is the name in the .t27 source.
#
# The regexes used to be narrower than the corpus. `dot27` was pinned to
# `int16_t`, so the four copies emitting `int8_t` were extracted, failed to
# compile against the harness, and vanished with no output: the gate reported
# "one behaviour across 9 specs" while 13 specs define it. Relaxing the return
# type and adding `tp` as a dep brings all 13 in -- and the 13-spec digest is
# byte-identical to the 9-spec one it replaces, which is the proof that the
# "9" was an artefact of the regex and never a signature boundary.
CASES = {
    "tmul": ("tmul", r"int8_t\s+tmul\s*\([^)]*\)\s*\{", [],
             'for(int a=0;a<256;a++)for(int b=0;b<256;b++){'
             'unsigned v=(unsigned)((long long)tmul((uint8_t)a,(uint8_t)b)&0xFFFFFFFF);'
             'h=(h^v)*16777619u;}'),
    "quantize2": ("quantize", r"uint8_t\s+quantize\s*\(\s*int16_t[^)]*\)\s*\{", [],
                  'for(int a=-32768;a<32768;a++)for(int b=0;b<64;b++){'
                  'unsigned v=(unsigned)quantize((int16_t)a,(int16_t)b);h=(h^v)*16777619u;}'),
    "quantize1": ("quantize", r"uint8_t\s+quantize\s*\(\s*int8_t\s+\w+\s*\)\s*\{", [],
                  'for(int a=-128;a<128;a++){'
                  'unsigned v=(unsigned)quantize((int8_t)a);h=(h^v)*16777619u;}'),
    "dot27": ("dot27", r"int(?:8|16|32)_t\s+dot27\s*\([^)]*\)\s*\{",
              [r"int8_t\s+tmul\s*\([^)]*\)\s*\{", r"int8_t\s+tp\s*\([^)]*\)\s*\{"],
              'for(unsigned k=0;k<200000;k++){uint64_t x=k*2654435761ULL,'
              'y=k*40503ULL+12009599006321322ULL;'
              'unsigned v=(unsigned)((long long)dot27(x,y)&0xFFFFFFFF);h=(h^v)*16777619u;}'),
}

# A function DEFINED in the .t27 source, read before gen-c runs, so a spec that
# fails to generate still counts as defining it.
DEF_RE = {n: re.compile(r"^\s*(?:pub\s+)?fn\s+" + n + r"\s*\(", re.M)
          for n in {c[0] for c in CASES.values()}}


def t27c():
    for p in ("target/release/t27c", "target/debug/t27c"):
        c = os.path.join(ROOT, p)
        if os.path.exists(c):
            return c
    sys.exit("FAIL: t27c not built. Run: cargo build --release -p t27c")


def extract(src, sig_re):
    """Balanced-brace extraction. A regex ending at a newline-brace is not enough:
    one spec writes its functions on a single line."""
    m = re.search(sig_re, src)
    if not m:
        return None
    i = src.index("{", m.end() - 1)
    d = 0
    for j in range(i, len(src)):
        if src[j] == "{":
            d += 1
        elif src[j] == "}":
            d -= 1
            if d == 0:
                return src[m.start():j + 1]
    return None


def digest_for(cbin, name, sig, deps, loop, csrc, wd, tag=""):
    parts = [extract(csrc, d) for d in deps] + [extract(csrc, sig)]
    if parts[-1] is None:
        return None
    body = "\n".join(p for p in parts if p)
    prog = ("#include <stdio.h>\n#include <stdint.h>\n#define assert_eq(x,y) ((void)0)\n"
            + body + "\nint main(void){unsigned h=2166136261u;" + loop
            + 'printf("%08x\\n",h);return 0;}')
    c = os.path.join(wd, f"d{tag}.c")
    open(c, "w").write(prog)
    b = os.path.join(wd, f"d{tag}")
    try:
        failed = subprocess.run(["cc", "-O2", "-o", b, c], capture_output=True).returncode
    except OSError:
        # T116: the compiler being ABSENT, which is not the compiler failing.
        # Without this the call raises FileNotFoundError and the gate dies with
        # a traceback -- exit 1 and no verdict, so "cc is not installed" and
        # "two copies of a function disagree" leave the same colour and the
        # same silence. Found by sweeping every gate for the shape that
        # verify_exhaustive.py had (#2515).
        #
        # None here means "no digest", which the caller already treats as
        # uncompared rather than as a disagreement -- so the absence flows into
        # the count of what could not be measured, where it belongs.
        print("  cc is not on PATH -- no C copy could be compiled, nothing compared")
        return None
    if failed:
        return None
    r = subprocess.run([b], capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else None


def scan(wd):
    """(digest groups, defined, compared).

    T79: `defined` is read from the .t27 SOURCE before gen-c runs, because a
    copy that vanishes is exactly what this gate was blind to. Two paths lost
    copies with no output -- `if r.returncode: continue` (the spec does not
    generate) and `if d:` (it generates, but the harness will not compile
    against it). Six of thirty-six spec-function pairs took the second one.
    """
    t = t27c()
    out = {}
    defined = {}
    compared = {}
    for f in sorted(glob.glob(os.path.join(ROOT, "specs/ternary/*.t27"))):
        base = os.path.basename(f)
        try:
            src = open(f, encoding="utf-8", errors="replace").read()
        except OSError:
            src = ""
        for fn, rx in DEF_RE.items():
            if rx.search(src):
                defined.setdefault(fn, set()).add(base)
        r = subprocess.run([t, "gen-c", f], capture_output=True, text=True, cwd=ROOT)
        if r.returncode:
            continue
        for case_id, (fn, sig, deps, loop) in CASES.items():
            d = digest_for(t, case_id, sig, deps, loop, r.stdout, wd, case_id)
            if d:
                out.setdefault(case_id, {}).setdefault(d, []).append(base)
                compared.setdefault(fn, set()).add(base)
    return out, defined, compared


FIXTURE = """module DupFixture{tag}

fn tmul(ta: i8, tb: i8) -> i8 {{
    if (ta == 0) {{ return 0; }}
    if (tb == 0) {{ return 0; }}
    if (ta == tb) {{ return {agree}; }}
    return {differ};
}}

test t_{tag}
    given a = tmul(1, 1)
    then a == {agree}
"""


def self_check():
    """Plant a real disagreement and run THIS WHOLE FILE against it.

    T73: the previous control built a literal `fake = {"x": {...}}`, evaluated
    a comprehension written inside itself, and returned before the reporting
    block. It proved that the copy of the comparison living in the control
    worked. Measured: three mutants of the REAL logic -- the verdict flag
    inverted, the digest grouping key destroyed, a bare `return 0` planted
    ahead of the report -- each let a genuinely divergent tree pass with exit
    0, and this control stayed green for two of them with identical output.

    A control has to execute the thing it certifies. This one spawns the gate
    as a subprocess against a planted tree, so scan(), the grouping, the report
    block and main()'s own return value all run. Both the message and the exit
    code are asserted: a fixture that stops compiling makes the child exit 1
    through the unrelated "no duplicated function found" guard, so an
    exit-code-only assertion would go wrongly green.
    """
    t = t27c()
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "specs/ternary"))
        os.makedirs(os.path.join(td, "target/release"))
        os.symlink(t, os.path.join(td, "target/release/t27c"))
        for tag, agree, differ in (("A", "1", "-1"), ("B", "-1", "1")):
            with open(os.path.join(td, f"specs/ternary/{tag.lower()}.t27"), "w") as fh:
                fh.write(FIXTURE.format(tag=tag, agree=agree, differ=differ))
        r = subprocess.run([sys.executable, os.path.abspath(__file__)],
                           capture_output=True, text=True,
                           env={**os.environ, "T27_DUP_ROOT": td})
    split = "FAIL tmul" in r.stdout and "DIFFERENT behaviours" in r.stdout
    print(f"  self-check: planted divergence reported as a split = {split}")
    print(f"  self-check: the gate's exit code on it = {r.returncode} (want 1)")
    # T98: the AGREEING direction. Every assertion above demands RED, and a
    # control made only of those is blind to every mutation that makes the gate
    # LOUDER -- the one-group branch rewritten to a constant false reports a
    # split on a tree where the copies agree, and every assertion above is still
    # satisfied. Measured: two such mutations passed this control and were
    # caught only by --self-check-drop, which exists for something else.
    # Coverage by a sibling's accident is not coverage.
    #
    # The branch is described rather than quoted on purpose: the first version
    # of this comment carried the line verbatim, and a text-replacing mutation
    # harness then hit the COMMENT instead of the code -- reporting the case as
    # blind when the gate had never been mutated at all.
    #
    # Same fixture for both copies, so they genuinely agree.
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "specs/ternary"))
        os.makedirs(os.path.join(td, "target/release"))
        os.symlink(t, os.path.join(td, "target/release/t27c"))
        for tag in ("A", "B"):
            with open(os.path.join(td, "specs/ternary/%s.t27" % tag.lower()), "w") as fh:
                fh.write(FIXTURE.format(tag=tag, agree="1", differ="-1"))
        q = subprocess.run([sys.executable, os.path.abspath(__file__)],
                           capture_output=True, text=True,
                           env={**os.environ, "T27_DUP_ROOT": td})
    quiet = ("DIFFERENT behaviours" not in q.stdout) and q.returncode == 0
    print("  self-check: agreeing copies reported as one behaviour = %s (exit %d, want 0)"
          % (quiet, q.returncode))
    # T116: the compiler being ABSENT. Both cases above run with a working cc,
    # so the branch that fires when it is missing was unreachable from this
    # control -- and until this commit the gate did not have that branch at all:
    # it raised FileNotFoundError, exiting 1 with no verdict, so "cc is not
    # installed" and "two copies disagree" left the same colour and the same
    # silence.
    #
    # The agreeing fixture is reused, so the ONLY thing wrong with this world is
    # the missing tool. `DIFFERENT behaviours` is named absent because reporting
    # an absence as a disagreement is the failure this branch exists to prevent.
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "specs/ternary"))
        os.makedirs(os.path.join(td, "target/release"))
        os.symlink(t, os.path.join(td, "target/release/t27c"))
        for tag in ("A", "B"):
            with open(os.path.join(td, "specs/ternary/%s.t27" % tag.lower()), "w") as fh:
                fh.write(FIXTURE.format(tag=tag, agree="1", differ="-1"))
        env = {**os.environ, "T27_DUP_ROOT": td}
        env["PATH"] = os.pathsep.join(
            d for d in env.get("PATH", "").split(os.pathsep)
            if d and not os.path.exists(os.path.join(d, "cc")))
        n = subprocess.run([sys.executable, os.path.abspath(__file__)],
                           capture_output=True, text=True, env=env)
    named = ("cc is not on PATH" in n.stdout
             and "DIFFERENT behaviours" not in n.stdout
             and "Traceback" not in n.stderr
             and n.returncode != 0)
    print("  self-check: a missing compiler is named, not reported as a split = %s "
          "(exit %d, traceback %s)"
          % (named, n.returncode, "YES" if "Traceback" in n.stderr else "no"))
    return 0 if (split and r.returncode == 1 and quiet and named) else 1


DROP_FIXTURE = """module DupFixtureDrop

fn tmul(ta: i8, tb: i8, tc: i8) -> i8 {
    if (ta == 0) { return 0; }
    if (tb == 0) { return 0; }
    if (tc == 0) { return 0; }
    return 1;
}

test t_drop
    given a = tmul(1, 1, 1)
    then a == 1
"""


def self_check_drop():
    """Plant a copy the harness CANNOT build against, and require it be named.

    T79: this is a different branch from the divergence control above. That one
    exercises the digest split (`len(groups) > 1`); this one exercises the drop
    join `if d:` reached via a compile failure -- the same path the two real
    one-argument `quantize` copies take.

    A three-argument `tmul` is deliberate. The signature pattern still matches
    it, so extraction succeeds and the fault survives into the harness, where
    cc rejects the fixed two-argument call. A return-type mismatch would be the
    wrong fault: it stops being planted the day someone widens a regex, which
    is the very repair this gate is supposed to survive.

    The assertion names the branch rather than trusting the exit code: a
    fixture that started DISAGREEING would also exit 1, through the split, so
    "DIFFERENT behaviours" must be absent.
    """
    t = t27c()
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "specs/ternary"))
        os.makedirs(os.path.join(td, "target/release"))
        os.symlink(t, os.path.join(td, "target/release/t27c"))
        with open(os.path.join(td, "specs/ternary/a.t27"), "w") as fh:
            fh.write(FIXTURE.format(tag="A", agree="1", differ="-1"))
        with open(os.path.join(td, "specs/ternary/drop.t27"), "w") as fh:
            fh.write(DROP_FIXTURE)
        r = subprocess.run([sys.executable, os.path.abspath(__file__)],
                           capture_output=True, text=True,
                           env={**os.environ, "T27_DUP_ROOT": td})
    named = "DROPPED 1 of 2 copies" in r.stdout and "drop.t27" in r.stdout
    right_branch = "DIFFERENT behaviours" not in r.stdout
    print(f"  self-check-drop: uncompared copy named as dropped = {named}")
    print(f"  self-check-drop: reported as a DROP, not a split = {right_branch}")
    print(f"  self-check-drop: the gate's exit code on it = {r.returncode} (want 1)")
    return 0 if (named and right_branch and r.returncode == 1) else 1


def main():
    if "--self-check-drop" in sys.argv:
        return self_check_drop()
    if "--self-check" in sys.argv:
        return self_check()
    with tempfile.TemporaryDirectory() as wd:
        found, defined, compared = scan(wd)

        # T79: a copy that was never compared is not evidence of agreement.
        # The docstring reported COMPARED counts as DEFINED counts -- "one
        # behaviour across 9 specs" for dot27 while 13 specs define it.
        dropped = []
        for fn, specs_ in sorted(defined.items()):
            miss = sorted(specs_ - compared.get(fn, set()))
            if miss:
                dropped.append((fn, len(miss), len(specs_), miss))

        if not found:
            print("FAIL: no duplicated function was found at all -- the extraction is broken, "
                  "not the tree (tmul alone is defined in 14 specs)")
            return 1
        bad = False
        for name, groups in sorted(found.items()):
            n = sum(len(v) for v in groups.values())
            if len(groups) == 1:
                print(f"OK   {name:<10} one behaviour across {n:>2} specs   "
                      f"digest {list(groups)[0]}")
            else:
                bad = True
                print(f"FAIL {name:<10} {len(groups)} DIFFERENT behaviours across {n} specs:")
                for d, fs in groups.items():
                    print(f"       {d}: {', '.join(fs)}")
        print()
        if bad:
            print("FAIL: a duplicated function has drifted. The copies are not the same "
                  "function any more, and a spec importing one of them means something "
                  "different from a spec importing the other.")
            return 1
        if dropped:
            for fn, n, tot, miss in dropped:
                print(f"FAIL {fn:<10} DROPPED {n} of {tot} copies -- defined but "
                      f"never compared: {', '.join(miss)}")
            print()
            print("A copy the harness could not build against is not a copy that")
            print("agrees -- it is a copy nobody looked at. Widen the signature")
            print("pattern, add the missing dependency, or give the arity its own")
            print("case; do not let the count of COMPARED specs be printed as the")
            print("count of specs that define the function.")
            return 1
        print("All duplicated functions agree behaviourally. Formatting differs between "
              "specs; behaviour does not.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
