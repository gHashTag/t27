#!/usr/bin/env python3
"""Do the copies of a duplicated function still agree with each other?

`tmul` is defined in **14** specs under specs/ternary/. `dot27` in 9, `quantize` in 7.
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

Result at the time of writing: one behaviour each, across all 14 / 9 / 7 specs. That is
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
CASES = {
    "tmul": (r"int8_t\s+tmul\s*\([^)]*\)\s*\{", [],
             'for(int a=0;a<256;a++)for(int b=0;b<256;b++){'
             'unsigned v=(unsigned)((long long)tmul((uint8_t)a,(uint8_t)b)&0xFFFFFFFF);'
             'h=(h^v)*16777619u;}'),
    "quantize": (r"uint8_t\s+quantize\s*\([^)]*\)\s*\{", [],
                 'for(int a=-32768;a<32768;a++)for(int b=0;b<64;b++){'
                 'unsigned v=(unsigned)quantize((int16_t)a,(int16_t)b);h=(h^v)*16777619u;}'),
    "dot27": (r"int16_t\s+dot27\s*\([^)]*\)\s*\{", [r"int8_t\s+tmul\s*\([^)]*\)\s*\{"],
              'for(unsigned k=0;k<200000;k++){uint64_t x=k*2654435761ULL,'
              'y=k*40503ULL+12009599006321322ULL;'
              'unsigned v=(unsigned)((long long)dot27(x,y)&0xFFFFFFFF);h=(h^v)*16777619u;}'),
}


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
    if subprocess.run(["cc", "-O2", "-o", b, c], capture_output=True).returncode:
        return None
    r = subprocess.run([b], capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else None


def scan(wd):
    t = t27c()
    out = {}
    for f in sorted(glob.glob(os.path.join(ROOT, "specs/ternary/*.t27"))):
        r = subprocess.run([t, "gen-c", f], capture_output=True, text=True, cwd=ROOT)
        if r.returncode:
            continue
        for name, (sig, deps, loop) in CASES.items():
            d = digest_for(t, name, sig, deps, loop, r.stdout, wd, name)
            if d:
                out.setdefault(name, {}).setdefault(d, []).append(os.path.basename(f))
    return out


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
    return 0 if (split and r.returncode == 1) else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    with tempfile.TemporaryDirectory() as wd:
        found = scan(wd)
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
        print("All duplicated functions agree behaviourally. Formatting differs between "
              "specs; behaviour does not.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
