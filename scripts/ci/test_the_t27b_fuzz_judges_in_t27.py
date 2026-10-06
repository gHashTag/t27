#!/usr/bin/env python3
"""tri t27b fuzz generates and judges in t27 (#6442, owner-approved exception).

The program generator is specs/tri/t27b/fuzz.t27 and the judge is
specs/tri/t27b/fuzz_oracle.t27. Python reaches them only as
gen/c/tri/t27b/fuzz.c and fuzz_oracle.c (t27c gen-c, L2). No t27c, no t27b
and no network here:

  1. the generated C of each spec passes every one of that spec's tests;
  2. the generator is deterministic: the same seed and case give the same
     bytes twice, another seed gives other bytes, and every committed
     regression fixture under tests/fixtures/t27b-fuzz/ is still exactly what
     its seed and case render;
  3. the judge, over a corpus JSON fixture shaped like `t27b corpus
     --reference`: agreement, a t27b bug, a reference bug, a reference that
     refuses to compile, a t27b timeout, and both sides failing an
     expect_pass test (a wrong checksum) -- the last one MUST be a
     disagreement;
  4. mutation control: a copy of fuzz_oracle.c that calls the both-fail case
     AGREE makes check 3 fail, so the check can fail.
"""

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "tri_loop"))
import t27b_fuzz  # noqa: E402

FIXTURES = ROOT / "tests" / "fixtures" / "t27b-fuzz"
fails = 0


def check(ok, what):
    global fails
    print(("ok      " if ok else "FAIL    ") + what)
    fails += 0 if ok else 1


def runs_tests(src):
    with tempfile.TemporaryDirectory() as tmp:
        exe = os.path.join(tmp, "spec_tests")
        b = subprocess.run([os.environ.get("CC", "cc"), "-DT27_TEST_MAIN", "-w", "-o", exe, str(src)],
                           capture_output=True, text=True)
        if b.returncode != 0:
            return -1, b.stderr[:300]
        r = subprocess.run([exe], capture_output=True, text=True)
        return r.returncode, r.stdout.strip()


# 1. Each spec's own tests, run from its generated C.
for spec, gen in (("fuzz.t27", "fuzz.c"), ("fuzz_oracle.t27", "fuzz_oracle.c")):
    declared = len(re.findall(r"^\s*test\s+[A-Za-z_][A-Za-z0-9_]*\s*\{",
                              (ROOT / "specs/tri/t27b" / spec).read_text(), re.M))
    code, out = runs_tests(ROOT / "gen/c/tri/t27b" / gen)
    m = re.search(r"All (\d+) tests passed", out)
    check(code == 0 and declared > 0 and m is not None and int(m.group(1)) == declared,
          f"gen/c/tri/t27b/{gen} passes every one of {spec}'s {declared} tests ({code}: {out[:120]})")

# 2. Determinism, and the committed fixtures still render from their seed.
a, b = t27b_fuzz.render(20261006, 7), t27b_fuzz.render(20261006, 7)
check(a is not None and a == b and "module FuzzCase;" in a, "the same seed and case render the same bytes")
check(t27b_fuzz.render(20261007, 7) != a, "another seed renders another case")
index = json.loads((FIXTURES / "findings.json").read_text()) if (FIXTURES / "findings.json").exists() else []
check(len(index) > 0, f"tests/fixtures/t27b-fuzz/findings.json lists the committed findings ({len(index)})")
for seed, case, name in sorted({(f["seed"], f["case"], f["file"]) for f in index}):
    got = t27b_fuzz.render(seed, case)
    check(got == (FIXTURES / name).read_text(), f"fixture {name} is what seed {seed} case {case} renders")


# 3. The judge over a corpus fixture.
def row(case, t27b="pass", tv=None, reference="pass", rt=None, detail=""):
    r = {"file": f"/tmp/x/{t27b_fuzz.case_file(case)}", "t27b": t27b, "reference": reference,
         "reference_detail": detail, "detail": ""}
    if tv is not None:
        r["test_verdicts"] = tv
    if rt is not None:
        r["reference_tests"] = rt
    return r


SEED = 20261006
sources = {}
for k in range(6):
    src = t27b_fuzz.render(SEED, k)
    sources[t27b_fuzz.case_file(k)] = (k, src)


def first_test(k):
    return t27b_fuzz.TEST_RE.findall(sources[t27b_fuzz.case_file(k)][1])[0]


def all_tests(k):
    return {n: n.endswith("_expect_pass")
            for n in t27b_fuzz.TEST_RE.findall(sources[t27b_fuzz.case_file(k)][1])}


def corpus():
    rows = [row(0, tv=all_tests(0), rt=all_tests(0))]                       # agree
    bad = all_tests(1)
    bad[first_test(1)] = not bad[first_test(1)]
    rows.append(row(1, t27b="fail", tv=bad, rt=all_tests(1)))                 # t27b bug
    wrong_ref = all_tests(2)
    wrong_ref[first_test(2)] = not wrong_ref[first_test(2)]
    rows.append(row(2, tv=all_tests(2), reference="fail", rt=wrong_ref))      # reference bug
    rows.append(row(3, t27b="blocked", reference="blocked", detail="does not compile"))  # both refuse
    rows.append(row(4, t27b="timeout", rt=all_tests(4)))                      # unjudged
    # A wrong checksum: both sides fail every expect_pass test; a kept trap still fails as it should.
    wrong = {t: False for t in all_tests(5)}
    rows.append(row(5, t27b="fail", tv=wrong, reference="fail", rt=wrong))
    return {"results": rows}


def judged():
    return t27b_fuzz.judge(corpus(), sources, SEED)


s = judged()
n = {k: len(t27b_fuzz.TEST_RE.findall(sources[t27b_fuzz.case_file(k)][1])) for k in range(6)}
c = s["classes"]
pass5 = sum(1 for t in t27b_fuzz.TEST_RE.findall(sources[t27b_fuzz.case_file(5)][1]) if t.endswith("_expect_pass"))
check(s["cases"] == 6 and s["tests"] == sum(n.values()), f"every generated test is judged ({s['tests']})")
check(c["T27B_BUG"] == 1 and c["REFERENCE_BUG"] == 1, f"one t27b bug and one reference bug ({c})")
check(c["REFERENCE_REJECTED"] == n[3] and s["rejected"] == n[3], "a case both sides refuse is rejected, not agree")
check(c["UNJUDGED"] == n[4] and s["unjudged"] == n[4], "a t27b timeout leaves its tests unjudged")
check(pass5 > 0 and c["SEMANTICS_GAP"] == pass5,
      f"a wrong checksum (both sides fail an expect_pass test) is a semantics gap ({c['SEMANTICS_GAP']} of {pass5})")
check(s["disagree"] == 2 + pass5, f"disagree counts the wrong answers ({s['disagree']})")
check(s["seeds"] == [f"{SEED}:{k}" for k in (1, 2, 3, 5)], f"seeds name the cases behind the findings ({s['seeds']})")
check(s["agree"] + s["disagree"] + s["rejected"] + s["unjudged"] == s["tests"], "the four counts add up to the tests")

# 4. Mutation control: an oracle that calls the both-fail case AGREE.
gen = (ROOT / "gen/c/tri/t27b/fuzz_oracle.c").read_text()
needle = "return CLS_SEMANTICS_GAP;"
check(gen.count(needle) == 1, "mutation: the semantics-gap return is present exactly once")
with tempfile.TemporaryDirectory() as tmp:
    mutant = Path(tmp) / "fuzz_oracle.c"
    mutant.write_text(gen.replace(needle, "return CLS_AGREE;"))
    real = t27b_fuzz.GENS
    t27b_fuzz.GENS = (real[0], mutant)
    t27b_fuzz.lib.cache_clear()
    os.environ["XDG_CACHE_HOME"] = tmp
    m = judged()
    t27b_fuzz.GENS = real
    t27b_fuzz.lib.cache_clear()
    check(m["disagree"] == 2 and m["classes"]["SEMANTICS_GAP"] == 0,
          f"mutation: an oracle that agrees with a wrong checksum is caught by check 3 (disagree {m['disagree']})")

print(f"\n{fails} failure(s)")
sys.exit(1 if fails else 0)
