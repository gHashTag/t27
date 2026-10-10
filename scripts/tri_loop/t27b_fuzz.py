"""tri t27b fuzz -- the source-level differential fuzzer (#6442, owner-approved exception).

The program generator is `specs/tri/t27b/fuzz.t27` and the judge is
`specs/tri/t27b/fuzz_oracle.t27`. This module makes no decision of its own:
it compiles their `t27c gen-c` output (gen/c/tri/t27b/fuzz.c and
fuzz_oracle.c, never hand-edited -- L2) into a cache and calls them through
ctypes.

What stays here is plumbing t27 cannot do yet (it has no I/O): writing the
rendered cases to files, running `t27b corpus --reference <t27c>` over them,
reading its JSON, and turning a t27b file label into the oracle's verdict code.

  tri t27b fuzz --cases N --seed S [--first C] --t27b <bin> --reference <t27c>
                [--runner "<cmd> [args]"] [--jobs J] [--dir D] [--json out]
                [--save-findings DIR]
  tri t27b fuzz --from-corpus corpus.json --seed S [--json out]

A case is reproduced by `--seed S --first C --cases 1`. Exit 0 when nothing
disagrees, 1 when a test disagrees, 2 when the run could not be made.
"""

import argparse
import ctypes
import functools
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GENS = (ROOT / "gen" / "c" / "tri" / "t27b" / "fuzz.c", ROOT / "gen" / "c" / "tri" / "t27b" / "fuzz_oracle.c")
STRINGS = ("CLS_NAMES", "PUBLISHED_KEYS", "HOST_FAILURES")
OUT_CAP = 16384
TEST_RE = re.compile(r"^test\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", re.M)
# t27b file labels that mean "never ran the tests", and the oracle's code for each.
T27B_REJECT = ("blocked", "frontend", "codegen")
T27B_BROKEN = ("crash", "mismatch")
FINDINGS_CAP = 200


class Unavailable(RuntimeError):
    """The spec's compiled code could not be loaded; nothing is decided without it."""


class Out(ctypes.Structure):
    _fields_ = [("buf", ctypes.c_uint8 * OUT_CAP), ("len", ctypes.c_uint32), ("over", ctypes.c_bool)]


def _cache_dir():
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
    return Path(base) / "t27" / "t27b-rules"


@functools.lru_cache(maxsize=1)
def lib():
    gens = GENS
    try:
        srcs = [g.read_bytes() for g in gens]
    except OSError as e:
        raise Unavailable(str(e)) from None
    shim = "".join(f'#include "{g}"\n' for g in gens)
    shim += "".join(f"const char *t27_str_{n}(void) {{ return {n}; }}\n" for n in STRINGS)
    key = hashlib.sha256(b"".join(srcs) + shim.encode()).hexdigest()[:16]
    so_path = _cache_dir() / f"fuzz-{key}.{'dylib' if sys.platform == 'darwin' else 'so'}"
    if not so_path.exists():
        so_path.parent.mkdir(parents=True, exist_ok=True)
        fd, c = tempfile.mkstemp(dir=so_path.parent, suffix=".c")
        with os.fdopen(fd, "w") as f:
            f.write(shim)
        fd, tmp = tempfile.mkstemp(dir=so_path.parent, suffix=so_path.suffix)
        os.close(fd)
        cc = os.environ.get("CC", "cc")
        p = subprocess.run([cc, "-shared", "-fPIC", "-O2", "-w", "-o", tmp, c], capture_output=True, text=True)
        os.unlink(c)
        if p.returncode != 0:
            os.unlink(tmp)
            raise Unavailable(f"{cc} could not compile the fuzz specs' gen-c output: {p.stderr.strip()[:300]}")
        os.replace(tmp, so_path)
    so = ctypes.CDLL(str(so_path))
    so.fuzz_render.argtypes = [ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(Out)]
    so.fuzz_render.restype = ctypes.c_bool
    so.classify.argtypes = [ctypes.c_uint8] * 3
    so.classify.restype = ctypes.c_uint8
    for f in ("is_disagree", "is_rejected", "is_finding"):
        getattr(so, f).argtypes = [ctypes.c_uint8]
        getattr(so, f).restype = ctypes.c_bool
    so.expect_of.argtypes = [ctypes.c_char_p, ctypes.c_uint32]
    so.expect_of.restype = ctypes.c_uint8
    for n in STRINGS:
        getattr(so, f"t27_str_{n}").restype = ctypes.c_char_p
    return so


def text(name):
    return getattr(lib(), f"t27_str_{name}")().decode()


def render(seed, case_no):
    """The source of one case, or None when the generator refuses it (too long, inconsistent)."""
    o = Out()
    ok = lib().fuzz_render(seed, case_no, ctypes.byref(o))
    return bytes(o.buf[: o.len]).decode("ascii") if ok else None


def case_file(case_no):
    return f"c{case_no:06d}.t27"


def side_verdict(verdicts, name, label, broken, reject, timeout, detail=""):
    """One side's verdict code (fuzz_oracle.t27 V_*) for one test."""
    # V_PASS 0, V_FAIL 1, V_REJECT 2, V_TIMEOUT 3, V_BROKEN 4, V_MISSING 5, V_HOST 6.
    if detail and any(m in detail for m in text("HOST_FAILURES").split(",")):
        return 6
    if label in broken:
        return 4
    if isinstance(verdicts, dict) and name in verdicts:
        return 0 if verdicts[name] else 1
    if label in reject:
        return 2
    if label in timeout:
        return 3
    return 5


def judge(corpus, sources, seed):
    """Classify every generated test of a `t27b corpus --reference` JSON.

    sources maps a case file's base name to (case number, source text)."""
    so = lib()
    names = text("CLS_NAMES").split(",")
    classes = {n: 0 for n in names}
    findings, seeds = [], []
    tests = cases = 0
    for row in corpus.get("results", []):
        base = os.path.basename(row["file"])
        if base not in sources:
            continue
        case_no, src = sources[base]
        cases += 1
        worst = []
        for name in TEST_RE.findall(src):
            e = so.expect_of(name.encode(), len(name))
            t = side_verdict(row.get("test_verdicts"), name, row.get("t27b"), T27B_BROKEN, T27B_REJECT, ("timeout",),
                             row.get("detail", ""))
            r = side_verdict(row.get("reference_tests"), name, row.get("reference"), (), ("blocked",), ("timeout",),
                             row.get("reference_detail", ""))
            c = so.classify(e, t, r)
            tests += 1
            classes[names[c]] += 1
            if so.is_finding(c):
                worst.append(names[c])
                if len(findings) < FINDINGS_CAP:
                    findings.append({"seed": seed, "case": case_no, "file": base, "test": name, "class": names[c],
                                     "t27b": row.get("t27b"), "t27b_detail": row.get("detail", ""),
                                     "reference": row.get("reference"),
                                     "reference_detail": row.get("reference_detail", "")})
        if worst:
            seeds.append(f"{seed}:{case_no}")
    count = lambda pred: sum(v for k, v in classes.items() if pred(names.index(k)))
    return {
        "seed": seed,
        "cases": cases,
        "tests": tests,
        "agree": classes["AGREE"],
        "disagree": count(so.is_disagree),
        "rejected": count(so.is_rejected),
        "unjudged": classes["UNJUDGED"],
        "seeds": seeds,
        "classes": classes,
        "findings": findings,
    }


def generate(seed, first, n, d):
    d.mkdir(parents=True, exist_ok=True)
    sources, refused = {}, []
    for k in range(first, first + n):
        src = render(seed, k)
        if src is None:
            refused.append(k)
            continue
        (d / case_file(k)).write_text(src)
        sources[case_file(k)] = (k, src)
    return sources, refused


def run_corpus(args, d, out):
    runner = args.runner.split() if args.runner else []
    cmd = runner + [args.t27b, "corpus", str(d), "--json", str(out), "--blockers", "--reference", args.reference,
                    "--jobs", str(args.jobs), "--timeout-ms", str(args.timeout_ms)]
    if args.runner:
        cmd += ["--runner", args.runner]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if not out.exists():
        raise Unavailable(f"t27b corpus exited {p.returncode} and wrote no JSON: {(p.stderr or p.stdout)[-400:]}")
    return json.loads(out.read_text())


def save_findings(summary, sources, dest):
    """Write the cases behind the findings into dest, with findings.json beside them."""
    dest.mkdir(parents=True, exist_ok=True)
    index_path = dest / "findings.json"
    index = json.loads(index_path.read_text()) if index_path.exists() else []
    have = {(f["seed"], f["case"], f["test"]) for f in index}
    for f in summary["findings"]:
        name = f"s{f['seed']}-{case_file(f['case'])}"
        (dest / name).write_text(sources[f["file"]][1])
        if (f["seed"], f["case"], f["test"]) not in have:
            index.append(dict(f, file=name))
    index.sort(key=lambda f: (f["seed"], f["case"], f["test"]))
    index_path.write_text(json.dumps(index, indent=1) + "\n")


def main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b fuzz", description=__doc__.split("\n")[0])
    ap.add_argument("--cases", type=int, default=1000)
    ap.add_argument("--seed", type=int, default=None, help="default: a fresh one (the current Unix time)")
    ap.add_argument("--first", type=int, default=0)
    ap.add_argument("--t27b", default=str(ROOT / "target" / "release" / "t27b"))
    ap.add_argument("--reference", default=str(ROOT / "target" / "release" / "t27c"))
    ap.add_argument("--runner", default="", help='how to start t27b, e.g. "qemu-aarch64 -L /usr/aarch64-linux-gnu"')
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--timeout-ms", type=int, default=20000)
    ap.add_argument("--dir", help="where the cases go (default: a temporary directory, removed after)")
    ap.add_argument("--from-corpus", help="judge this corpus JSON of cases already generated with --seed")
    ap.add_argument("--json", help="write the summary here")
    ap.add_argument("--save-findings", help="copy the cases behind every finding here, with findings.json")
    try:
        args = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    seed = args.seed if args.seed is not None else int(time.time())
    t0 = time.time()
    tmp = None
    try:
        d = Path(args.dir) if args.dir else Path(tmp := tempfile.mkdtemp(prefix="t27b-fuzz-"))
        if args.from_corpus:
            corpus = json.loads(Path(args.from_corpus).read_text())
            sources = {}
            for row in corpus.get("results", []):
                m = re.search(r"c(\d+)\.t27$", row["file"])
                if m:
                    k = int(m.group(1))
                    src = render(seed, k)
                    if src is not None:
                        sources[case_file(k)] = (k, src)
            refused = []
        else:
            sources, refused = generate(seed, args.first, args.cases, d / "cases")
            corpus = run_corpus(args, d / "cases", d / "corpus.json")
        summary = judge(corpus, sources, seed)
    except (Unavailable, OSError, ValueError) as e:
        print(f"tri t27b fuzz: {e}", file=sys.stderr)
        return 2
    finally:
        if tmp:
            shutil.rmtree(tmp, ignore_errors=True)
    summary.update({"first": args.first, "refused": refused, "seconds": round(time.time() - t0, 1),
                    "generator": "specs/tri/t27b/fuzz.t27", "oracle": "specs/tri/t27b/fuzz_oracle.t27"})
    if args.save_findings:
        save_findings(summary, sources, Path(args.save_findings))
    if args.json:
        Path(args.json).write_text(json.dumps(summary, indent=1) + "\n")
    print(f"tri t27b fuzz: seed {seed} cases {summary['cases']} tests {summary['tests']}: agree {summary['agree']}"
          f" disagree {summary['disagree']} rejected {summary['rejected']} unjudged {summary['unjudged']}"
          f" ({summary['seconds']} s)")
    for k, v in summary["classes"].items():
        if v and k != "AGREE":
            print(f"  {k:<20} {v}")
    for f in summary["findings"][:20]:
        print(f"  {f['class']:<20} --seed {f['seed']} --first {f['case']} --cases 1  {f['test']}:"
              f" t27b {f['t27b']} {f['t27b_detail'][:80]} | reference {f['reference']} {f['reference_detail'][:80]}")
    return 1 if summary["disagree"] else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
