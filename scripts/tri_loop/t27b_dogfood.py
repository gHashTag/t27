"""The dogfood rule, loaded from its t27 spec (#6457, owner-approved exception).

`specs/tri/t27b/dogfood.t27` decides which specs are our own, what one lab
record of such a spec means (PASS, VACUOUS, a t27b lane, a t27c issue,
unjudged), the words a report prints, and the tie-break `tri t27b next` uses.
This module holds no decision of its own: it compiles
`gen/c/tri/t27b/dogfood.c` (written by `t27c gen-c` from that spec, never by
hand -- L2) into a cache and calls it through ctypes.

What stays here is plumbing t27 cannot do yet (it has no I/O): reading the
spec's string constants, matching a path against the prefixes, and finding the
specs whose gen-c output a loader in scripts/tri_loop/ reads. The debt is
#6198: when t27 can read files, this moves into the spec.
"""

import ctypes
import functools
import hashlib
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "gen" / "c" / "tri" / "t27b" / "dogfood.c"
STRINGS = ("OWN_PREFIXES", "LOADER_DIR", "ROW_NAMES", "GROUP_T27B_LANE", "GROUP_T27C_ISSUE")
# The lab's verdict strings, in the order of steward.t27's verdict codes.
VERDICTS = ("pass", "pass_vacuous", "blocked", "frontend", "codegen",
            "fail", "mismatch", "crash", "timeout", "not run", "missing", "lab_error")


class RulesUnavailable(RuntimeError):
    """The spec's compiled rules could not be loaded; nothing is decided without them."""


def _cache_dir():
    base = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
    return Path(base) / "t27" / "t27b-rules"


@functools.lru_cache(maxsize=1)
def _lib():
    try:
        src = GEN.read_bytes()
    except OSError as e:
        raise RulesUnavailable(f"{GEN.relative_to(ROOT)}: {e}") from None
    # A one-line accessor per string constant: the spec's #defines have no symbol of their own.
    shim = f'#include "{GEN}"\n' + "".join(f"const char *t27_str_{n}(void) {{ return {n}; }}\n" for n in STRINGS)
    key = hashlib.sha256(src + shim.encode()).hexdigest()[:16]
    lib = _cache_dir() / f"dogfood-{key}.{'dylib' if sys.platform == 'darwin' else 'so'}"
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
    so.own_spec.argtypes = [ctypes.c_bool] * 2
    so.own_spec.restype = ctypes.c_bool
    so.dogfood_row.argtypes = [ctypes.c_bool, ctypes.c_uint8, ctypes.c_uint8]
    so.dogfood_row.restype = ctypes.c_uint8
    so.is_work.argtypes = [ctypes.c_uint8]
    so.is_work.restype = ctypes.c_bool
    so.lane_before.argtypes = [ctypes.c_uint32] * 4
    so.lane_before.restype = ctypes.c_bool
    for n in STRINGS:
        getattr(so, f"t27_str_{n}").restype = ctypes.c_char_p
    return so


def text(name):
    """One of the spec's string constants."""
    return getattr(_lib(), f"t27_str_{name}")().decode()


def verdict(v):
    if v not in VERDICTS:
        raise ValueError(f"unknown lab verdict {v!r}")
    return VERDICTS.index(v)


@functools.lru_cache(maxsize=1)
def fed_specs():
    """Specs whose gen-c output a loader under the spec's LOADER_DIR names, and which exist."""
    fed = set()
    for py in sorted((ROOT / text("LOADER_DIR")).glob("*.py")):
        t = py.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(r'"gen"\s*/\s*"c"((?:\s*/\s*"[^"]+")+)', t):
            fed.add("specs/" + "/".join(re.findall(r'"([^"]+)"', m.group(1)))[:-2] + ".t27")
        for m in re.finditer(r"gen/c/([\w/.-]+)\.c\b", t):
            fed.add("specs/" + m.group(1) + ".t27")
    return tuple(sorted(f for f in fed if (ROOT / f).exists()))


def own(path):
    prefixes = [p for p in text("OWN_PREFIXES").split(",") if p]
    return _lib().own_spec(any(path.startswith(p) for p in prefixes), path in fed_specs())


def row(rec):
    """The row name of one lab record ("-" when the spec is not our own)."""
    names = text("ROW_NAMES").split(",")
    return names[_lib().dogfood_row(own(rec.get("file") or ""), verdict(rec.get("reference") or "missing"),
                                    verdict(rec.get("t27b") or "missing"))]


def is_work(name):
    return _lib().is_work(text("ROW_NAMES").split(",").index(name))


def lane_before(score_a, own_a, score_b, own_b):
    return _lib().lane_before(score_a, own_a, score_b, own_b)
