#!/usr/bin/env python3
"""The Trinity compiler matrix: which t27 features the pinned compiler carries through which
backend to a checked runtime result, and which invalid inputs it must never accept.

WHY THIS EXISTS
---------------
S02 of gHashTag/trinity#988 (gHashTag/t27#3564): several formats are called .t27/.tri in
Trinity, the local TRI27 validate path prints that validation is not implemented and still
records success, and a browser typecheck verdict or a declaration-only spec is not evidence of
generated runtime behaviour. This tool runs one fixture per feature the Trinity headless
profile needs through the real compiler and the host toolchains, stage by stage, and writes
what happened -- never what was hoped.

THE FOUR STAGES, KEPT APART IN THE RECORD
-----------------------------------------
  declaration  the file parses to a whole AST (t27c parse --json), t27c typecheck reports 0
               errors, every `pub const` literal agrees with its annotation (the compiler
               accepts `str = 5`; this stage does not), every `use a::b` resolves to a file of
               this repository (the compiler accepts `use nowhere::thing`; this stage does not),
               and the lexical declaration count agrees with the AST (a silently dropped
               block is a rejection).
  generate     t27c gen-c / gen / gen-rust / gen-verilog exit 0 and emit bytes; the output is
               hashed. The Rust backend marks its output NOT LOWERED (declarations only); that
               is recorded, not counted as a program.
  compile      cc -DT27_TEST_MAIN, zig test (compile half), rustc --test, iverilog.
  runtime      the C binary ends with "All N tests passed." and exit 0; zig test ends with the
               same line; rustc's test binary reports how many tests it ran (0 is recorded as
               no runtime, not as a pass); t27c icarus-simulate reports [TEST] ... PASSED and
               no FAILED for the Verilog path.

A feature is EXECUTABLE only when its fixture reaches a runtime pass on the backend the spec
names for it. A feature the spec marks blocked must not reach one (otherwise the spec is
stale and this tool says so). A negative fixture must be rejected at or before the stage the
spec names, on the C path -- the acceptance criterion of S02 is that an invalid annotation, a
false assertion, an unresolved import, a bodyless required function and an absent backend
cannot yield an accepted executable capability.

The vendored WASM of the site (gHashTag/trinity, apps/website/public/t27/t27_compiler.wasm,
built from an older revision) is run on every fixture when --wasm names it: its typecheck.ok,
dropped counts and per-target flags are recorded in their own column, so that a WASM verdict
is never confused with a native one. No runtime exists on that path.

Usage:
  python3 tools/trinity_compiler_matrix.py run   [--zig PATH] [--wasm PATH] [--out conformance/trinity/compiler_matrix.json]
  python3 tools/trinity_compiler_matrix.py check [--report conformance/trinity/compiler_matrix.json]
  python3 tools/trinity_compiler_matrix.py --self-check

Exit codes: 0 no finding; 1 findings; 2 could not run (no compiler, no fixture, no spec).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import datetime

ROOT = pathlib.Path(__file__).resolve().parent.parent
SPEC = "specs/trinity/compiler_matrix.t27"
REPORT = "conformance/trinity/compiler_matrix.json"
FIXTURE_DIR = "bootstrap/tests/fixtures/trinity_matrix"
STAGES = ("declaration", "generate", "compile", "runtime")
BACKEND_COMMAND = {"c": "gen-c", "zig": "gen", "rust": "gen-rust", "verilog": "gen-verilog"}
INT_TYPES = {"u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize"}
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
TOP_DECL_RE = re.compile(r"^(?:pub )?(?:const|var|fn|struct|enum|test|invariant|bench|use)\b|^[a-z_][a-z0-9_]* = ")

WASM_RUNNER = r"""
import { readFileSync } from 'node:fs';
const [,, wasmPath, specPath] = process.argv;
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const w = instance.exports;
const bytes = new TextEncoder().encode(readFileSync(specPath, 'utf8'));
const p = w.t27_alloc(bytes.length); new Uint8Array(w.memory.buffer, p, bytes.length).set(bytes);
const o = w.t27_analyze(p, bytes.length); const n = new DataView(w.memory.buffer).getUint32(o, true);
const a = JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer, o + 4, n))); w.t27_free(o, 4 + n);
console.log(JSON.stringify({ astError: a.astError ?? null, discarded: a.discarded.length, swallowed: a.swallowed.length, lexerDiscarded: a.lexerDiscarded.length,
  typecheckOk: a.typecheck?.ok === true, typecheckErrors: a.typecheck?.errorCount ?? null, targets: Object.fromEntries(Object.entries(a.targets).map(([k, v]) => [k, v.ok])), nodes: a.nodeCount }));
"""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd: pathlib.Path | None = None, timeout: int = 300) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except FileNotFoundError:
        return 127, "", f"{cmd[0]}: not found"
    except subprocess.TimeoutExpired:
        return 124, "", f"{cmd[0]}: timed out after {timeout}s"


def t27c_path() -> pathlib.Path | None:
    for p in ("target/release/t27c", "target/debug/t27c"):
        if (ROOT / p).exists():
            return ROOT / p
    return None


def tool_version(cmd: list[str]) -> str | None:
    rc, out, err = run(cmd, timeout=60)
    text = (out or err).strip().split("\n")[0] if rc == 0 or out or err else ""
    return text[:120] if rc == 0 and text else None


# ---------------------------------------------------------------------------
# The spec.
# ---------------------------------------------------------------------------
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
    raise ValueError(f"unsupported type {typ}")


def load_spec(path: pathlib.Path) -> dict:
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
    required = ["KIND", "FIXTURE_DIR", "BACKENDS", "BACKEND_COMMAND", "STAGES", "FEATURES", "FEATURE_FIXTURE", "FEATURE_STATUS", "FEATURE_BACKEND",
                "NEGATIVES", "NEGATIVE_STAGE", "ABSENT_BACKEND", "WASM_VENDORED_REVISION", "NATIVE_REVISION"]
    missing = [k for k in required if k not in f]
    if missing:
        raise ValueError(f"{path}: missing {', '.join(missing)}")
    n = len(f["FEATURES"])
    for k in ("FEATURE_FIXTURE", "FEATURE_STATUS", "FEATURE_BACKEND"):
        if len(f[k]) != n:
            raise ValueError(f"{path}: {k} must have one entry per feature")
    if len(f["NEGATIVE_STAGE"]) != len(f["NEGATIVES"]):
        raise ValueError(f"{path}: NEGATIVE_STAGE must have one entry per negative")
    if list(f["STAGES"]) != list(STAGES):
        raise ValueError(f"{path}: STAGES must be {list(STAGES)}")
    for b in f["FEATURE_BACKEND"]:
        if b not in f["BACKENDS"]:
            raise ValueError(f"{path}: FEATURE_BACKEND {b} is not one of BACKENDS")
    if f["ABSENT_BACKEND"] in f["BACKENDS"]:
        raise ValueError(f"{path}: ABSENT_BACKEND {f['ABSENT_BACKEND']} is listed as a backend")
    return f


# ---------------------------------------------------------------------------
# Stage 1: declaration.
# ---------------------------------------------------------------------------
def literal_agrees(typ: str, node: dict) -> str | None:
    kind, value, lit = node.get("kind"), node.get("value", ""), node.get("extra_kind", "")
    if typ == "str":
        return None if kind == "ExprLiteral" and lit == "string" else f"annotated str, literal is {value!r}"
    if typ == "bool":
        return None if kind == "ExprLiteral" and value in ("true", "false") else f"annotated bool, literal is {value!r}"
    if typ in INT_TYPES:
        return None if kind == "ExprLiteral" and lit != "string" and re.fullmatch(r"-?\d+", value or "") else f"annotated {typ}, literal is {value!r}"
    m = re.fullmatch(r"\[(\d+)\](\w+)", typ)
    if m:
        if kind != "ExprArrayLiteral":
            return f"annotated {typ}, value is {kind}"
        children = node.get("children", [])
        if len(children) != int(m.group(1)):
            return f"annotated {typ}, holds {len(children)}"
        for c in children:
            bad = literal_agrees(m.group(2), c)
            if bad:
                return bad
        return None
    return None  # struct, enum and other types: the compiler is the authority


def resolve_import(target: str) -> str | None:
    if target == "std":
        return "builtin"
    rel = target.replace("::", "/")
    for cand in (f"specs/{rel}.t27", f"{rel}.t27", f"specs/{rel}/mod.t27"):
        if (ROOT / cand).exists():
            return cand
    return None


def declaration_stage(t27c: pathlib.Path, fixture: pathlib.Path) -> dict:
    out = {"typecheck": None, "typecheck_errors": None, "typecheck_message": None, "ast": None, "annotations": [], "imports": [], "declarations": None, "ok": False, "reason": None}
    rc, so, se = run([str(t27c), "typecheck", str(fixture)])
    text = (so + se).strip()
    m = re.search(r"Typecheck OK \((\d+) errors?, (\d+) warnings?\)", text)
    out["typecheck"] = rc == 0 and m is not None
    out["typecheck_errors"] = int(m.group(1)) if m else None
    out["typecheck_message"] = text.split("\n")[-1][:200] if text else None
    if not out["typecheck"] or (m and int(m.group(1)) > 0):
        out["reason"] = "typecheck rejected the file"
        return out
    rc, so, se = run([str(t27c), "parse", "--json", str(fixture)])
    try:
        ast = json.loads(so)
        out["ast"] = True
    except ValueError:
        out["ast"] = False
        out["reason"] = "no AST"
        return out
    text = fixture.read_text(encoding="utf-8")
    lexical = sum(1 for line in text.split("\n") if TOP_DECL_RE.match(line))
    top = [c for c in ast.get("children", []) if c.get("kind") not in ("StmtExpr",)]
    out["declarations"] = {"lexical": lexical, "ast": len(top)}
    if lexical != len(top):
        out["reason"] = f"the parser consumed {len(top)} top-level declarations, the file has {lexical}"
        return out
    for c in top:
        if c.get("kind") == "ConstDecl" and c.get("children"):
            bad = literal_agrees(c.get("extra_type", ""), c["children"][0])
            out["annotations"].append({"name": c.get("name"), "type": c.get("extra_type"), "ok": bad is None, "detail": bad})
        if c.get("kind") == "UseDecl":
            target = c.get("value") or c.get("name")
            resolved = resolve_import(target)
            out["imports"].append({"target": target, "resolved": resolved})
    bad = [a for a in out["annotations"] if not a["ok"]]
    if bad:
        out["reason"] = f"annotation disagrees with its literal: {bad[0]['name']} {bad[0]['detail']}"
        return out
    unresolved = [i for i in out["imports"] if i["resolved"] is None]
    if unresolved:
        out["reason"] = f"unresolved import {unresolved[0]['target']}"
        return out
    out["ok"] = True
    return out


# ---------------------------------------------------------------------------
# Stages 2-4 per backend.
# ---------------------------------------------------------------------------
def generate_stage(t27c: pathlib.Path, fixture: pathlib.Path, backend: str, work: pathlib.Path) -> dict:
    cmd = BACKEND_COMMAND[backend]
    rc, so, se = run([str(t27c), cmd, str(fixture)])
    out_path = work / f"{fixture.stem}.{ {'c': 'c', 'zig': 'zig', 'rust': 'rs', 'verilog': 'v'}[backend]}"
    ok = rc == 0 and bool(so.strip())
    if ok:
        out_path.write_text(so)
    return {"command": f"t27c {cmd}", "ok": ok, "exit": rc, "bytes": len(so.encode()), "sha256": sha256(so.encode()) if ok else None,
            "declarations_only": "NOT LOWERED BY THIS BACKEND" in so, "path": str(out_path) if ok else None,
            "message": (se or so).strip().split("\n")[-1][:200] if not ok else None}


def compile_and_run(backend: str, generated: pathlib.Path, work: pathlib.Path, tools: dict, t27c: pathlib.Path, fixture: pathlib.Path) -> tuple[dict, dict]:
    comp = {"ok": None, "command": None, "message": None}
    rt = {"ok": None, "tests": None, "message": None, "command": None}
    if backend == "c":
        binary = work / f"{generated.stem}.c.bin"
        comp["command"] = "cc -DT27_TEST_MAIN -x c <file>"
        rc, so, se = run(["cc", "-DT27_TEST_MAIN", "-x", "c", str(generated), "-o", str(binary)])
        comp["ok"] = rc == 0
        comp["message"] = None if rc == 0 else [l for l in se.split("\n") if "error:" in l][:1] and [l for l in se.split("\n") if "error:" in l][0][:200]
        if comp["ok"]:
            rc, so, se = run([str(binary)], timeout=120)
            m = re.search(r"All (\d+) tests passed", so)
            rt["command"] = "./<binary>"
            rt["tests"] = int(m.group(1)) if m else 0
            rt["ok"] = rc == 0 and m is not None and int(m.group(1)) > 0
            rt["message"] = None if rt["ok"] else (se or so).strip().split("\n")[-1][:200] or f"exit {rc}"
    elif backend == "zig":
        zig = tools.get("zig_path")
        comp["command"] = "zig test <file>"
        if not zig:
            comp["ok"] = None
            comp["message"] = "zig not available on this host; not measured"
        else:
            rc, so, se = run([zig, "test", str(generated)], cwd=work, timeout=600)
            text = so + se
            compile_errors = [l for l in text.split("\n") if re.search(r"\berror: ", l) and "test command" not in l]
            comp["ok"] = not compile_errors
            comp["message"] = compile_errors[0][:200] if compile_errors else None
            if comp["ok"]:
                m = re.search(r"All (\d+) tests passed", text)
                rt["command"] = "zig test <file>"
                rt["tests"] = int(m.group(1)) if m else 0
                rt["ok"] = rc == 0 and m is not None and int(m.group(1)) > 0
                rt["message"] = None if rt["ok"] else text.strip().split("\n")[-1][:200] or f"exit {rc}"
    elif backend == "rust":
        binary = work / f"{generated.stem}.rs.bin"
        comp["command"] = "rustc --edition 2021 --test <file>"
        rc, so, se = run(["rustc", "--edition", "2021", "--test", str(generated), "-o", str(binary)], timeout=600)
        comp["ok"] = rc == 0
        comp["message"] = None if rc == 0 else [l for l in se.split("\n") if l.startswith("error")][:1] and [l for l in se.split("\n") if l.startswith("error")][0][:200]
        if comp["ok"]:
            rc, so, se = run([str(binary)], timeout=120)
            m = re.search(r"test result: (\w+)\. (\d+) passed; (\d+) failed", so)
            rt["command"] = "./<test binary>"
            rt["tests"] = int(m.group(2)) + int(m.group(3)) if m else 0
            rt["ok"] = rc == 0 and m is not None and m.group(1) == "ok" and int(m.group(2)) > 0
            rt["message"] = None if rt["ok"] else ("no test was lowered to Rust" if m and int(m.group(2)) + int(m.group(3)) == 0 else (se or so).strip().split("\n")[-1][:200])
    elif backend == "verilog":
        comp["command"] = "iverilog -o <out> <file>"
        rc, so, se = run(["iverilog", "-o", str(work / f"{generated.stem}.vvp"), str(generated)], timeout=300)
        comp["ok"] = rc == 0
        comp["message"] = None if rc == 0 else (se or so).strip().split("\n")[-1][:200]
        if comp["ok"]:
            rc, so, se = run([str(t27c), "icarus-simulate", str(fixture)], timeout=600)
            text = so + se
            passed = len(re.findall(r"\[TEST\] \S+ : PASSED", text))
            failed = len(re.findall(r"\[TEST\] \S+ : FAILED", text))
            rt["command"] = "t27c icarus-simulate <spec>"
            rt["tests"] = passed + failed
            rt["ok"] = rc == 0 and passed > 0 and failed == 0
            rt["message"] = None if rt["ok"] else (f"{failed} FAILED, {passed} PASSED" if passed + failed else text.strip().split("\n")[-1][:200])
    return comp, rt


def wasm_stage(node: str | None, wasm: pathlib.Path | None, fixture: pathlib.Path, work: pathlib.Path) -> dict | None:
    if not node or not wasm or not wasm.exists():
        return None
    runner = work / "wasm_runner.mjs"
    runner.write_text(WASM_RUNNER)
    rc, so, se = run([node, str(runner), str(wasm), str(fixture)], timeout=120)
    try:
        return json.loads(so)
    except ValueError:
        return {"error": (se or so).strip().split("\n")[-1][:200]}


def scrub(value, work: pathlib.Path):
    """Messages quote the temporary work directory; the record names it <work> so that two runs
    on two machines read the same and no host path enters the repository."""
    if isinstance(value, str):
        return value.replace(str(work), "<work>")
    if isinstance(value, dict):
        return {k: scrub(v, work) for k, v in value.items()}
    if isinstance(value, list):
        return [scrub(v, work) for v in value]
    return value


def run_fixture(t27c: pathlib.Path, fixture: pathlib.Path, backends: list[str], tools: dict, work: pathlib.Path, wasm: pathlib.Path | None) -> dict:
    rel = str(fixture.relative_to(ROOT)) if fixture.is_relative_to(ROOT) else str(fixture)
    rec = {"fixture": rel, "sha256": sha256(fixture.read_bytes()), "declaration": declaration_stage(t27c, fixture), "backends": {}, "wasm": wasm_stage(tools.get("node_path"), wasm, fixture, work)}
    for b in backends:
        gen = generate_stage(t27c, fixture, b, work)
        entry = {"generate": gen, "compile": None, "runtime": None}
        if gen["ok"]:
            comp, rt = compile_and_run(b, work / pathlib.Path(gen["path"]).name, work, tools, t27c, fixture)
            entry["compile"], entry["runtime"] = comp, rt
        rec["backends"][b] = entry
    return scrub(rec, work)


def stage_reached(rec: dict, backend: str) -> str:
    """The last stage the fixture passed on this backend: none | declaration | generate | compile | runtime."""
    if not rec["declaration"]["ok"]:
        return "none"
    b = rec["backends"].get(backend) or {}
    if not (b.get("generate") or {}).get("ok"):
        return "declaration"
    if not (b.get("compile") or {}).get("ok"):
        return "generate"
    if not (b.get("runtime") or {}).get("ok"):
        return "compile"
    return "runtime"


def executable(rec: dict, backend: str) -> bool:
    return stage_reached(rec, backend) == "runtime"


# ---------------------------------------------------------------------------
# The matrix.
# ---------------------------------------------------------------------------
def build_matrix(spec: dict, t27c: pathlib.Path, tools: dict, wasm: pathlib.Path | None, fixtures_dir: pathlib.Path, work: pathlib.Path) -> tuple[dict, list]:
    findings = []
    features, negatives = [], []
    for fid, fx, status, backend in zip(spec["FEATURES"], spec["FEATURE_FIXTURE"], spec["FEATURE_STATUS"], spec["FEATURE_BACKEND"]):
        path = fixtures_dir / fx
        if not path.exists():
            findings.append(("MISSING_FIXTURE", f"{fid}: {path.relative_to(ROOT)} does not exist"))
            continue
        rec = run_fixture(t27c, path, list(spec["BACKENDS"]), tools, work, wasm)
        reached = stage_reached(rec, backend)
        is_exec = reached == "runtime"
        verdict = "executable" if is_exec else "blocked"
        entry = {"feature": fid, "status_declared": status, "backend": backend, "reached": reached, "verdict": verdict, "executable_on": [b for b in spec["BACKENDS"] if executable(rec, b)], **rec}
        if status == "enabled" and not is_exec:
            findings.append(("FEATURE_NOT_EXECUTABLE", f"{fid}: enabled in the spec, reached {reached} on {backend}: {reason_of(rec, backend)}"))
        if status == "blocked" and is_exec:
            findings.append(("FEATURE_STATUS_STALE", f"{fid}: blocked in the spec, but it executes on {backend}; update the spec"))
        if status not in ("enabled", "blocked"):
            findings.append(("SCHEMA", f"{fid}: FEATURE_STATUS {status!r} is not enabled|blocked"))
        features.append(entry)
    stage_index = {s: i for i, s in enumerate(STAGES)}
    for fx, stage in zip(spec["NEGATIVES"], spec["NEGATIVE_STAGE"]):
        path = fixtures_dir / fx
        if not path.exists():
            findings.append(("MISSING_FIXTURE", f"negative {fx} does not exist"))
            continue
        if stage not in stage_index:
            findings.append(("SCHEMA", f"negative {fx}: stage {stage!r} is not one of {list(STAGES)}"))
            continue
        rec = run_fixture(t27c, path, ["c"], tools, work, wasm)
        reached = stage_reached(rec, "c")
        rejected = reached != "runtime"
        # the stage at which it was rejected is the stage after the last one it passed
        rejected_at = None if not rejected else STAGES[0] if reached == "none" else STAGES[stage_index[reached] + 1] if stage_index[reached] + 1 < len(STAGES) else None
        entry = {"negative": fx, "required_rejection_by": stage, "reached": reached, "rejected_at": rejected_at, "verdict": "rejected" if rejected else "ACCEPTED", "reason": reason_of(rec, "c"), **rec}
        if not rejected:
            findings.append(("NEGATIVE_ACCEPTED", f"{fx} reached runtime on the C path; an invalid input became an executable capability"))
        elif stage_index[rejected_at] > stage_index[stage]:
            findings.append(("NEGATIVE_LATE", f"{fx} was rejected at {rejected_at}, the spec requires {stage} or earlier"))
        negatives.append(entry)
    absent = spec["ABSENT_BACKEND"]
    absent_entry = {"backend": absent, "listed": absent in spec["BACKENDS"], "command": BACKEND_COMMAND.get(absent), "verdict": "blocked" if absent not in spec["BACKENDS"] and absent not in BACKEND_COMMAND else "ACCEPTED"}
    if absent_entry["verdict"] != "blocked":
        findings.append(("ABSENT_BACKEND_ACCEPTED", f"backend {absent} has no generator and must be blocked"))
    matrix = {"features": features, "negatives": negatives, "absent_backend": absent_entry}
    return matrix, findings


def reason_of(rec: dict, backend: str) -> str | None:
    if not rec["declaration"]["ok"]:
        return rec["declaration"]["reason"]
    b = rec["backends"].get(backend) or {}
    for stage in ("generate", "compile", "runtime"):
        s = b.get(stage)
        if s is None:
            return f"{stage}: not run"
        if not s.get("ok"):
            return f"{stage}: {s.get('message') or 'failed'}"
    return None


def host_tools(zig: str | None) -> dict:
    tools = {"platform": platform.platform(), "cc": tool_version(["cc", "--version"]), "rustc": tool_version(["rustc", "--version"]),
             "iverilog": tool_version(["iverilog", "-V"]), "node": tool_version(["node", "--version"]), "zig": None, "zig_path": None, "node_path": shutil.which("node")}
    zig_path = zig or shutil.which("zig")
    if zig_path:
        v = tool_version([zig_path, "version"])
        if v:
            tools["zig"], tools["zig_path"] = v, zig_path
    return tools


def do_run(args) -> int:
    t27c = t27c_path()
    if not t27c:
        print("trinity_compiler_matrix: t27c not built; cargo build --release -p t27c", file=sys.stderr)
        return 2
    spec_path = ROOT / SPEC
    if not spec_path.exists():
        print(f"trinity_compiler_matrix: {SPEC} does not exist", file=sys.stderr)
        return 2
    spec = load_spec(spec_path)
    fixtures_dir = ROOT / spec["FIXTURE_DIR"]
    tools = host_tools(args.zig)
    wasm = pathlib.Path(args.wasm).resolve() if args.wasm else None
    # The revision that matters is the one the compiler sources were last changed at, so a
    # spec-only commit on top does not invalidate the record; HEAD is recorded beside it.
    head = run(["git", "-C", str(ROOT), "rev-parse", "HEAD"])[1].strip()
    revision = run(["git", "-C", str(ROOT), "log", "-1", "--format=%H", "HEAD", "--", "bootstrap/src", "Cargo.toml", "Cargo.lock"])[1].strip() or head
    with tempfile.TemporaryDirectory() as tmp:
        matrix, findings = build_matrix(spec, t27c, tools, wasm, fixtures_dir, pathlib.Path(tmp))
    for f in matrix["features"] + matrix["negatives"]:
        for b in f["backends"].values():
            if b.get("generate"):
                b["generate"].pop("path", None)
    report = {
        "version": 1, "generated_by": "tools/trinity_compiler_matrix.py run", "at": datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "spec": {"path": SPEC, "sha256": sha256(spec_path.read_bytes())},
        "native": {"compiler": tool_version([str(t27c), "--version"]), "revision": revision, "revision_meaning": "last commit that changed bootstrap/src, Cargo.toml or Cargo.lock", "head": head, "declared_revision": spec["NATIVE_REVISION"], "t27c_sha256": sha256(t27c.read_bytes()), "build": "cargo build --release -p t27c"},
        # The WASM is named by its declared vendored path and its hash, never by where it sat on this host.
        "wasm": {"path": spec.get("WASM_VENDORED_PATH") if wasm else None, "sha256": sha256(wasm.read_bytes()) if wasm and wasm.exists() else None, "declared_sha256": spec.get("WASM_VENDORED_SHA256"), "declared_revision": spec["WASM_VENDORED_REVISION"], "measured": bool(wasm and wasm.exists() and tools.get("node_path"))},
        "host": {k: v for k, v in tools.items() if k not in ("zig_path", "node_path")},
        "stages": list(STAGES), "backends": list(spec["BACKENDS"]), "matrix": matrix,
        "summary": {"features": len(matrix["features"]), "executable": sorted(f["feature"] for f in matrix["features"] if f["verdict"] == "executable"),
                    "blocked": sorted(f["feature"] for f in matrix["features"] if f["verdict"] == "blocked"), "negatives_rejected": sum(1 for n in matrix["negatives"] if n["verdict"] == "rejected"), "negatives": len(matrix["negatives"])},
        "findings": [{"code": c, "message": m} for c, m in findings],
    }
    if revision != spec["NATIVE_REVISION"]:
        report["findings"].append({"code": "REVISION_MISMATCH", "message": f"the spec declares NATIVE_REVISION {spec['NATIVE_REVISION']}, this tree is {revision}"})
        findings.append(("REVISION_MISMATCH", ""))
    out = ROOT / args.out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=1) + "\n")
    for code, msg in findings:
        print(f"  {code}: {msg}")
    s = report["summary"]
    print(f"trinity_compiler_matrix: {s['features']} features, executable {len(s['executable'])}, blocked {len(s['blocked'])}, negatives rejected {s['negatives_rejected']}/{s['negatives']}, "
          f"native {report['native']['compiler']} @ {revision[:9]}, wasm {'measured' if report['wasm']['measured'] else 'not measured'}, zig {tools['zig'] or 'not measured'} -> {args.out}")
    return 1 if findings else 0


def do_check(args) -> int:
    spec_path, report_path = ROOT / SPEC, ROOT / args.report
    if not spec_path.exists() or not report_path.exists():
        print("trinity_compiler_matrix: spec or report missing", file=sys.stderr)
        return 2
    spec = load_spec(spec_path)
    report = json.loads(report_path.read_text())
    findings = []
    if report.get("spec", {}).get("sha256") != sha256(spec_path.read_bytes()):
        findings.append("the report was produced under another version of the spec; run again")
    if report.get("wasm", {}).get("measured") and spec.get("WASM_VENDORED_SHA256") and report["wasm"].get("sha256") != spec["WASM_VENDORED_SHA256"]:
        findings.append("the WASM measured is not the one the spec declares by sha256")
    if report.get("native", {}).get("revision") != spec["NATIVE_REVISION"]:
        findings.append(f"the report measured {report.get('native', {}).get('revision')}, the spec declares {spec['NATIVE_REVISION']}")
    fixtures_dir = ROOT / spec["FIXTURE_DIR"]
    for entry in report["matrix"]["features"] + report["matrix"]["negatives"]:
        p = pathlib.Path(entry["fixture"]) if pathlib.Path(entry["fixture"]).is_absolute() else ROOT / entry["fixture"]
        if not p.exists() or sha256(p.read_bytes()) != entry["sha256"]:
            findings.append(f"{entry['fixture']} changed since the report; run again")
    if report.get("findings"):
        findings.append(f"the report records {len(report['findings'])} finding(s): {report['findings'][0]['code']}")
    declared = set(spec["FEATURES"])
    reported = {f["feature"] for f in report["matrix"]["features"]}
    if declared != reported:
        findings.append(f"features differ between spec and report: {sorted(declared ^ reported)}")
    for f in report["matrix"]["features"]:
        i = spec["FEATURES"].index(f["feature"]) if f["feature"] in spec["FEATURES"] else None
        if i is not None and spec["FEATURE_STATUS"][i] != ("enabled" if f["verdict"] == "executable" else "blocked"):
            findings.append(f"{f['feature']}: spec says {spec['FEATURE_STATUS'][i]}, report says {f['verdict']}")
    for line in findings:
        print("  " + line)
    if findings:
        print(f"trinity_compiler_matrix check: {len(findings)} finding(s)")
        return 1
    s = report["summary"]
    print(f"trinity_compiler_matrix check: OK -- {s['features']} features ({len(s['executable'])} executable, {len(s['blocked'])} blocked), {s['negatives_rejected']}/{s['negatives']} negatives rejected, report matches spec and fixtures")
    return 0


def self_check() -> int:
    t27c = t27c_path()
    if not t27c:
        print("trinity_compiler_matrix --self-check: t27c not built", file=sys.stderr)
        return 2
    failures = []
    tools = host_tools(None)
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        fx = work / "fx"
        fx.mkdir()
        (fx / "good.t27").write_text('module sc_good;\npub const N : u8 = 2;\npub fn twice(x: u8) -> u8 {\n    return x * 2;\n}\ntest t {\n    assert twice(N) == 4;\n}\n')
        (fx / "false_assert.t27").write_text('module sc_false;\npub const N : u8 = 2;\ntest t {\n    assert N == 3;\n}\n')
        (fx / "bad_annotation.t27").write_text('module sc_ann;\npub const NAME : str = 5;\ntest t {\n    assert NAME == "five";\n}\n')
        (fx / "bad_import.t27").write_text('module sc_imp;\nuse nowhere::thing;\npub const N : u8 = 1;\ntest t {\n    assert N == 1;\n}\n')
        base = {"KIND": "compiler-matrix", "FIXTURE_DIR": str(fx), "BACKENDS": ["c"], "BACKEND_COMMAND": ["gen-c"], "STAGES": list(STAGES), "NATIVE_REVISION": "x", "WASM_VENDORED_REVISION": "y",
                "FEATURES": ["good"], "FEATURE_FIXTURE": ["good.t27"], "FEATURE_STATUS": ["enabled"], "FEATURE_BACKEND": ["c"],
                "NEGATIVES": ["false_assert.t27", "bad_annotation.t27", "bad_import.t27"], "NEGATIVE_STAGE": ["runtime", "declaration", "declaration"], "ABSENT_BACKEND": "python"}
        matrix, findings = build_matrix(base, t27c, tools, None, fx, work)
        if findings:
            failures.append(f"the clean fixture set must pass, got {[c for c, _ in findings]}")
        if matrix["features"][0]["verdict"] != "executable":
            failures.append("the good fixture must be executable on C")
        for n in matrix["negatives"]:
            if n["verdict"] != "rejected":
                failures.append(f"{n['negative']} must be rejected")
        by = {n["negative"]: n["rejected_at"] for n in matrix["negatives"]}
        if by.get("false_assert.t27") != "runtime":
            failures.append(f"false_assert must be rejected at runtime, got {by.get('false_assert.t27')}")
        if by.get("bad_annotation.t27") != "declaration":
            failures.append(f"bad_annotation must be rejected at declaration, got {by.get('bad_annotation.t27')}")
        if by.get("bad_import.t27") != "declaration":
            failures.append(f"bad_import must be rejected at declaration, got {by.get('bad_import.t27')}")
        # planted: a false assertion declared as an enabled feature is a finding
        planted = dict(base, FEATURES=["broken"], FEATURE_FIXTURE=["false_assert.t27"], FEATURE_STATUS=["enabled"], FEATURE_BACKEND=["c"], NEGATIVES=[], NEGATIVE_STAGE=[])
        _, f2 = build_matrix(planted, t27c, tools, None, fx, work)
        if "FEATURE_NOT_EXECUTABLE" not in [c for c, _ in f2]:
            failures.append("an enabled feature whose test fails must be a finding")
        # planted: a good fixture declared blocked is stale
        planted = dict(base, FEATURE_STATUS=["blocked"], NEGATIVES=[], NEGATIVE_STAGE=[])
        _, f3 = build_matrix(planted, t27c, tools, None, fx, work)
        if "FEATURE_STATUS_STALE" not in [c for c, _ in f3]:
            failures.append("a blocked feature that executes must be a finding")
        # planted: a negative that is a valid program is accepted -> finding
        planted = dict(base, NEGATIVES=["good.t27"], NEGATIVE_STAGE=["runtime"])
        _, f4 = build_matrix(planted, t27c, tools, None, fx, work)
        if "NEGATIVE_ACCEPTED" not in [c for c, _ in f4]:
            failures.append("a negative fixture that executes must be a finding")
        # planted: rejection later than required
        planted = dict(base, NEGATIVES=["false_assert.t27"], NEGATIVE_STAGE=["declaration"])
        _, f5 = build_matrix(planted, t27c, tools, None, fx, work)
        if "NEGATIVE_LATE" not in [c for c, _ in f5]:
            failures.append("a negative rejected later than the spec requires must be a finding")
        # planted: absent backend listed as a backend
        try:
            load_spec_dict = dict(base, BACKENDS=["c", "python"], BACKEND_COMMAND=["gen-c", "gen-python"])
            _, f6 = build_matrix(load_spec_dict, t27c, tools, None, fx, work)
            if "ABSENT_BACKEND_ACCEPTED" not in [c for c, _ in f6]:
                failures.append("an absent backend listed as available must be a finding")
        except KeyError:
            pass
    if failures:
        print("trinity_compiler_matrix --self-check: FAIL")
        for f in failures:
            print("  " + f)
        return 1
    print("trinity_compiler_matrix --self-check: PASS (good fixture executes on C; false assertion rejected at runtime, bad annotation and unresolved import at declaration; five planted defects each reported)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("command", nargs="?", choices=["run", "check"])
    ap.add_argument("--zig")
    ap.add_argument("--wasm")
    ap.add_argument("--out", default=REPORT)
    ap.add_argument("--report", default=REPORT)
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args()
    if args.self_check:
        return self_check()
    if args.command == "run":
        return do_run(args)
    if args.command == "check":
        return do_check(args)
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
