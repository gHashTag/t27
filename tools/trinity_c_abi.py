#!/usr/bin/env python3
"""The C ABI of libtrinity-vsa: the spec's function table held to the header and to the source
exports of gHashTag/trinity, and the ABI fixture compiled against the header.

WHY THIS EXISTS
---------------
S05 of gHashTag/trinity#988 (gHashTag/t27#3567): the consumer's host boundary is src/c_api.zig,
described by a hand-written header and by a prose contract; at the pinned revision the source
does not parse, so nothing has ever crossed the boundary under a test. specs/api/c_abi.t27
states the twenty-two exported functions with their prototypes, ownership and NULL rules. This
tool proves the cheap half of that claim -- that the table is the header's and the source's
(names, arity, prototypes), and that the fixture uses the header correctly -- and records the
expensive half honestly: the fixture is not linked, the library is not built, `zig ast-check`
(when a zig is given) says why.

Usage:
  python3 tools/trinity_c_abi.py check --trinity-root <clone at PINNED_REVISION> [--zig PATH] [--report conformance/trinity/c_abi.json]
  python3 tools/trinity_c_abi.py --self-check

Exit codes: 0 no finding; 1 findings; 2 could not run (no spec, no clone, no cc).
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = ROOT / "specs/api/c_abi.t27"
FIXTURE = ROOT / "conformance/trinity/abi/abi_fixture.c"
REPORT = ROOT / "conformance/trinity/c_abi.json"
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
INT_TYPES = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"}
REQUIRED = ["KIND", "PINNED_REVISION", "OWNER_MODULE", "HEADER", "VERSION_STRING", "MAX_DIM", "FUNCTION_COUNT",
            "FUNCTIONS", "FUNCTION_ARITY", "FUNCTION_C", "FUNCTION_OWNERSHIP", "FUNCTION_ON_NULL", "FINDINGS"]
PROTO_RE = re.compile(r"^\s*((?:const\s+)?[A-Za-z_]\w*(?:\s*\*)*)\s+\*?(trinity_vsa_\w+)\s*\(([^)]*)\)\s*;")
EXPORT_RE = re.compile(r"^export fn (trinity_vsa_\w+)\(([^)]*)\)")


def sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd=None, timeout: int = 300) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except (OSError, subprocess.TimeoutExpired) as e:
        return 127, "", str(e)


def parse_value(raw: str, typ: str):
    raw = raw.strip()
    if typ == "bool":
        return raw == "true"
    if typ in INT_TYPES:
        return int(raw)
    if typ == "str":
        return json.loads(raw)
    m = re.fullmatch(r"\[(\d+)\](\w+)", typ)
    if m:
        items = json.loads("[" + raw[1:-1] + "]") if m.group(2) == "str" else [int(x) for x in raw[1:-1].split(",") if x.strip()]
        if len(items) != int(m.group(1)):
            raise ValueError(f"annotated {typ}, holds {len(items)}")
        return items
    if typ == "f64":
        return float(raw)
    raise ValueError(f"unknown type {typ}")


def load_spec(path: pathlib.Path = SPEC) -> dict:
    text = path.read_text(encoding="utf-8")
    if re.search(r"[^\x00-\x7f]", text):
        raise ValueError(f"{path}: non-ASCII byte (L3)")
    f = {}
    for n, line in enumerate(text.split("\n"), 1):
        m = CONST_RE.match(line)
        if m:
            f[m.group(1)] = parse_value(m.group(3), m.group(2).strip())
        elif line.startswith("pub const"):
            raise ValueError(f"{path}:{n}: cannot read constant line")
    missing = [k for k in REQUIRED if k not in f]
    if missing:
        raise ValueError(f"{path}: missing {', '.join(missing)}")
    n = len(f["FUNCTIONS"])
    for k in ("FUNCTION_ARITY", "FUNCTION_C", "FUNCTION_OWNERSHIP", "FUNCTION_ON_NULL"):
        if len(f[k]) != n:
            raise ValueError(f"{path}: {k} must have one entry per function")
    if f["FUNCTION_COUNT"] != n:
        raise ValueError(f"{path}: FUNCTION_COUNT {f['FUNCTION_COUNT']} but {n} functions")
    return f


def normalize_proto(s: str) -> str:
    s = re.sub(r"\s+", " ", s.strip().rstrip(";").strip())
    s = re.sub(r"\s*\*\s*", "* ", s)
    s = re.sub(r"\s*\(\s*", "(", s)
    s = re.sub(r"\s*\)\s*", ")", s)
    s = re.sub(r"\s*,\s*", ", ", s)
    return s.strip()


def param_count(params: str) -> int:
    params = params.strip()
    if not params or params == "void":
        return 0
    return len([p for p in params.split(",") if p.strip()])


def parse_header(text: str) -> dict[str, dict]:
    out = {}
    for n, line in enumerate(text.split("\n"), 1):
        m = PROTO_RE.match(line)
        if m:
            out[m.group(2)] = {"line": n, "prototype": normalize_proto(line), "arity": param_count(m.group(3))}
    return out


def parse_exports(text: str) -> dict[str, dict]:
    out = {}
    for n, line in enumerate(text.split("\n"), 1):
        m = EXPORT_RE.match(line)
        if m:
            out[m.group(1)] = {"line": n, "arity": param_count(m.group(2))}
    return out


def cc_syntax_check(fixture: pathlib.Path, include_dir: pathlib.Path) -> dict:
    cmd = ["cc", "-std=c11", "-Wall", "-Wextra", "-fsyntax-only", "-I", str(include_dir), str(fixture)]
    rc, out, err = run(cmd)
    return {"command": "cc -std=c11 -Wall -Wextra -fsyntax-only -I <include> " + str(fixture.relative_to(ROOT)) if fixture.is_relative_to(ROOT) else " ".join(cmd),
            "rc": rc, "ok": rc == 0, "diagnostics": (err or out).strip()[:800]}


def ast_check(zig: str | None, source: pathlib.Path) -> dict | None:
    if not zig:
        return None
    rc, out, err = run([zig, "version"], timeout=60)
    version = (out or err).strip().split("\n")[0][:60] if rc == 0 else None
    rc, out, err = run([zig, "ast-check", str(source)], timeout=120)
    first = next((l for l in (err or out).split("\n") if "error:" in l), "").strip()
    return {"zig": version or zig, "command": f"zig ast-check {source.name}", "rc": rc, "ok": rc == 0, "first_error": first[:300]}


def build_report(f: dict, trinity: pathlib.Path, zig: str | None) -> tuple[dict, list[str]]:
    findings = []
    rc, out, _ = run(["git", "-C", str(trinity), "rev-parse", "HEAD"], timeout=60)
    head = out.strip() if rc == 0 else ""
    if head != f["PINNED_REVISION"]:
        findings.append(f"the clone is at {head[:12] or '?'}, the spec pins {f['PINNED_REVISION'][:12]}")
    header_path = trinity / f["HEADER"]
    source_path = trinity / f["OWNER_MODULE"]
    if not header_path.exists():
        findings.append(f"header missing: {f['HEADER']}")
        header = {}
    else:
        header = parse_header(header_path.read_text(encoding="utf-8", errors="replace"))
    if not source_path.exists():
        findings.append(f"source missing: {f['OWNER_MODULE']}")
        exports = {}
    else:
        exports = parse_exports(source_path.read_text(encoding="utf-8", errors="replace"))
    spec_names = list(f["FUNCTIONS"])
    for name in spec_names:
        if header and name not in header:
            findings.append(f"{name}: in the spec, not in the header")
        if exports and name not in exports:
            findings.append(f"{name}: in the spec, not exported by the source")
    for name in header:
        if name not in spec_names:
            findings.append(f"{name}: in the header, not in the spec")
    for name in exports:
        if name not in spec_names:
            findings.append(f"{name}: exported by the source, not in the spec")
    arity_mismatches, proto_mismatches = [], []
    for name, arity, proto in zip(spec_names, f["FUNCTION_ARITY"], f["FUNCTION_C"]):
        if name in header and header[name]["arity"] != arity:
            arity_mismatches.append({"function": name, "spec": arity, "header": header[name]["arity"]})
        if name in exports and exports[name]["arity"] != arity:
            arity_mismatches.append({"function": name, "spec": arity, "source": exports[name]["arity"]})
        if name in header and header[name]["prototype"] != normalize_proto(proto):
            proto_mismatches.append({"function": name, "spec": normalize_proto(proto), "header": header[name]["prototype"]})
    for m in arity_mismatches:
        findings.append(f"{m['function']}: arity {m}")
    for m in proto_mismatches:
        findings.append(f"{m['function']}: prototype differs: spec `{m['spec']}` header `{m['header']}`")
    header_text = header_path.read_text(encoding="utf-8", errors="replace") if header_path.exists() else ""
    source_text = source_path.read_text(encoding="utf-8", errors="replace") if source_path.exists() else ""
    version_in_header = f["VERSION_STRING"] in header_text
    version_in_source = ('"' + f["VERSION_STRING"] + '"') in source_text
    max_dim_in_header = str(f["MAX_DIM"]) in header_text
    if header_text and not version_in_header:
        findings.append(f"the header does not mention version {f['VERSION_STRING']}")
    if source_text and not version_in_source:
        findings.append(f"the source does not return version {f['VERSION_STRING']}")
    if header_text and not max_dim_in_header:
        findings.append(f"the header does not mention the maximum dimension {f['MAX_DIM']}")
    fixture = cc_syntax_check(FIXTURE, header_path.parent) if header_path.exists() and shutil.which("cc") else {"ok": False, "rc": None, "command": "", "diagnostics": "no header or no cc"}
    if not fixture["ok"]:
        findings.append("the fixture does not compile against the header: " + fixture.get("diagnostics", "")[:200])
    ast = ast_check(zig, source_path) if source_path.exists() else None
    says_no_parse = any("does not parse" in x for x in f["FINDINGS"])
    if ast is not None and ast["ok"] and says_no_parse:
        findings.append("the spec says the source does not parse at the pin, but `zig ast-check` accepted it")
    if ast is not None and not ast["ok"] and not says_no_parse:
        findings.append("`zig ast-check` rejects the source and the spec does not say so: " + ast["first_error"])
    report = {
        "version": 1,
        "generated_by": "tools/trinity_c_abi.py check",
        "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "spec": {"path": "specs/api/c_abi.t27", "sha256": sha256(SPEC.read_bytes()), "functions": len(spec_names)},
        "consumer": {"repo": "gHashTag/trinity", "revision": head, "pinned": f["PINNED_REVISION"], "header": f["HEADER"], "source": f["OWNER_MODULE"],
                     "header_sha256": sha256(header_path.read_bytes()) if header_path.exists() else None,
                     "source_sha256": sha256(source_path.read_bytes()) if source_path.exists() else None},
        "host": {"system": f"{platform.system()} {platform.machine()}", "cc": (run(["cc", "--version"])[1] or "").split("\n")[0][:80] if shutil.which("cc") else None},
        "header_functions": {k: v for k, v in sorted(header.items())},
        "source_exports": {k: v for k, v in sorted(exports.items())},
        "agreement": {"spec_header_source_names_equal": set(spec_names) == set(header) == set(exports) if header and exports else False,
                      "arity_mismatches": arity_mismatches, "prototype_mismatches": proto_mismatches,
                      "version_in_header": version_in_header, "version_in_source": version_in_source, "max_dim_in_header": max_dim_in_header},
        "fixture": {"path": "conformance/trinity/abi/abi_fixture.c", "sha256": sha256(FIXTURE.read_bytes()), "syntax_check": fixture},
        "ast_check": ast,
        "link_attempted": False,
        "evidence": "declared: prototypes and arity agree across spec, header and source; the fixture compiles against the header; the library is not built and the fixture is not linked at the pin",
        "findings": findings,
    }
    return report, findings


def self_check() -> int:
    if shutil.which("cc") is None:
        print("trinity_c_abi --self-check: needs cc", file=sys.stderr)
        return 2
    ok = True

    def expect(cond: bool, what: str):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and cond

    f = load_spec(SPEC)
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp) / "trinity"
        (root / pathlib.Path(f["HEADER"]).parent).mkdir(parents=True)
        (root / pathlib.Path(f["OWNER_MODULE"]).parent).mkdir(parents=True, exist_ok=True)
        run(["git", "init", "-q", str(root)])
        header = "#ifndef TRINITY_VSA_H\n#define TRINITY_VSA_H\n#include <stdint.h>\n#include <stddef.h>\n/* @version " + f["VERSION_STRING"] + " max " + str(f["MAX_DIM"]) + " */\ntypedef void* trinity_vsa_vector_t;\n" + "".join(p + ";\n" for p in f["FUNCTION_C"]) + "#endif\n"
        (root / f["HEADER"]).write_text(header)
        source = "".join("export fn %s(%s) void {}\n" % (n, ", ".join(f"p{i}: usize" for i in range(a))) for n, a in zip(f["FUNCTIONS"], f["FUNCTION_ARITY"]))
        source += 'const V = "' + f["VERSION_STRING"] + '";\n'
        (root / f["OWNER_MODULE"]).write_text(source)
        run(["git", "-C", str(root), "add", "-A"])
        run(["git", "-C", str(root), "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-q", "-m", "x"])
        report, findings = build_report(f, root, None)
        rev_only = [x for x in findings if not x.startswith("the clone is at")]
        expect(not rev_only, f"a header and a source that match the table give no finding but the revision ({len(rev_only)} other finding(s){': ' + rev_only[0] if rev_only else ''})")
        expect(report["fixture"]["syntax_check"]["ok"], "the fixture compiles against a header made of the spec's prototypes")
        # planted: a prototype missing from the header
        (root / f["HEADER"]).write_text(header.replace(f["FUNCTION_C"][6] + ";\n", ""))
        _, findings = build_report(f, root, None)
        expect(any("not in the header" in x for x in findings), "planted: a function missing from the header is reported")
        (root / f["HEADER"]).write_text(header)
        # planted: an arity change in the source
        (root / f["OWNER_MODULE"]).write_text(source.replace("export fn trinity_vsa_bind(p0: usize, p1: usize)", "export fn trinity_vsa_bind(p0: usize)"))
        _, findings = build_report(f, root, None)
        expect(any("trinity_vsa_bind: arity" in x for x in findings), "planted: an arity change in the source is reported")
        (root / f["OWNER_MODULE"]).write_text(source)
        # planted: a prototype that differs in type
        (root / f["HEADER"]).write_text(header.replace("double trinity_vsa_cosine_similarity", "float trinity_vsa_cosine_similarity"))
        _, findings = build_report(f, root, None)
        expect(any("prototype differs" in x and "cosine" in x for x in findings), "planted: a changed return type is reported")
        (root / f["HEADER"]).write_text(header)
        # planted: a fixture that misuses the header
        bad = FIXTURE.read_text().replace("trinity_vsa_bind(key, val)", "trinity_vsa_bind(key, val, val)")
        bad_path = pathlib.Path(tmp) / "bad_fixture.c"
        bad_path.write_text(bad)
        r = cc_syntax_check(bad_path, root / pathlib.Path(f["HEADER"]).parent)
        expect(not r["ok"], "planted: a fixture calling with the wrong arity fails the syntax check")
    print("trinity_c_abi --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["check"])
    ap.add_argument("--trinity-root")
    ap.add_argument("--zig")
    ap.add_argument("--report", default=str(REPORT))
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    if a.self_check:
        return self_check()
    if a.command != "check":
        ap.print_help()
        return 2
    if not a.trinity_root:
        print("check: --trinity-root is required", file=sys.stderr)
        return 2
    trinity = pathlib.Path(a.trinity_root).resolve()
    if not trinity.exists() or not SPEC.exists() or shutil.which("cc") is None:
        print("check: needs the clone, the spec and cc", file=sys.stderr)
        return 2
    try:
        f = load_spec(SPEC)
    except ValueError as e:
        print(f"finding: {e}")
        return 1
    report, findings = build_report(f, trinity, a.zig)
    pathlib.Path(a.report).write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for x in findings:
        print(f"finding: {x}")
    ag = report["agreement"]
    print(f"check: {len(f['FUNCTIONS'])} functions; header {len(report['header_functions'])}, source {len(report['source_exports'])}; names equal {ag['spec_header_source_names_equal']}; "
          f"fixture syntax {'ok' if report['fixture']['syntax_check']['ok'] else 'FAILED'}; ast-check {report['ast_check']['ok'] if report['ast_check'] else 'not run'}; {len(findings)} finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
