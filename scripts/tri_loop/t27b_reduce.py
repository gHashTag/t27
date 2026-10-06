"""`tri t27b reduce`: shrink a spec to a minimal t27b repro (gHashTag/t27#6445, owner-approved exception).

    tri t27b reduce <spec> [--family X | --disagree] [--out PATH] [--t27b BIN] [--t27c BIN]
                    [--reference-cache FILE] [--json]

Delta debugging (ddmin) over the spec's declarations, then its statements,
then its expressions, repeated until a whole cycle removes nothing. Every
candidate must parse under `t27c parse`; then t27b runs it (`t27b corpus
--blockers`), and only a candidate t27b still finds interesting goes to the
reference (`--reference <t27c> --reference-cache <file>`), because the
reference is the slow half. A candidate is kept only when the reference
passes it (the guard: a repro the reference cannot compile is a reference bug,
not a t27b lane -- slip Q38) and, in family mode, t27b's first blocker is still
the family; in disagree mode t27b fails, crashes, mismatches, or a test's
verdict differs from the reference's.

Every decision -- the interestingness answer, the t27b pre-filter, the ddmin
chunk bounds and granularity, the level order, the budget and the goal -- is
specs/tri/t27b/reduce.t27, compiled from `gen/c/tri/t27b/reduce.c` (written by
`t27c gen-c`, never by hand -- L2) and called through ctypes. What stays here is
I/O t27 has no construct for yet: splitting the source into units, running
t27c and t27b, the memo and the fixture's header. Debt #6198.

The fixture lands in the spec's FIXTURES_DIR (outside specs/, so it is not a
second module in the corpus the lab ratchets) with a header naming the origin,
the family and the command that regenerates it; `tri t27b next` reads the
family line back and links the repro under its lane.

The interface is a spec path in and a fixture path out, so a fuzzer (#6442)
can hand its failing seeds straight to it.
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
GEN = ROOT / "gen" / "c" / "tri" / "t27b" / "reduce.c"
STRINGS = ("ANSWER_NAMES", "LEVEL_NAMES", "FIXTURES_DIR", "FAMILY_TAG")
# The lab's verdict strings, in the order of steward.t27's verdict codes.
VERDICTS = ("pass", "pass_vacuous", "blocked", "frontend", "codegen",
            "fail", "mismatch", "crash", "timeout", "not run", "missing", "lab_error")


class RulesUnavailable(RuntimeError):
    """The spec's compiled rules could not be loaded; nothing is decided without them."""


def _cache_dir():
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
    return Path(base) / "t27"


@functools.lru_cache(maxsize=1)
def _lib():
    try:
        src = GEN.read_bytes()
    except OSError as e:
        raise RulesUnavailable(f"{GEN.relative_to(ROOT)}: {e}") from None
    shim = f'#include "{GEN}"\n' + "".join(f"const char *t27_str_{n}(void) {{ return {n}; }}\n" for n in STRINGS)
    key = hashlib.sha256(src + shim.encode()).hexdigest()[:16]
    lib = _cache_dir() / "t27b-rules" / f"reduce-{key}.{'dylib' if sys.platform == 'darwin' else 'so'}"
    if not lib.exists():
        lib.parent.mkdir(parents=True, exist_ok=True)
        fd, c = tempfile.mkstemp(dir=lib.parent, suffix=".c")
        with os.fdopen(fd, "w") as f:
            f.write(shim)
        fd, tmp = tempfile.mkstemp(dir=lib.parent, suffix=lib.suffix)
        os.close(fd)
        cc = os.environ.get("CC", "cc")
        p = subprocess.run([cc, "-shared", "-fPIC", "-O2", "-w", "-o", tmp, c], capture_output=True, text=True)
        os.unlink(c)
        if p.returncode != 0:
            os.unlink(tmp)
            raise RulesUnavailable(f"{cc} could not compile {GEN.relative_to(ROOT)}: {p.stderr.strip()[:300]}")
        os.replace(tmp, lib)
    so = ctypes.CDLL(str(lib))
    u8, u32, b = ctypes.c_uint8, ctypes.c_uint32, ctypes.c_bool
    sigs = {"t27b_side": ([u8, u8, b], b), "answer": ([u8, u8, u32, b, u8, b, u32], u8),
            "start_chunks": ([u32], u32), "chunk_lo": ([u32] * 3, u32), "chunk_hi": ([u32] * 3, u32),
            "after_removal": ([u32] * 2, u32), "after_round": ([u32] * 2, u32),
            "next_level": ([u8, b], u8), "within_budget": ([u32], b), "goal_met": ([u32], b)}
    for name, (args, res) in sigs.items():
        getattr(so, name).argtypes = args
        getattr(so, name).restype = res
    for n in STRINGS:
        getattr(so, f"t27_str_{n}").restype = ctypes.c_char_p
    return so


def text(name):
    return getattr(_lib(), f"t27_str_{name}")().decode()


def verdict(v):
    if v == "skip":
        v = "not run"
    if v not in VERDICTS:
        raise ValueError(f"unknown verdict {v!r}")
    return VERDICTS.index(v)


# --- the fixtures `tri t27b next` links --------------------------------------

def repros(root=ROOT):
    """{family: path relative to root} for every fixture whose header names a family."""
    tag = text("FAMILY_TAG")
    out = {}
    d = Path(root) / text("FIXTURES_DIR")
    for p in sorted(d.glob("*.t27")) if d.is_dir() else ():
        for line in p.read_text(encoding="utf-8", errors="replace").splitlines()[:8]:
            if line.startswith(tag):
                out.setdefault(line[len(tag):].strip(), str(p.relative_to(root)))
    return out


# --- splitting a source into units (I/O t27 cannot do yet) -------------------

def _is_comment(line):
    s = line.strip()
    return not s or s.startswith("//") or s.startswith(";")


def _code(line):
    """The line with string and char literals blanked and a // comment cut, so braces count."""
    if line.lstrip().startswith(";"):
        return ""
    out, i, n = [], 0, len(line)
    while i < n:
        c = line[i]
        if c == "/" and line.startswith("//", i):
            break
        if c == '"':
            j = i + 1
            while j < n and line[j] != '"':
                j += 2 if line[j] == "\\" else 1
            out.append('""')
            i = j + 1
            continue
        if c == "'":
            m = re.match(r"'(\\.|[^'\\])'", line[i:])
            if m:
                out.append("''")
                i += m.end()
                continue
        out.append(c)
        i += 1
    return "".join(out)


def _depths(lines):
    """Per line: (depth at start, depth at end, lowest depth inside the line)."""
    d, res = 0, []
    for line in lines:
        start, low = d, d
        for c in _code(line):
            if c in "{([":
                d += 1
            elif c in "})]":
                d -= 1
                low = min(low, d)
        res.append((start, d, low))
    return res


def _line_spans(src):
    lines = src.split("\n")
    offs, o = [], 0
    for line in lines:
        offs.append(o)
        o += len(line) + 1
    return lines, offs


def stmt_spans(src):
    """Every statement-shaped line span: (depth, first line, last line, is_module)."""
    lines, _ = _line_spans(src)
    dep = _depths(lines)
    spans = []
    for i, line in enumerate(lines):
        if _is_comment(line):
            continue
        start, _, low = dep[i]
        if low < start:
            continue
        if i > 0 and not _ended(lines, dep, i - 1, start):
            continue
        j = i
        while j < len(lines):
            if dep[j][1] == start and _code(lines[j]).rstrip().endswith((";", "}")):
                break
            if dep[j][1] < start:
                j = None
                break
            j += 1
        if j is None or j >= len(lines):
            continue
        spans.append((start, i, j, lines[i].lstrip().startswith("module ") and dep[i][1] > start))
    return spans


def _ended(lines, dep, k, depth):
    """Does the code up to line k end a statement at `depth`, or open the block that holds it?"""
    while k >= 0 and _is_comment(lines[k]):
        k -= 1
    if k < 0:
        return True
    return dep[k][1] == depth and _code(lines[k]).rstrip().endswith((";", "}", "{"))


def units(src, level):
    """The character spans one ddmin level removes, in source order."""
    lines, offs = _line_spans(src)
    span = lambda a, b: (offs[a], offs[b] + len(lines[b]) + 1)  # noqa: E731
    all_spans = stmt_spans(src)
    module_depths = {d + 1 for d, _, _, m in all_spans if m}
    decl_depths = {0} | module_depths
    if level == 0:
        out = [span(a, b) for d, a, b, m in all_spans if d in decl_depths and not m]
        out += [span(i, i) for i, line in enumerate(lines) if _is_comment(line) and line.strip()]
        return sorted(set(out))
    if level == 1:
        by = {}
        for d, a, b, m in all_spans:
            if d not in decl_depths and not m:
                by.setdefault(d, []).append(span(a, b))
        return [sorted(v) for _, v in sorted(by.items())]
    return expr_units(src)


TOKEN = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'|//.*|[A-Za-z_][A-Za-z0-9_]*|\d[\w.]*'
                   r'|::|->|=>|==|!=|<=|>=|&&|\|\||<<|>>|\.\.\.?|[-+*/%&|^!<>=]=?|\S')
BINOPS = {"+", "-", "*", "/", "%", "&", "|", "^", "&&", "||", "==", "!=", "<", ">", "<=", ">=", "<<", ">>"}
KEYWORDS = {"return", "if", "else", "while", "for", "in", "const", "var", "let"}
OPEN, CLOSE = "([{", ")]}"


def _tokens(src):
    """(text, start, end) per token; // comments and ; prose lines are skipped."""
    toks, pos = [], 0
    for line in src.split("\n"):
        if not line.lstrip().startswith(";"):
            for m in TOKEN.finditer(line):
                if m.group().startswith("//"):
                    break
                toks.append((m.group(), pos + m.start(), pos + m.end()))
        pos += len(line) + 1
    return toks


def _operand_end(tok):
    return tok not in KEYWORDS and (tok[-1].isalnum() or tok[-1] in "_\"')]")


def expr_units(src):
    """Token-level units inside every bracket pair: each comma or semicolon separated item, and each
    binary operator with its right operand (or the first operand with its operator)."""
    toks = _tokens(src)
    stack, groups = [], []
    for k, (t, _, _) in enumerate(toks):
        if t in OPEN:
            stack.append(k)
        elif t in CLOSE and stack:
            groups.append((stack.pop(), k))
    out = set()
    for o, c in groups:
        depth, seg, segs = 0, o + 1, []
        for k in range(o + 1, c):
            t = toks[k][0]
            if t in OPEN:
                depth += 1
            elif t in CLOSE:
                depth -= 1
            elif depth == 0 and t in (",", ";"):
                segs.append((seg, k))
                seg = k + 1
        if seg < c:
            segs.append((seg, c))
        for a, b in segs:
            if a >= b:
                continue
            if len(segs) > 1:
                if b < c:
                    out.add((toks[a][1], toks[b][2]))
                elif a - 1 > o:
                    out.add((toks[a - 1][1], toks[b - 1][2]))
            ops, depth = [], 0
            for k in range(a, b):
                t = toks[k][0]
                if t in OPEN:
                    depth += 1
                elif t in CLOSE:
                    depth -= 1
                elif depth == 0 and t in BINOPS and k > a and _operand_end(toks[k - 1][0]):
                    ops.append(k)
            for n, k in enumerate(ops):
                stop = ops[n + 1] if n + 1 < len(ops) else b
                out.add((toks[k][1], toks[stop - 1][2]))
            if ops:
                out.add((toks[a][1], toks[ops[0]][2]))
    return sorted(out)


def apply(src, removed):
    """src without the removed character spans, blank lines dropped, trailing spaces cut."""
    keep = bytearray(b"\x01") * len(src)
    for a, b in removed:
        keep[a:b] = b"\x00" * (min(b, len(src)) - a)
    s = "".join(ch for ch, k in zip(src, keep) if k)
    lines = [line.rstrip() for line in s.split("\n")]
    return "\n".join(line for line in lines if line.strip()) + "\n"


# --- the oracle ---------------------------------------------------------------

def _use_closure(path, specs_root):
    """The spec's `use` imports, transitively (cli/t27b/src/blockers.rs use_closure)."""
    seen, todo = [], [path]
    while todo:
        p = todo.pop()
        if p in seen:
            continue
        seen.append(p)
        try:
            src = p.read_text(encoding="utf-8")
        except OSError:
            continue
        for line in src.splitlines():
            s = line.strip()
            if not s.startswith("use "):
                continue
            expr = s[4:].split("//")[0].strip().rstrip(";").strip()
            segs = [x for part in expr.split("::") for x in part.split(".")]
            if not segs or any(not x for x in segs):
                continue
            t = specs_root.joinpath(*segs).with_suffix(".t27")
            if t.is_file():
                todo.append(t)
    return seen[1:]


class Oracle:
    def __init__(self, a, rel, mode, family):
        self.a, self.rel, self.mode, self.family = a, rel, mode, family
        self.need_tests = False
        self.memo = {}
        self.n = {"tried": 0, "parse_rejected": 0, "t27b_rejected": 0, "reference_runs": 0,
                  "reference_rejected": 0, "kept": 0, "budget_hit": False}
        slug = re.sub(r"[^A-Za-z0-9]+", "_", rel).strip("_")
        self.work = Path(a.work or (_cache_dir() / "t27b-reduce" / "work" / slug))
        self.specs = self.work / "specs"
        self.file = self.specs / rel
        self.last_row = None
        self.ref_tests_field = True

    def setup(self, orig_path):
        if self.specs.exists():
            shutil.rmtree(self.specs)
        self.file.parent.mkdir(parents=True, exist_ok=True)
        root = ROOT / "specs"
        for dep in _use_closure(orig_path, root):
            to = self.specs / dep.relative_to(root)
            to.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(dep, to)

    def _corpus(self, reference):
        out = self.work / ("ref.json" if reference else "t27b.json")
        argv = [self.a.t27b, "corpus", "specs", "--blockers", "--jobs", "1", "--json", str(out)]
        if reference:
            argv += ["--reference", self.a.t27c, "--reference-cache", self.a.reference_cache,
                     "--reference-timeout-ms", str(self.a.reference_timeout_ms)]
        try:
            out.unlink()
        except OSError:
            pass
        subprocess.run(argv, cwd=self.work, capture_output=True, text=True, timeout=900)
        try:
            doc = json.loads(out.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return None
        want = "specs/" + self.rel
        return next((r for r in doc.get("results") or [] if r.get("file") == want), None)

    def judge(self, src, count=True):
        """(interesting, answer name, row). The decision is reduce.t27's answer()."""
        L = _lib()
        names = text("ANSWER_NAMES").split(",")
        h = hashlib.sha256(src.encode()).hexdigest()
        if h in self.memo:
            return self.memo[h]
        if count:
            if not L.within_budget(self.n["tried"]):
                self.n["budget_hit"] = True
                return (False, "BUDGET", None)
            self.n["tried"] += 1
        self.file.write_text(src, encoding="utf-8")
        p = subprocess.run([self.a.t27c, "parse", str(self.file)], capture_output=True, text=True, timeout=120)
        if p.returncode != 0:
            self.n["parse_rejected"] += 1
            r = (False, "PARSE", None)
            self.memo[h] = r
            return r
        row = self._corpus(False)
        if row is None:
            r = (False, "NO-ROW", None)
            self.memo[h] = r
            return r
        if self.mode == 0 and self.family is None:
            self.family = (row.get("blockers") or [""])[0]
        first = (row.get("blockers") or [""])[0] == self.family
        if not L.t27b_side(self.mode, verdict(row.get("t27b") or "missing"), first):
            self.n["t27b_rejected"] += 1
            r = (False, "T27B-SIDE", row)
            self.memo[h] = r
            return r
        self.n["reference_runs"] += 1
        row = self._corpus(True)
        if row is None:
            r = (False, "NO-ROW", None)
            self.memo[h] = r
            return r
        if "reference_tests" not in row and row.get("reference") == "pass":
            self.ref_tests_field = False
        ref_tests = len(row.get("reference_tests") or {})
        first = (row.get("blockers") or [""])[0] == self.family
        code = L.answer(self.mode, verdict(row.get("reference") or "missing"), ref_tests,
                        self.need_tests and self.ref_tests_field, verdict(row.get("t27b") or "missing"), first,
                        len(row.get("reference_disagree") or []))
        name = names[code] if code < len(names) else str(code)
        if name in ("REFERENCE-NOT-PASS", "REFERENCE-RAN-NO-TEST"):
            self.n["reference_rejected"] += 1
        if code == 0:
            self.n["kept"] += 1
        r = (code == 0, name, row)
        self.memo[h] = r
        return r


def ddmin(base, spans, oracle, log):
    """reduce.t27's ddmin over `spans` of `base`; returns the reduced source."""
    L = _lib()
    cur = list(spans)
    n = L.start_chunks(len(cur))
    best = base
    while n:
        removed = False
        for i in range(n):
            lo, hi = L.chunk_lo(len(cur), n, i), L.chunk_hi(len(cur), n, i)
            if lo >= hi:
                continue
            cand_keep = cur[:lo] + cur[hi:]
            kept = set(cand_keep)
            gone = [s for s in spans if s not in kept]
            cand = apply(base, gone)
            if cand == best:
                continue
            ok, _, _ = oracle.judge(cand)
            if oracle.n["budget_hit"]:
                return best
            if ok:
                cur, best, removed = cand_keep, cand, True
                log(f"    {len(best.splitlines()):>4} lines  (removed {hi - lo} of {len(cur) + hi - lo} units)")
                n = L.after_removal(n, len(cur))
                break
        if not removed:
            n = L.after_round(n, len(cur))
    return best


def reduce_src(src, oracle, log):
    """Comments first (one candidate), then reduce.t27's level order until a cycle removes nothing."""
    L = _lib()
    level_names = text("LEVEL_NAMES").split(",")
    cur = apply(src, [])
    bare = apply("\n".join(x for x in cur.split("\n") if not _is_comment(x)), [])
    if bare != cur and oracle.judge(bare)[0]:
        cur = bare
        log(f"  comments: {len(cur.splitlines())} lines")
    level, progressed = 0, False
    while level != 3 and not oracle.n["budget_hit"]:
        before = cur
        cur = _stmt_pass(cur, oracle, log) if level == 1 else ddmin(cur, units(cur, level), oracle, log)
        log(f"  {level_names[level]}: {len(cur.splitlines())} lines, {oracle.n['tried']} candidates")
        progressed = progressed or cur != before
        nxt = L.next_level(level, progressed)
        if nxt < level:
            progressed = False
        level = nxt
    return cur


def _stmt_pass(cur, oracle, log):
    """Statements, shallowest depth first; the spans are recomputed after each depth."""
    depth_i = 0
    while not oracle.n["budget_hit"]:
        groups = units(cur, 1)
        if depth_i >= len(groups):
            break
        cur = ddmin(cur, groups[depth_i], oracle, log)
        depth_i += 1
    return cur


# --- the command --------------------------------------------------------------

def _default_bin(env, name):
    v = os.environ.get(env)
    if v:
        return v
    for p in (ROOT / "target" / "release" / name, ROOT / "target" / "debug" / name):
        if os.access(p, os.X_OK):
            return str(p)
    return shutil.which(name) or name


def main(argv):
    ap = argparse.ArgumentParser(prog="tri t27b reduce",
                                 description="shrink a spec to a minimal t27b repro by ddmin, reference-guarded (#6445)")
    ap.add_argument("spec")
    g = ap.add_mutually_exclusive_group()
    g.add_argument("--family", help="keep t27b's first blocker equal to this (default: the spec's own first blocker)")
    g.add_argument("--disagree", action="store_true", help="keep t27b disagreeing with a passing reference")
    ap.add_argument("--out", help="fixture path (default: <FIXTURES_DIR>/<family>.t27)")
    ap.add_argument("--t27b", default=_default_bin("T27B", "t27b"))
    ap.add_argument("--t27c", default=_default_bin("TRI_T27C", "t27c"))
    ap.add_argument("--reference-cache", default=str(_cache_dir() / "t27b-reduce" / "refcache.tsv"))
    ap.add_argument("--reference-timeout-ms", type=int, default=120000)
    ap.add_argument("--work", help="scratch directory (default: under ~/.cache/t27/t27b-reduce/work)")
    ap.add_argument("--json", action="store_true")
    try:
        a = ap.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    log = (lambda s: print(s, file=sys.stderr, flush=True)) if a.json else (lambda s: print(s, flush=True))
    spec = Path(a.spec).resolve()
    try:
        rel = str(spec.relative_to(ROOT / "specs"))
    except ValueError:
        rel = spec.name
    try:
        src = spec.read_text(encoding="utf-8")
    except OSError as e:
        print(f"tri t27b reduce: cannot read {a.spec}: {e}")
        return 2
    try:
        _lib()
    except RulesUnavailable as e:
        print(f"tri t27b reduce: UNREADABLE {e}")
        return 2
    Path(a.reference_cache).parent.mkdir(parents=True, exist_ok=True)
    mode = 1 if a.disagree else 0
    o = Oracle(a, rel, mode, a.family)
    o.setup(spec)
    t0 = time.time()
    ok, why, row = o.judge(src, count=False)
    shown = os.path.relpath(spec, ROOT) if str(spec).startswith(str(ROOT)) else str(spec)
    if not ok:
        res = {"spec": shown, "interesting": False, "answer": why, "family": o.family,
               "reference": row and row.get("reference"), "t27b": row and row.get("t27b"),
               "reference_detail": row and (row.get("reference_detail") or "")[:200]}
        if a.json:
            print(json.dumps(res, indent=1))
        else:
            print(f"tri t27b reduce: {shown} is not interesting: {why}")
            print(f"  reference {res['reference']}, t27b {res['t27b']}, first blocker "
                  f"{((row or {}).get('blockers') or ['-'])[0]}")
            if why == "REFERENCE-NOT-PASS":
                print("  the reference does not pass it: a reference bug to file, not a t27b lane (Q38)")
        return 1
    orig_ref_tests = len(row.get("reference_tests") or {})
    o.need_tests = orig_ref_tests > 0
    if not o.ref_tests_field:
        log("  note: this t27b reports no reference_tests; the guard cannot require that the reference ran a test")
    log(f"tri t27b reduce: {shown} ({len(src.splitlines())} lines), "
        + (f"family {o.family}" if mode == 0 else f"disagree (t27b {row.get('t27b')})")
        + f"; reference pass, {orig_ref_tests} test(s)")
    red = reduce_src(src, o, log)
    final_row = o.judge(red, count=False)[2]
    tag = text("FAMILY_TAG")
    sha = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short=9", "HEAD"],
                         capture_output=True, text=True).stdout.strip() or "?"
    what = o.family if mode == 0 else f"disagree (t27b {final_row.get('t27b')})"
    cmd = f'tri t27b reduce {shown} ' + (f'--family "{o.family}"' if mode == 0 else "--disagree")
    header = [f"{tag}{what}",
              f"// origin: {shown} ({len(src.splitlines())} lines) at {sha}; reduced by tri t27b reduce (#6445)",
              f"// reference (t27c test-report): {final_row.get('reference')}, "
              f"{len(final_row.get('reference_tests') or {})} test(s); t27b: {final_row.get('t27b')}",
              f"// regenerate: {cmd}"]
    fixture = "\n".join(header) + "\n" + red
    ok2, why2, _ = o.judge(fixture, count=False)
    if a.out:
        out = Path(a.out)
    else:
        name = re.sub(r"[^a-z0-9]+", "_", (o.family if mode == 0 else f"disagree {Path(rel).stem}").lower()).strip("_")
        out = ROOT / text("FIXTURES_DIR") / f"{name}.t27"
    lines = len(fixture.splitlines())
    res = {"spec": shown, "interesting": True, "family": o.family if mode == 0 else None,
           "mode": "family" if mode == 0 else "disagree", "lines_before": len(src.splitlines()),
           "lines_after": lines, "goal_met": bool(_lib().goal_met(lines)), "fixture": None,
           "fixture_recheck": why2, "seconds": round(time.time() - t0, 1), **o.n}
    if not ok2:
        log(f"tri t27b reduce: the fixture with its header is not interesting ({why2}); nothing written")
        if a.json:
            print(json.dumps(res, indent=1))
        return 1
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(fixture, encoding="utf-8")
    res["fixture"] = os.path.relpath(out, ROOT) if str(out.resolve()).startswith(str(ROOT)) else str(out)
    if a.json:
        print(json.dumps(res, indent=1))
    else:
        print(f"lines {res['lines_before']} -> {lines} ({'under' if res['goal_met'] else 'NOT under'} the goal)")
        print(f"candidates {o.n['tried']}: parse-rejected {o.n['parse_rejected']}, t27b-rejected "
              f"{o.n['t27b_rejected']}, reference runs {o.n['reference_runs']}, reference-rejected "
              f"{o.n['reference_rejected']}, kept {o.n['kept']}"
              + ("; candidate budget hit" if o.n["budget_hit"] else ""))
        print(f"fixture {res['fixture']}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
