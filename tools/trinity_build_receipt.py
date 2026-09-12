#!/usr/bin/env python3
"""The Trinity build graph and the receipts that tie a generated artifact to what produced it.

WHY THIS EXISTS
---------------
S03 of gHashTag/trinity#988 (gHashTag/t27#3565): Trinity's real build has many targets,
optional platform requirements and extracted packages, and recreating it needs a declared
graph and exact source-to-output provenance -- not the five-binary description of an old
README and not moving dependency branches. This tool reads the graph build.zig actually
declares, holds it to specs/trinity/build_graph.t27, and writes receipts for the
bootstrap/fixture profile that two clean runs must reproduce byte for byte.

THREE RECORDS
-------------
  graph     conformance/trinity/build_graph.json -- from a clean pinned checkout of the
            consumer (the same revision as conformance/trinity/inventory.json): every named
            module of build.zig with its root file, every import edge from an artifact or a
            module to the module or package it names, the pinned packages and whether
            build.zig references them, the profiles (installed by default, guarded, on
            demand), the untracked build outputs, and the tracked files that are generated
            with the generator and the inputs the spec declares for each, hashed at the
            pinned revision (blob hashes for files, tree hashes for directories).
  receipt   conformance/trinity/bootstrap_receipt.json -- the bootstrap/fixture profile of
            THIS repository: the compiler-source revision and the t27c binary hash, the host
            tools, every fixture of the compiler matrix with its hash, and the hash of what
            each backend generated from it in N independent runs. Deterministic means every
            run produced the same bytes under the declared normalization policy (none).
  check     recomputes what can be recomputed offline: the fixtures still hash as receipted,
            the compiler sources are still at the receipted revision, a fresh generation
            still produces the receipted bytes (a changed spec, compiler or dependency
            invalidates the receipt), and -- with --trinity-root -- the generated tracked
            files of the consumer against their inputs: a generated file that changed while
            its inputs did not is a suspected hand edit; inputs that changed while the file
            did not is a stale artifact; a file in a declared generated location that no
            spec entry registers is unregistered.

What this does not do: build the Trinity application. zig build -Dci=true is measured by
the consumer's own CI (gHashTag/trinity, Build & Test on ubuntu-latest with zig 0.15.2) and
full application-profile reproduction belongs to S12; the receipt here covers the profile
this repository can reproduce from a fresh clone: the compiler and the fixture generation.

Usage:
  python3 tools/trinity_build_receipt.py graph   --trinity-root DIR [--out conformance/trinity/build_graph.json]
  python3 tools/trinity_build_receipt.py receipt [--runs 2] [--out conformance/trinity/bootstrap_receipt.json]
  python3 tools/trinity_build_receipt.py check   [--trinity-root DIR]
  python3 tools/trinity_build_receipt.py --self-check

Exit codes: 0 no finding; 1 findings; 2 could not run.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import platform
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
SPEC = "specs/trinity/build_graph.t27"
MATRIX_SPEC = "specs/trinity/compiler_matrix.t27"
INVENTORY = "conformance/trinity/inventory.json"
GRAPH = "conformance/trinity/build_graph.json"
RECEIPT = "conformance/trinity/bootstrap_receipt.json"
BACKEND_COMMAND = {"c": "gen-c", "zig": "gen", "rust": "gen-rust", "verilog": "gen-verilog"}
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
INT_TYPES = {"u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize"}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd=None, timeout: int = 300) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except FileNotFoundError:
        return 127, "", f"{cmd[0]}: not found"
    except subprocess.TimeoutExpired:
        return 124, "", f"{cmd[0]}: timed out"


def git(root: pathlib.Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(root), *args], check=True, capture_output=True, text=True).stdout


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


def load_spec(path: pathlib.Path, required: list[str]) -> dict:
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
    missing = [k for k in required if k not in f]
    if missing:
        raise ValueError(f"{path}: missing {', '.join(missing)}")
    return f


GRAPH_REQUIRED = ["KIND", "CONSUMER_REPO", "PINNED_REVISION", "OPTIONS", "DEPENDENCIES", "DEPENDENCY_PINS", "DEPENDENCY_USE", "SUBMODULES",
                  "PROFILES", "PROFILE_STATUS", "OUTPUT_UNTRACKED", "GENERATED_TRACKED", "GENERATED_BY", "GENERATED_INPUTS", "GENERATED_LOCATIONS",
                  "NORMALIZATION", "BOOTSTRAP_RUNS", "BOOTSTRAP_COMMANDS"]


# ---------------------------------------------------------------------------
# build.zig: modules, imports, dependencies.
# ---------------------------------------------------------------------------
def balanced(src: str, start: int, open_ch: str, close_ch: str) -> int:
    depth, i, in_str = 0, start, False
    while i < len(src):
        c = src[i]
        if in_str:
            if c == "\\":
                i += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == open_ch:
            depth += 1
        elif c == close_ch:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unbalanced")


def parse_graph(text: str) -> dict:
    modules = {}
    for m in re.finditer(r"(?:const|var)\s+(\w+)\s*=\s*b\.createModule\(", text):
        end = balanced(text, m.end() - 1, "(", ")")
        body = text[m.end():end]
        root = re.search(r'\.root_source_file\s*=\s*b\.path\("([^"]+)"\)', body)
        modules[m.group(1)] = {"var": m.group(1), "root": root.group(1) if root else None, "line": text[: m.start()].count("\n") + 1}
    owners = []  # (owner label, body)
    for m in re.finditer(r"(?:const|var)\s+(\w+)\s*=\s*b\.(addExecutable|addTest|addLibrary|addStaticLibrary|addSharedLibrary|addObject|createModule)\(", text):
        end = balanced(text, m.end() - 1, "(", ")")
        body = text[m.end():end]
        kind = {"addExecutable": "exe", "addTest": "test", "createModule": "module"}.get(m.group(2), "lib")
        name = re.search(r'^\s*\.name\s*=\s*"([^"]*)"', body.split("{", 1)[-1], re.M)
        root = re.search(r'\.root_source_file\s*=\s*b\.path\("([^"]+)"\)', body)
        label = f"{kind}:{m.group(1)}" if kind == "module" else f"{kind}:{(root.group(1) if kind == 'test' and root else (name.group(1) if name else m.group(1)))}"
        owners.append((label, body, m.group(1)))
    deps_used = {}
    for m in re.finditer(r'b\.dependency\("(\w+)"', text):
        deps_used.setdefault(m.group(1), 0)
        deps_used[m.group(1)] += 1
    # `const dep = b.dependency("zig_hdc", .{})` and `const m = dep.module("hdc_vsa")`: the
    # variables through which a package module reaches an import.
    dep_vars = {m.group(1): m.group(2) for m in re.finditer(r'(?:const|var)\s+(\w+)\s*=\s*b\.dependency\("(\w+)"', text)}
    package_modules = {}
    for m in re.finditer(r'(?:const|var)\s+(\w+)\s*=\s*(\w+)\.module\("([^"]+)"\)', text):
        if m.group(2) in dep_vars:
            package_modules[m.group(1)] = {"package": dep_vars[m.group(2)], "module": m.group(3)}
    aliases = {m.group(1): m.group(2) for m in re.finditer(r'(?:const|var)\s+(\w+)\s*=\s*(\w+);', text)}
    edges = []

    def module_expr(text_: str, start: int) -> str:
        """The expression after `.module =` up to the comma or brace that closes the entry, with
        nested parentheses, braces and brackets kept whole (b.dependency("x", .{}).module("m"))."""
        depth, i = 0, start
        while i < len(text_):
            c = text_[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                if depth == 0:
                    break
                depth -= 1
            elif c == "," and depth == 0:
                break
            i += 1
        return text_[start:i].strip()

    for label, body, var in owners:
        for im in re.finditer(r'\.\{\s*\.name\s*=\s*"([^"]+)"\s*,\s*\.module\s*=\s*', body):
            expr = module_expr(body, im.end())
            edges.append({"from": label, "import": im.group(1), "expr": expr[:80]})
    for m in re.finditer(r'(\w+)\.root_module\.addImport\("([^"]+)",\s*([^)]+)\)', text):
        edges.append({"from": f"var:{m.group(1)}", "import": m.group(2), "expr": m.group(3).strip()[:80]})
    for e in edges:
        expr = e["expr"]
        seen = set()
        while expr in aliases and expr not in modules and expr not in package_modules and expr not in seen:
            seen.add(expr)
            expr = aliases[expr]
        if expr in modules:
            e["to"] = {"kind": "module", "var": expr, "root": modules[expr]["root"]}
        elif expr in package_modules:
            e["to"] = {"kind": "package", **package_modules[expr]}
        elif re.match(r'b\.dependency\("(\w+)"', expr):
            e["to"] = {"kind": "package", "package": re.match(r'b\.dependency\("(\w+)"', expr).group(1)}
        elif expr.startswith("b.createModule"):
            e["to"] = {"kind": "inline-module"}
        elif re.fullmatch(r"\w+", expr):
            e["to"] = {"kind": "variable", "var": expr}
        else:
            e["to"] = {"kind": "unresolved"}
    return {"modules": sorted(modules.values(), key=lambda x: x["var"]), "edges": edges, "dependencies_referenced": deps_used}


def graph(trinity: pathlib.Path, spec: dict, inventory: dict) -> tuple[dict, list]:
    findings = []
    sha = git(trinity, "rev-parse", "HEAD").strip()
    if git(trinity, "status", "--porcelain", "--untracked-files=no").strip():
        raise SystemExit("trinity_build_receipt: the consumer tree has modified tracked files")
    if sha != inventory.get("sha"):
        findings.append(("PIN_MISMATCH", f"the tree is {sha[:9]}, the inventory is of {str(inventory.get('sha'))[:9]}"))
    if sha != spec["PINNED_REVISION"]:
        findings.append(("PIN_MISMATCH", f"the tree is {sha[:9]}, the spec pins {spec['PINNED_REVISION'][:9]}"))
    text = (trinity / "build.zig").read_text(encoding="utf-8")
    g = parse_graph(text)
    tracked = set()
    for d, names in inventory.get("tree", {}).items():
        for n in names:
            tracked.add(f"{d}/{n}" if d else n)
    for m in g["modules"]:
        m["tracked"] = m["root"] in tracked if m["root"] else False
        if not m["tracked"]:
            findings.append(("MODULE_ROOT_MISSING", f"module {m['var']} names {m['root']} which is not tracked"))
    resolved = {"module": 0, "package": 0, "inline-module": 0, "variable": 0, "unresolved": 0}
    known_vars = {m["var"] for m in g["modules"]}
    for e in g["edges"]:
        kind = e["to"]["kind"]
        if kind == "variable" and e["to"]["var"] not in known_vars:
            # a variable that is not a createModule -- e.g. a wasm module alias -- is recorded, not counted as a module
            resolved["variable"] += 1
        else:
            resolved[kind] += 1
        if kind == "unresolved":
            findings.append(("UNRESOLVED_IMPORT", f"{e['from']} imports {e['import']} from {e['expr']!r}, which resolves to nothing build.zig declares"))
    zon = inventory["dependencies"]["dependencies"]
    deps = []
    for d, pin, use in zip(spec["DEPENDENCIES"], spec["DEPENDENCY_PINS"], spec["DEPENDENCY_USE"]):
        z = next((x for x in zon if x["name"] == d), None)
        if not z:
            findings.append(("DEPENDENCY_MISSING", f"{d} is declared in the spec but not in build.zig.zon"))
            continue
        if z["url"] != pin:
            findings.append(("PIN_MISMATCH", f"{d}: spec pins {pin}, build.zig.zon has {z['url']}"))
        deps.append({"name": d, "url": z["url"], "hash": z["hash"], "referenced_by_build_zig": g["dependencies_referenced"].get(d, 0), "use": use})
    for z in zon:
        if z["name"] not in spec["DEPENDENCIES"]:
            findings.append(("DEPENDENCY_UNDECLARED", f"build.zig.zon pins {z['name']} and the spec does not list it"))
    for name, cnt in g["dependencies_referenced"].items():
        if name not in spec["DEPENDENCIES"]:
            findings.append(("DEPENDENCY_UNPINNED", f"build.zig references dependency {name} which build.zig.zon does not pin"))
    generated = []
    for path, by, inputs in zip(spec["GENERATED_TRACKED"], spec["GENERATED_BY"], spec["GENERATED_INPUTS"]):
        p = path.split(":", 1)[-1]
        entry = {"path": p, "generator": by, "inputs": [], "sha256": None}
        if p not in tracked:
            findings.append(("GENERATED_MISSING", f"{p} is declared generated but is not tracked at {sha[:9]}"))
        else:
            entry["sha256"] = sha256(subprocess.run(["git", "-C", str(trinity), "show", f"{sha}:{p}"], capture_output=True, check=True).stdout)
        for inp in inputs.split(","):
            inp = inp.strip()
            if not inp:
                continue
            if inp.startswith("external:"):
                # an input outside the consumer tree (another repository, GitHub): named, never hashed
                entry["inputs"].append({"path": inp, "object": "external"})
                continue
            inp = inp.split(":", 1)[-1]
            rc, so, _ = run(["git", "-C", str(trinity), "rev-parse", f"{sha}:{inp}"])
            if rc != 0:
                findings.append(("INPUT_MISSING", f"{p}: input {inp} does not exist at {sha[:9]}"))
                entry["inputs"].append({"path": inp, "object": None})
            else:
                entry["inputs"].append({"path": inp, "object": so.strip()})
        generated.append(entry)
    inv_b = inventory["build"]
    profiles = {
        "headless": {"status": spec["PROFILE_STATUS"][spec["PROFILES"].index("headless")], "targets": inv_b["installed_by_default"]},
        "guarded": {"targets": inv_b["installed_guarded"], "guards": sorted({g for i in inv_b["installs"] for g in i["guards"]})},
        "on_demand_steps": sorted(s["key"] for s in inv_b["steps"]),
    }
    for name, status in zip(spec["PROFILES"], spec["PROFILE_STATUS"]):
        profiles.setdefault(name, {})["status"] = status
    out = {
        "version": 1, "generated_by": "tools/trinity_build_receipt.py graph", "repo": spec["CONSUMER_REPO"], "sha": sha,
        "options": inv_b["options"], "dependencies": deps, "submodules": inventory["submodules"],
        "modules": g["modules"], "edges": g["edges"], "edge_resolution": resolved, "edge_count": len(g["edges"]),
        "profiles": profiles, "outputs": {"untracked": spec["OUTPUT_UNTRACKED"], "generated_tracked": generated},
        "findings": [{"code": c, "message": m} for c, m in findings],
    }
    return out, findings


# ---------------------------------------------------------------------------
# The bootstrap/fixture-profile receipt of this repository.
# ---------------------------------------------------------------------------
def t27c_path() -> pathlib.Path | None:
    for p in ("target/release/t27c", "target/debug/t27c"):
        if (ROOT / p).exists():
            return ROOT / p
    return None


def compiler_identity(t27c: pathlib.Path) -> dict:
    head = git(ROOT, "rev-parse", "HEAD").strip()
    rev = git(ROOT, "log", "-1", "--format=%H", "HEAD", "--", "bootstrap/src", "Cargo.toml", "Cargo.lock").strip() or head
    rc, so, se = run([str(t27c), "--version"])
    return {"head": head, "compiler_revision": rev, "cargo_lock_sha256": sha256((ROOT / "Cargo.lock").read_bytes()) if (ROOT / "Cargo.lock").exists() else None,
            "t27c_version": (so or se).strip().split("\n")[0][:80], "t27c_sha256": sha256(t27c.read_bytes()), "build": "cargo build --release -p t27c"}


def generate_all(t27c: pathlib.Path, fixtures: list[pathlib.Path], backends: list[str]) -> dict:
    out = {}
    for f in fixtures:
        out[f.name] = {}
        for b in backends:
            rc, so, se = run([str(t27c), BACKEND_COMMAND[b], str(f)])
            out[f.name][b] = {"exit": rc, "sha256": sha256(so.encode()) if rc == 0 else None, "bytes": len(so.encode()) if rc == 0 else 0}
    return out


def receipt(runs: int, matrix_spec: dict, t27c: pathlib.Path) -> tuple[dict, list]:
    findings = []
    fixtures_dir = ROOT / matrix_spec["FIXTURE_DIR"]
    names = sorted(set(matrix_spec["FEATURE_FIXTURE"]) | set(matrix_spec["NEGATIVES"]))
    fixtures = [fixtures_dir / n for n in names]
    for f in fixtures:
        if not f.exists():
            findings.append(("MISSING_FIXTURE", str(f.relative_to(ROOT))))
    fixtures = [f for f in fixtures if f.exists()]
    backends = list(matrix_spec["BACKENDS"])
    results = [generate_all(t27c, fixtures, backends) for _ in range(runs)]
    differing = []
    for f in fixtures:
        for b in backends:
            hashes = {r[f.name][b]["sha256"] for r in results}
            if len(hashes) > 1:
                differing.append({"fixture": f.name, "backend": b, "hashes": sorted(h or "none" for h in hashes)})
    if differing:
        findings.append(("NONDETERMINISTIC", f"{len(differing)} output(s) differ between {runs} identical runs: " + ", ".join(f"{d['fixture']}/{d['backend']}" for d in differing[:5])))
    rec = {
        "version": 1, "generated_by": "tools/trinity_build_receipt.py receipt", "at": datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "compiler": compiler_identity(t27c), "host": {"platform": platform.platform()},
        "profile": {"commands": ["cargo build --release -p t27c"] + [f"t27c {BACKEND_COMMAND[b]} <fixture>" for b in backends], "fixture_dir": matrix_spec["FIXTURE_DIR"], "backends": backends},
        "normalization": "none: two runs must produce identical bytes",
        "runs": runs, "deterministic": not differing, "differing": differing,
        "fixtures": [{"name": f.name, "sha256": sha256(f.read_bytes()), "outputs": {b: results[0][f.name][b] for b in backends}} for f in fixtures],
        "findings": [{"code": c, "message": m} for c, m in findings],
    }
    return rec, findings


def check_receipt(rec: dict, matrix_spec: dict, t27c: pathlib.Path | None) -> list:
    findings = []
    fixtures_dir = ROOT / matrix_spec["FIXTURE_DIR"]
    rev = git(ROOT, "log", "-1", "--format=%H", "HEAD", "--", "bootstrap/src", "Cargo.toml", "Cargo.lock").strip()
    if rev != rec["compiler"]["compiler_revision"]:
        findings.append(("RECEIPT_INVALID", f"the compiler sources changed: receipted {rec['compiler']['compiler_revision'][:9]}, now {rev[:9]}"))
    lock = sha256((ROOT / "Cargo.lock").read_bytes()) if (ROOT / "Cargo.lock").exists() else None
    if lock != rec["compiler"]["cargo_lock_sha256"]:
        findings.append(("RECEIPT_INVALID", "Cargo.lock changed since the receipt"))
    if t27c and sha256(t27c.read_bytes()) != rec["compiler"]["t27c_sha256"]:
        findings.append(("BINARY_DIFFERS", "the built t27c is not the receipted binary; rebuilt from the same sources on another host is expected to differ, the generated bytes below decide"))
    changed = []
    for fx in rec["fixtures"]:
        p = fixtures_dir / fx["name"]
        if not p.exists() or sha256(p.read_bytes()) != fx["sha256"]:
            changed.append(fx["name"])
    if changed:
        findings.append(("RECEIPT_INVALID", f"fixture(s) changed since the receipt: {', '.join(changed[:5])}"))
    if t27c and not changed:
        fresh = generate_all(t27c, [fixtures_dir / fx["name"] for fx in rec["fixtures"]], rec["profile"]["backends"])
        drift = [f"{fx['name']}/{b}" for fx in rec["fixtures"] for b in rec["profile"]["backends"] if fresh[fx["name"]][b]["sha256"] != fx["outputs"][b]["sha256"]]
        if drift:
            findings.append(("RECEIPT_INVALID", f"a fresh generation differs from the receipt: {', '.join(drift[:5])}"))
    if not rec.get("deterministic", False):
        findings.append(("NONDETERMINISTIC", "the receipt records differing runs"))
    return findings


def check_graph_drift(g: dict, spec: dict, trinity: pathlib.Path) -> list:
    """The consumer at its current HEAD against the receipted generated files."""
    findings = []
    sha = git(trinity, "rev-parse", "HEAD").strip()
    for entry in g["outputs"]["generated_tracked"]:
        rc, blob, _ = run(["git", "-C", str(trinity), "show", f"{sha}:{entry['path']}"])
        if rc != 0:
            findings.append(("GENERATED_MISSING", f"{entry['path']} is not tracked at {sha[:9]}"))
            continue
        now_sha = sha256(blob.encode() if isinstance(blob, str) else blob)
        tracked_inputs = [i for i in entry["inputs"] if i["object"] != "external"]
        inputs_now = []
        for inp in tracked_inputs:
            rc2, obj, _ = run(["git", "-C", str(trinity), "rev-parse", f"{sha}:{inp['path']}"])
            inputs_now.append(obj.strip() if rc2 == 0 else None)
        inputs_changed = any(a != b["object"] for a, b in zip(inputs_now, tracked_inputs))
        if not tracked_inputs:
            continue  # only external inputs: drift cannot be judged from the tree
        file_changed = now_sha != entry["sha256"]
        if file_changed and not inputs_changed:
            findings.append(("HAND_EDIT_SUSPECTED", f"{entry['path']} changed since {g['sha'][:9]} while none of its declared inputs did"))
        if inputs_changed and not file_changed:
            findings.append(("STALE_GENERATED", f"{entry['path']} is unchanged since {g['sha'][:9]} while its inputs changed; run {entry['generator']}"))
    registered = {e["path"] for e in g["outputs"]["generated_tracked"]}
    # Only the locations the spec declares as generator output directories are swept; a state
    # directory that happens to hold one generated file (.trinity/) is not one of them.
    dirs = set(spec.get("GENERATED_LOCATIONS", []))
    for d in sorted(dirs):
        rc, listing, _ = run(["git", "-C", str(trinity), "ls-tree", "--name-only", f"{sha}:{d}"])
        if rc != 0:
            continue
        for name in listing.split("\n"):
            if not name or name in ("files", "models"):
                continue
            p = f"{d}/{name}"
            if p not in registered and name.endswith(".json") and any(r.startswith(d + "/") and r.endswith(".json") for r in registered):
                # JSON beside registered generated JSON in the same directory: either a generator wrote it or a hand did
                if p in spec.get("GENERATED_UNREGISTERED_ALLOWED", []):
                    continue
                findings.append(("UNREGISTERED_GENERATED", f"{p} sits in a generated location and no spec entry registers a generator for it"))
    return findings


def self_check() -> int:
    failures = []
    # 1. parse_graph resolves modules and flags an unresolved import
    text = 'const a_mod = b.createModule(.{ .root_source_file = b.path("src/a.zig") });\nconst exe = b.addExecutable(.{ .name = "x", .root_module = b.createModule(.{ .root_source_file = b.path("src/main.zig"), .imports = &.{ .{ .name = "a", .module = a_mod }, .{ .name = "g", .module = b.dependency("gf", .{}).module("golden_float") }, .{ .name = "ghost", .module = something.weird() } } }) });\n'
    g = parse_graph(text)
    kinds = [e["to"]["kind"] for e in g["edges"]]
    if kinds.count("module") != 1 or kinds.count("package") != 1 or kinds.count("unresolved") != 1:
        failures.append(f"parse_graph must resolve one module, one package and one unresolved import, got {kinds}")
    # 2. a receipt of two identical runs is deterministic; a changed fixture invalidates it
    t27c = t27c_path()
    if not t27c:
        failures.append("t27c not built")
    else:
        with tempfile.TemporaryDirectory() as tmp:
            d = pathlib.Path(tmp) / "fx"
            d.mkdir()
            (d / "one.t27").write_text('module sc_one;\npub const N : u8 = 1;\ntest t {\n    assert N == 1;\n}\n')
            spec = {"FIXTURE_DIR": os.path.relpath(d, ROOT), "FEATURE_FIXTURE": ["one.t27"], "NEGATIVES": [], "BACKENDS": ["c", "verilog"]}
            rec, f = receipt(2, spec, t27c)
            if f or not rec["deterministic"]:
                failures.append(f"two identical runs must be deterministic, got {f}")
            f2 = check_receipt(rec, spec, t27c)
            if f2:
                failures.append(f"an unchanged fixture must keep its receipt, got {f2}")
            (d / "one.t27").write_text('module sc_one;\npub const N : u8 = 2;\ntest t {\n    assert N == 2;\n}\n')
            f3 = check_receipt(rec, spec, t27c)
            if "RECEIPT_INVALID" not in [c for c, _ in f3]:
                failures.append("a changed fixture must invalidate the receipt")
            rec2 = dict(rec, deterministic=False, differing=[{"fixture": "one.t27", "backend": "c", "hashes": ["a", "b"]}])
            if "NONDETERMINISTIC" not in [c for c, _ in check_receipt(rec2, spec, None)]:
                failures.append("a receipt that records differing runs must be a finding")
    # 3. drift: a generated file that changed without its inputs is a suspected hand edit
    with tempfile.TemporaryDirectory() as tmp:
        repo = pathlib.Path(tmp)
        subprocess.run(["git", "init", "-q", str(repo)], check=True)
        (repo / "src").mkdir(); (repo / "out").mkdir()
        (repo / "src/in.txt").write_text("input v1\n"); (repo / "out/gen.json").write_text('{"v":1}\n'); (repo / "out/other.json").write_text('{"x":1}\n')
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "add", "-A"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "one"], check=True)
        sha1 = git(repo, "rev-parse", "HEAD").strip()
        obj = git(repo, "rev-parse", f"{sha1}:src/in.txt").strip()
        g = {"sha": sha1, "outputs": {"generated_tracked": [{"path": "out/gen.json", "generator": "make gen", "inputs": [{"path": "src/in.txt", "object": obj}], "sha256": sha256(b'{"v":1}\n')}]}}
        f0 = check_graph_drift(g, {"GENERATED_LOCATIONS": ["out"]}, repo)
        if [c for c, _ in f0] != ["UNREGISTERED_GENERATED"]:
            failures.append(f"an unregistered JSON beside a generated one must be the only finding on an unchanged tree, got {f0}")
        (repo / "out/gen.json").write_text('{"v":2}\n')
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-am", "hand edit"], check=True)
        if "HAND_EDIT_SUSPECTED" not in [c for c, _ in check_graph_drift(g, {"GENERATED_LOCATIONS": ["out"]}, repo)]:
            failures.append("a generated file changed without its inputs must be a suspected hand edit")
        (repo / "src/in.txt").write_text("input v2\n"); (repo / "out/gen.json").write_text('{"v":1}\n')
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-am", "stale"], check=True)
        if "STALE_GENERATED" not in [c for c, _ in check_graph_drift(g, {"GENERATED_LOCATIONS": ["out"]}, repo)]:
            failures.append("inputs changed while the generated file did not must be stale")
    if failures:
        print("trinity_build_receipt --self-check: FAIL")
        for f in failures:
            print("  " + f)
        return 1
    print("trinity_build_receipt --self-check: PASS (imports resolved and an unresolved one flagged; two runs deterministic; a changed fixture, differing runs, a hand edit, a stale artifact and an unregistered file each reported)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("command", nargs="?", choices=["graph", "receipt", "check"])
    ap.add_argument("--trinity-root")
    ap.add_argument("--inventory", default=INVENTORY)
    ap.add_argument("--out")
    ap.add_argument("--runs", type=int, default=2)
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args()
    if args.self_check:
        return self_check()
    try:
        spec = load_spec(ROOT / SPEC, GRAPH_REQUIRED) if (ROOT / SPEC).exists() else None
        matrix_spec = load_spec(ROOT / MATRIX_SPEC, ["FIXTURE_DIR", "FEATURE_FIXTURE", "NEGATIVES", "BACKENDS"])
    except ValueError as e:
        print(f"trinity_build_receipt: {e}", file=sys.stderr)
        return 2
    if args.command == "graph":
        if not spec or not args.trinity_root:
            print("trinity_build_receipt: graph needs the spec and --trinity-root DIR", file=sys.stderr)
            return 2
        inventory = json.loads((ROOT / args.inventory).read_text())
        g, findings = graph(pathlib.Path(args.trinity_root).resolve(), spec, inventory)
        out = ROOT / (args.out or GRAPH)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(g, indent=1) + "\n")
        for c, m in findings:
            print(f"  {c}: {m}")
        print(f"trinity_build_receipt graph: {len(g['modules'])} modules, {g['edge_count']} import edges {g['edge_resolution']}, {len(g['dependencies'])} pinned packages, "
              f"{len(g['outputs']['generated_tracked'])} generated tracked files, {len(findings)} finding(s) -> {out.relative_to(ROOT)}")
        return 1 if findings else 0
    if args.command == "receipt":
        t27c = t27c_path()
        if not t27c:
            print("trinity_build_receipt: t27c not built", file=sys.stderr)
            return 2
        rec, findings = receipt(args.runs, matrix_spec, t27c)
        out = ROOT / (args.out or RECEIPT)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(rec, indent=1) + "\n")
        for c, m in findings:
            print(f"  {c}: {m}")
        n = len(rec["fixtures"]) * len(rec["profile"]["backends"])
        print(f"trinity_build_receipt receipt: {args.runs} runs over {len(rec['fixtures'])} fixtures x {len(rec['profile']['backends'])} backends = {n} outputs, "
              f"{'deterministic' if rec['deterministic'] else str(len(rec['differing'])) + ' differing'}; compiler {rec['compiler']['t27c_version']} @ {rec['compiler']['compiler_revision'][:9]} -> {out.relative_to(ROOT)}")
        return 1 if findings else 0
    if args.command == "check":
        findings = []
        rec_path = ROOT / RECEIPT
        if rec_path.exists():
            findings += check_receipt(json.loads(rec_path.read_text()), matrix_spec, t27c_path())
        else:
            findings.append(("MISSING", f"{RECEIPT} does not exist; run receipt"))
        g_path = ROOT / GRAPH
        if g_path.exists() and spec:
            g = json.loads(g_path.read_text())
            if g.get("sha") != spec["PINNED_REVISION"]:
                findings.append(("PIN_MISMATCH", f"the graph is of {str(g.get('sha'))[:9]}, the spec pins {spec['PINNED_REVISION'][:9]}"))
            if g.get("findings"):
                findings.append(("GRAPH_FINDINGS", f"the graph records {len(g['findings'])} finding(s): {g['findings'][0]['code']}"))
            if args.trinity_root:
                findings += check_graph_drift(g, spec, pathlib.Path(args.trinity_root).resolve())
        else:
            findings.append(("MISSING", f"{GRAPH} or {SPEC} does not exist"))
        for c, m in findings:
            print(f"  {c}: {m}")
        if findings:
            print(f"trinity_build_receipt check: {len(findings)} finding(s)")
            return 1
        print("trinity_build_receipt check: OK -- the receipt reproduces (compiler sources, Cargo.lock, fixtures and a fresh generation all match) and the graph is of the pinned revision" + (" with no generated-file drift in the consumer" if args.trinity_root else ""))
        return 0
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
