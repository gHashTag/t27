#!/usr/bin/env python3
"""Generate .github/workflows/<ID>.yml from its card, specs/ci/gates/<ID>.t27.

WHY THIS EXISTS. Epic #5933, phase P1 (#5954). A workflow is machinery a
resolver parses, so by the own-language-first rule in CLAUDE.md it is generated
from our own source and never hand-written into a second home for the truth.
The card is the source; this file is the only thing that writes the YAML.

WHAT IT DOES, IN ORDER, FOR EVERY CARD -- and it refuses at the first step that
does not hold:

  1. `t27c test-report` on a scratch copy of the card with one test spliced in
     per law of specs/ci/schema.t27 (its LAWS and LAW_CHECKS). Refused when the
     report is BLOCKED, when any test FAILs, or when a law's test did not run.
  2. `t27c parse --json` on the card. Refused when the parse fails, when the
     module is not `ci_gate_<ID, - as _>` (a lost `module` line parses, and
     test-report passes it, so this is the only guard there), when the card does
     not `use ci::schema`, or when its consts are not exactly the schema's
     FIELDS with the schema's types.
  3. Render YAML.
  4. `yaml.safe_load` the YAML and read the card's fields back out of it. Refused
     when any field reads back different.
  5. Write it -- or, with --check, compare it byte for byte with the file on
     disk.

MODES
  (none)        write every generated workflow
  --check       write nothing; exit 1 when any file differs from its card, when
                a generated file's card is gone, or when a card is refused
  --self-check  run the negative controls in a temp tree: a hand-edited YAML, a
                law bent, a BLOCKED card, the `;`-alone trap and a lost module
                line must each be refused, and an untouched tree must pass

EXIT CODES. 0 clean. 1 refused, or a file differs. 2 COULD NOT RUN: no t27c,
no zig for test-report, no pyyaml, no schema, no cards. A green that means
"nobody looked" is the failure this repository has written down more than once.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

SCHEMA = "specs/ci/schema.t27"
SCHEMA_MODULE = "ci_schema"
SCHEMA_USE = "ci::schema"
CARDS = "specs/ci/gates"
WORKFLOWS = ".github/workflows"
TOOL = "tools/ci/gen_workflows.py"
MARK = "# GENERATED from specs/ci/gates/"


class Refused(Exception):
    """The card, or what it generates, does not hold. Exit 1."""


class CouldNotRun(Exception):
    """A precondition is absent, so nothing was measured. Exit 2."""


# ---------------------------------------------------------------- t27c

def run(cmd: list[str], timeout: int = 300) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except FileNotFoundError as e:
        raise CouldNotRun(f"{cmd[0]}: {e}") from e
    except subprocess.TimeoutExpired as e:
        raise CouldNotRun(f"{' '.join(cmd)}: timed out after {timeout}s") from e


def parse_json(t27c: str, path: str) -> dict:
    p = run([t27c, "parse", "--json", path])
    if p.returncode != 0:
        msg = (p.stderr or p.stdout).strip().splitlines()
        raise Refused(f"{path}: t27c parse failed: {msg[-1] if msg else p.returncode}")
    try:
        return json.loads(p.stdout)
    except json.JSONDecodeError as e:
        raise Refused(f"{path}: t27c parse --json printed no JSON ({e})") from e


def test_report(t27c: str, path: str, specs_dir: str, label: str = "") -> dict[str, str]:
    """Every test's verdict, by name. test-report exits 0 whatever it finds, so
    its text is the answer. `label` names the file in messages when `path` is a
    scratch copy of it."""
    label = label or path
    p = run([t27c, "test-report", path, "--specs-dir", specs_dir, "--verbose"])
    text = p.stdout + p.stderr
    for line in text.splitlines():
        m = re.match(r"\s*BLOCKED\s+(.*)", line)
        if m:
            if "zig not on PATH" in m.group(1):
                raise CouldNotRun(f"{label}: test-report BLOCKED: {m.group(1)}")
            raise Refused(f"{label}: test-report BLOCKED: {m.group(1)[:300]}")
    verdicts: dict[str, str] = {}
    for line in text.splitlines():
        m = re.match(r"\s*(pass|FAIL)\s+([A-Za-z_]\w*)\s*$", line)
        if m:
            verdicts[m.group(2)] = m.group(1)
    m = re.search(r"^\s*tests\s+(\d+)\s*$", text, re.M)
    if p.returncode != 0 or not m:
        raise CouldNotRun(f"{label}: test-report gave no summary (exit {p.returncode}): "
                          f"{text.strip()[-300:]}")
    if int(m.group(1)) != len(verdicts):
        raise CouldNotRun(f"{label}: test-report counted {m.group(1)} tests and named "
                          f"{len(verdicts)}")
    return verdicts


# ---------------------------------------------------------------- reading a card

def literal(node: dict, where: str):
    if node["kind"] != "ExprLiteral":
        raise Refused(f"{where}: not a literal ({node['kind']})")
    return node["value"], node["extra_kind"]


def const_value(decl: dict, ty: str, where: str):
    """The value of one card const, checked against the schema's type."""
    declared = decl["extra_type"]
    kids = decl["children"]
    if len(kids) != 1:
        raise Refused(f"{where}: expected one initializer")
    if ty == "[]str":
        m = re.fullmatch(r"\[(\d+)\]str", declared)
        if not m:
            raise Refused(f"{where}: declared {declared}, the schema says {ty}")
        if kids[0]["kind"] != "ExprArrayLiteral":
            raise Refused(f"{where}: not an array literal")
        out = []
        for i, k in enumerate(kids[0]["children"]):
            v, kind = literal(k, f"{where}[{i}]")
            if kind != "string":
                raise Refused(f"{where}[{i}]: not a string")
            out.append(text_value(v, f"{where}[{i}]"))
        if len(out) != int(m.group(1)):
            raise Refused(f"{where}: declared [{m.group(1)}], holds {len(out)}")
        return out
    if declared != ty:
        raise Refused(f"{where}: declared {declared}, the schema says {ty}")
    v, kind = literal(kids[0], where)
    if ty == "str":
        if kind != "string":
            raise Refused(f"{where}: not a string")
        return text_value(v, where)
    if ty == "bool":
        if v not in ("true", "false"):
            raise Refused(f"{where}: {v!r} is not a bool")
        return v == "true"
    if ty == "u32":
        if not re.fullmatch(r"\d+", v):
            raise Refused(f"{where}: {v!r} is not a u32")
        return int(v)
    if ty == "f64":
        if not re.fullmatch(r"\d+(\.\d+)?", v):
            raise Refused(f"{where}: {v!r} is not an f64")
        return float(v)
    raise Refused(f"{where}: the schema names a type this generator does not know: {ty}")


def text_value(v: str, where: str) -> str:
    # A backslash would be read once by the parser and once more by YAML; a
    # card does not need one, so it may not hold one. ASCII is law L3.
    if "\\" in v:
        raise Refused(f"{where}: a backslash in a card string")
    if not v.isascii():
        raise Refused(f"{where}: not ASCII (L3)")
    return v


def str_array(module: dict, name: str) -> list[str]:
    for c in module["children"]:
        if c["kind"] == "ConstDecl" and c["name"] == name:
            return const_value(c, "[]str", f"{SCHEMA}: {name}")
    raise CouldNotRun(f"{SCHEMA}: no const {name}")


class Schema:
    def __init__(self, root: str, t27c: str):
        path = os.path.join(root, SCHEMA)
        if not os.path.isfile(path):
            raise CouldNotRun(f"{SCHEMA}: not found under {root}")
        verdicts = test_report(t27c, path, os.path.join(root, "specs"))
        bad = sorted(n for n, v in verdicts.items() if v != "pass")
        if not verdicts or bad:
            raise Refused(f"{SCHEMA}: its own tests do not pass: {bad or 'none ran'}")
        mod = parse_json(t27c, path)
        if mod.get("name") != SCHEMA_MODULE:
            raise Refused(f"{SCHEMA}: module is {mod.get('name')!r}, not {SCHEMA_MODULE}")
        self.path = path
        self.fields: dict[str, str] = {}
        for entry in str_array(mod, "FIELDS"):
            name, ty = entry.split(" ", 1)
            self.fields[name] = ty
        self.laws = str_array(mod, "LAWS")
        self.checks = str_array(mod, "LAW_CHECKS")
        if len(self.laws) != len(self.checks) or not self.laws:
            raise Refused(f"{SCHEMA}: LAWS and LAW_CHECKS do not line up")
        self.own_names = {c["name"] for c in mod["children"] if c.get("name")}


def module_name_for(card_id: str) -> str:
    return "ci_gate_" + card_id.replace("-", "_")


def read_card(t27c: str, schema: Schema, path: str) -> dict:
    card_id = os.path.basename(path)[: -len(".t27")]
    mod = parse_json(t27c, path)
    want = module_name_for(card_id)
    if mod.get("name") != want:
        raise Refused(
            f"{path}: module is {mod.get('name')!r}, not {want!r}. A lost `module` "
            f"line still parses and still passes test-report; a lone `;` before it "
            f"is the usual way to lose it.")
    uses = [c["value"] for c in mod["children"] if c["kind"] == "UseDecl"]
    if uses != [SCHEMA_USE]:
        raise Refused(f"{path}: must `use {SCHEMA_USE};` and nothing else, has {uses}")
    card: dict = {}
    tests: list[str] = []
    for c in mod["children"]:
        kind, name = c["kind"], c["name"]
        if kind == "UseDecl":
            continue
        if kind == "TestBlock":
            if name.startswith("law_"):
                raise Refused(f"{path}: test {name}: the law_ prefix belongs to the schema")
            tests.append(name)
            continue
        if kind != "ConstDecl":
            raise Refused(f"{path}: a {kind} `{name}`. A card holds data and its own "
                          f"tests; logic lives in {SCHEMA}.")
        if name not in schema.fields:
            raise Refused(f"{path}: `{name}` is not a field of {SCHEMA}")
        if not c["extra_pub"]:
            raise Refused(f"{path}: `{name}` must be pub")
        if name in card:
            raise Refused(f"{path}: `{name}` declared twice")
        card[name] = const_value(c, schema.fields[name], f"{path}: {name}")
    missing = [f for f in schema.fields if f not in card]
    if missing:
        raise Refused(f"{path}: missing field(s) {missing}")
    if not tests:
        raise Refused(f"{path}: no test of its own (L4)")
    if card["ID"] != card_id:
        raise Refused(f"{path}: ID is {card['ID']!r}, the file is {card_id}.t27")
    card["_tests"] = tests
    return card


def run_laws(t27c: str, schema: Schema, path: str, tests: list[str]) -> None:
    """Step 1: the card's own tests and one test per law, compiled and run."""
    card_id = os.path.basename(path)[: -len(".t27")]
    with tempfile.TemporaryDirectory(prefix="ci-card-") as tmp:
        specs = os.path.join(tmp, "specs")
        os.makedirs(os.path.join(specs, "ci", "gates"))
        shutil.copyfile(schema.path, os.path.join(specs, "ci", "schema.t27"))
        with open(path, encoding="utf-8") as f:
            text = f.read()
        laws = "".join(f"\ntest law_{law} {{ {check} }}\n"
                       for law, check in zip(schema.laws, schema.checks))
        scratch = os.path.join(specs, "ci", "gates", card_id + ".t27")
        with open(scratch, "w", encoding="utf-8") as f:
            f.write(text.rstrip("\n") + "\n" + laws)
        verdicts = test_report(t27c, scratch, specs, label=path)
    failed = sorted(n for n, v in verdicts.items() if v != "pass")
    if failed:
        raise Refused(f"{path}: FAIL {', '.join(failed)}")
    expected = set(tests) | {f"law_{law}" for law in schema.laws}
    if set(verdicts) != expected:
        raise Refused(f"{path}: tests that ran do not match the tests asked for: "
                      f"missing {sorted(expected - set(verdicts))}, "
                      f"extra {sorted(set(verdicts) - expected)}")


# ---------------------------------------------------------------- rendering

def scalar(v) -> str:
    """A YAML scalar that reads back as exactly `v`: plain when it does,
    single-quoted when it does not."""
    import yaml
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if v and v == v.strip():
        try:
            if yaml.safe_load(f"k: {v}\n") == {"k": v}:
                return v
        except yaml.YAMLError:
            pass
    return "'" + v.replace("'", "''") + "'"


def flow_item(v: str) -> str:
    if re.fullmatch(r"[A-Za-z0-9._/-]+", v):
        return v
    return "'" + v.replace("'", "''") + "'"


def with_inputs(card: dict, uses: str) -> list[tuple[str, object]]:
    """The `with:` inputs of the step whose STEP_USES is `uses`, in card order.
    A key given twice is one multi-line value; a value of digits only is an
    integer."""
    keys: list[str] = []
    vals: dict[str, list[str]] = {}
    for u, k, v in zip(card["WITH_USES"], card["WITH_KEY"], card["WITH_VALUE"]):
        if u != uses:
            continue
        if k not in vals:
            keys.append(k)
            vals[k] = []
        vals[k].append(v)
    out: list[tuple[str, object]] = []
    for k in keys:
        vs = vals[k]
        if len(vs) > 1:
            out.append((k, "\n".join(vs) + "\n"))
        elif re.fullmatch(r"\d+", vs[0]):
            out.append((k, int(vs[0])))
        else:
            out.append((k, vs[0]))
    return out


def render(card: dict) -> str:
    cid = card["ID"]
    out = [
        f"{MARK}{cid}.t27 by {TOOL}.",
        "# Do not edit this file: edit the card, then run",
        f"#   python3 {TOOL}",
        "# The card carries the reasons. `--check` (.github/workflows/ci-cards.yml) fails",
        "# on any byte here that differs from what the card generates.",
        f"name: {scalar(card['NAME'])}",
        "",
        "on:",
    ]
    paths = card["PATHS"]

    def path_lines() -> list[str]:
        if not paths:
            return []
        return ["    paths:"] + [f"      - {scalar(p)}" for p in paths]

    if card["ON_PULL_REQUEST"]:
        out.append("  pull_request:")
        out += path_lines()
    if card["ON_PUSH"]:
        out.append("  push:")
        if card["PUSH_BRANCHES"]:
            out.append("    branches: [" + ", ".join(flow_item(b) for b in card["PUSH_BRANCHES"]) + "]")
        out += path_lines()
    if card["ON_DISPATCH"]:
        out.append("  workflow_dispatch:")
    if card["PERMISSIONS"]:
        out += ["", "permissions:"]
        for entry in card["PERMISSIONS"]:
            scope, level = entry.split(": ", 1)
            out.append(f"  {scope}: {level}")
    if card["CONCURRENCY_GROUP"]:
        out += ["", "concurrency:",
                f"  group: {scalar(card['CONCURRENCY_GROUP'])}",
                f"  cancel-in-progress: {scalar(card['CANCEL_IN_PROGRESS'])}"]
    out += ["", "jobs:", f"  {card['JOB_ID']}:"]
    if card["JOB_NAME"]:
        out.append(f"    name: {scalar(card['JOB_NAME'])}")
    out.append(f"    runs-on: {scalar(card['RUNS_ON'])}")
    if card["TIMEOUT_MIN"]:
        out.append(f"    timeout-minutes: {card['TIMEOUT_MIN']}")
    out.append("    steps:")
    for i, (name, uses, cmd) in enumerate(zip(card["STEP_NAME"], card["STEP_USES"], card["RUN"])):
        if i:
            out.append("")
        lines: list[str] = []
        if name:
            lines.append(f"name: {scalar(name)}")
        if uses:
            lines.append(f"uses: {scalar(uses)}")
            inputs = with_inputs(card, uses)
            if inputs:
                lines.append("with:")
                for k, v in inputs:
                    if isinstance(v, str) and "\n" in v:
                        lines.append(f"  {k}: |")
                        lines += [f"    {x}" for x in v.rstrip("\n").split("\n")]
                    else:
                        lines.append(f"  {k}: {scalar(v)}")
        if cmd:
            lines.append(f"run: {scalar(cmd)}")
        out.append("      - " + lines[0])
        out += ["        " + x for x in lines[1:]]
    return "\n".join(out) + "\n"


# ---------------------------------------------------------------- reading it back

def read_back(text: str, where: str) -> dict:
    """Step 4: the card's fields, read out of the YAML by a second reader."""
    import yaml
    doc = yaml.safe_load(text)

    def keys(d, allowed, at):
        extra = set(d) - set(allowed)
        if extra:
            raise Refused(f"{where}: {at} carries keys no card field writes: {sorted(map(str, extra))}")

    keys(doc, ["name", True, "permissions", "concurrency", "jobs"], "the document")
    on = doc.get(True, doc.get("on"))  # YAML 1.1 reads the key `on` as boolean true
    if not isinstance(on, dict):
        raise Refused(f"{where}: no `on:` mapping")
    keys(on, ["pull_request", "push", "workflow_dispatch"], "on:")
    pr = on.get("pull_request") or {}
    push = on.get("push") or {}
    keys(pr, ["paths"], "on.pull_request")
    keys(push, ["branches", "paths"], "on.push")
    if "pull_request" in on and "push" in on and pr.get("paths") != push.get("paths"):
        raise Refused(f"{where}: pull_request and push read different paths")
    got = {
        "NAME": doc["name"],
        "ON_PULL_REQUEST": "pull_request" in on,
        "ON_PUSH": "push" in on,
        "ON_DISPATCH": "workflow_dispatch" in on,
        "PUSH_BRANCHES": push.get("branches", []),
        "PATHS": pr.get("paths", push.get("paths", [])),
        "PERMISSIONS": [f"{k}: {v}" for k, v in (doc.get("permissions") or {}).items()],
        "CONCURRENCY_GROUP": (doc.get("concurrency") or {}).get("group", ""),
        "CANCEL_IN_PROGRESS": (doc.get("concurrency") or {}).get("cancel-in-progress", False),
    }
    if "concurrency" in doc:
        keys(doc["concurrency"], ["group", "cancel-in-progress"], "concurrency")
    jobs = doc["jobs"]
    if len(jobs) != 1:
        raise Refused(f"{where}: {len(jobs)} jobs, a card renders one")
    (job_id, job), = jobs.items()
    keys(job, ["name", "runs-on", "timeout-minutes", "steps"], f"jobs.{job_id}")
    got.update({
        "JOB_ID": job_id,
        "JOB_NAME": job.get("name", ""),
        "RUNS_ON": job["runs-on"],
        "TIMEOUT_MIN": job.get("timeout-minutes", 0),
        "STEP_NAME": [], "STEP_USES": [], "RUN": [],
        "WITH_USES": [], "WITH_KEY": [], "WITH_VALUE": [],
    })
    for n, step in enumerate(job["steps"]):
        keys(step, ["name", "uses", "with", "run"], f"step {n}")
        got["STEP_NAME"].append(step.get("name", ""))
        got["STEP_USES"].append(step.get("uses", ""))
        got["RUN"].append(step.get("run", ""))
        for k, v in (step.get("with") or {}).items():
            vs = v.rstrip("\n").split("\n") if isinstance(v, str) and "\n" in v else [str(v)]
            for x in vs:
                got["WITH_USES"].append(step["uses"])
                got["WITH_KEY"].append(k)
                got["WITH_VALUE"].append(x)
    return got


def generate(t27c: str, schema: Schema, path: str) -> tuple[str, str]:
    """(workflow path, workflow text) for one card, or Refused."""
    card_id = os.path.basename(path)[: -len(".t27")]
    # Step 1 needs the card's own test names, which only a parse gives; the
    # parse is read here but judged in step 2, after test-report has spoken.
    try:
        tests = [c["name"] for c in parse_json(t27c, path)["children"]
                 if c["kind"] == "TestBlock"]
    except Refused:
        tests = []
    run_laws(t27c, schema, path, tests)
    card = read_card(t27c, schema, path)
    text = render(card)
    got = read_back(text, f"{path}: generated {card['FILE']}")
    wrong = [f for f in got if got[f] != card[f]]
    if wrong:
        raise Refused(f"{path}: the YAML reads back different on {wrong}: "
                      + "; ".join(f"{f} card={card[f]!r} yaml={got[f]!r}" for f in wrong))
    if not text.isascii():
        raise Refused(f"{path}: the generated YAML is not ASCII (L3)")
    if card["FILE"] != f"{WORKFLOWS}/{card_id}.yml":
        raise Refused(f"{path}: FILE is {card['FILE']!r}")
    return card["FILE"], text


# ---------------------------------------------------------------- modes

def find_t27c(root: str, given: str | None) -> str:
    cand = given or os.environ.get("T27C_BIN") or os.path.join(root, "target/release/t27c")
    if not os.path.isabs(cand) and not os.path.exists(cand):
        cand = os.path.join(root, cand)
    if not (os.path.isfile(cand) and os.access(cand, os.X_OK)):
        raise CouldNotRun(f"t27c not found at {cand}: cargo build --release -p t27c, "
                          f"or pass --t27c / set T27C_BIN")
    return os.path.abspath(cand)


def main_run(root: str, t27c_arg: str | None, check: bool) -> int:
    try:
        import yaml  # noqa: F401
    except ImportError:
        print("gen_workflows: pyyaml is not installed. Exit 2 = COULD NOT RUN.", file=sys.stderr)
        return 2
    root = os.path.abspath(root)
    try:
        t27c = find_t27c(root, t27c_arg)
        schema = Schema(root, t27c)
        card_dir = os.path.join(root, CARDS)
        cards = sorted(f for f in os.listdir(card_dir) if f.endswith(".t27")) \
            if os.path.isdir(card_dir) else []
        if not cards:
            raise CouldNotRun(f"no card under {CARDS}/: a population of zero would print as clean")
    except CouldNotRun as e:
        print(f"gen_workflows: COULD NOT RUN: {e}", file=sys.stderr)
        return 2
    except Refused as e:
        print(f"gen_workflows: REFUSED: {e}", file=sys.stderr)
        return 1

    refused: list[str] = []
    differs: list[str] = []
    written: list[str] = []
    generated: set[str] = set()
    for name in cards:
        path = os.path.join(CARDS, name)
        try:
            rel, text = generate(t27c, schema, os.path.join(root, path))
        except CouldNotRun as e:
            print(f"gen_workflows: COULD NOT RUN: {e}", file=sys.stderr)
            return 2
        except Refused as e:
            refused.append(str(e).replace(root + os.sep, ""))
            continue
        generated.add(rel)
        dest = os.path.join(root, rel)
        old = open(dest, encoding="utf-8").read() if os.path.isfile(dest) else None
        if old == text:
            continue
        if check:
            differs.append(rel)
        else:
            with open(dest, "w", encoding="utf-8") as f:
                f.write(text)
            written.append(rel)

    orphans = []
    wdir = os.path.join(root, WORKFLOWS)
    for f in sorted(os.listdir(wdir)) if os.path.isdir(wdir) else []:
        rel = f"{WORKFLOWS}/{f}"
        if rel in generated:
            continue
        with open(os.path.join(wdir, f), encoding="utf-8", errors="replace") as fh:
            first = fh.readline()
        if first.startswith(MARK) and not refused_card_for(rel, refused):
            orphans.append(rel)

    print(f"cards read                 {len(cards)}")
    print(f"workflows generated        {len(generated)}")
    print(f"cards refused              {len(refused)}")
    if check:
        print(f"workflows differing        {len(differs)}")
    else:
        print(f"workflows written          {len(written)}")
    print(f"generated files, no card   {len(orphans)}")
    print(f"  {len(generated)} + {len(refused)} = {len(generated) + len(refused)}"
          f"  (must equal {len(cards)})")
    for r in refused:
        print(f"REFUSED  {r}")
    for d in differs:
        print(f"DIFFERS  {d}: hand-edited, or its card moved without it. Edit the card, "
              f"then run `python3 {TOOL}`.")
    for w in written:
        print(f"WROTE    {w}")
    for o in orphans:
        print(f"ORPHAN   {o}: says it is generated, and no card generates it")
    if refused or differs or orphans or len(generated) + len(refused) != len(cards):
        return 1
    print("CLEAN: every workflow matches its card." if check else "done.")
    return 0


def refused_card_for(rel: str, refused: list[str]) -> bool:
    stem = os.path.basename(rel)[: -len(".yml")]
    return any(r.startswith(f"{CARDS}/{stem}.t27:") for r in refused)


# ---------------------------------------------------------------- self-check

def self_check(root: str, t27c_arg: str | None) -> int:
    """Each control damages a copy of the tree and must be refused for the
    reason it names. The untouched copy must pass first, or every refusal
    below proves nothing."""
    root = os.path.abspath(root)
    try:
        t27c = find_t27c(root, t27c_arg)
    except CouldNotRun as e:
        print(f"gen_workflows: COULD NOT RUN: {e}", file=sys.stderr)
        return 2
    me = os.path.abspath(__file__)
    probe = "conflict-markers"
    card_rel = f"{CARDS}/{probe}.t27"
    wf_rel = f"{WORKFLOWS}/{probe}.yml"

    def tree(tmp: str) -> str:
        # The schema and one card: every control below damages that card or
        # its workflow, and --check over the real tree is a separate step.
        t = os.path.join(tmp, "t")
        os.makedirs(os.path.join(t, CARDS))
        os.makedirs(os.path.join(t, WORKFLOWS))
        shutil.copyfile(os.path.join(root, SCHEMA), os.path.join(t, SCHEMA))
        shutil.copyfile(os.path.join(root, card_rel), os.path.join(t, card_rel))
        return t

    def call(t: str, *args: str) -> tuple[int, str]:
        p = run([sys.executable, me, "--root", t, "--t27c", t27c, *args], timeout=900)
        return p.returncode, p.stdout + p.stderr

    def edit(t: str, rel: str, old: str, new: str) -> None:
        path = os.path.join(t, rel)
        with open(path, encoding="utf-8") as f:
            text = f.read()
        if old not in text:
            raise CouldNotRun(f"self-check: {rel} no longer holds {old!r}; the control "
                              f"must be re-aimed")
        with open(path, "w", encoding="utf-8") as f:
            f.write(text.replace(old, new, 1))

    controls = [
        ("(a) a hand-edited YAML", wf_rel, "timeout-minutes: 5", "timeout-minutes: 6",
         1, f"DIFFERS  {wf_rel}"),
        ("(b) a law bent: a pull_request gate with no concurrency group", card_rel,
         'CONCURRENCY_GROUP : str = "conflict-markers-${{ github.ref }}"',
         'CONCURRENCY_GROUP : str = ""', 1, "FAIL law_pr_has_group"),
        ("(c) a BLOCKED card: a test that calls a law the schema does not have", card_rel,
         "assert TOOLS.len == 1;", "assert no_such_law(ID);", 1, "test-report BLOCKED"),
        ("(d1) the `;`-alone trap: a lone `;` in the prose", card_rel,
         "\n\n; The self-check runs first", "\n;\n; The self-check runs first", 1,
         "test-report BLOCKED: codegen failed: parse error"),
        ("(d2) a lost module declaration", card_rel,
         "module ci_gate_conflict_markers;", "", 1, "module is '', not 'ci_gate_conflict_markers'"),
    ]
    bad = 0
    with tempfile.TemporaryDirectory(prefix="gen-workflows-self-check-") as tmp:
        base = tree(tmp)
        code, out = call(base)
        code2, out2 = call(base, "--check")
        ok = code == 0 and code2 == 0 and "CLEAN" in out2
        print(f"{'ok  ' if ok else 'BAD '} positive control: write then --check -> exit {code}, {code2}")
        if not ok:
            print(out + out2)
            return 1
        for i, (label, rel, old, new, want, needle) in enumerate(controls):
            t = os.path.join(tmp, f"c{i}")
            shutil.copytree(base, t)
            try:
                edit(t, rel, old, new)
            except CouldNotRun as e:
                print(f"gen_workflows: COULD NOT RUN: {e}", file=sys.stderr)
                return 2
            code, out = call(t, "--check")
            hit = code == want and needle in out
            bad += not hit
            last = (out.strip().splitlines() or [""])[-1]
            line = next((x for x in out.splitlines() if needle in x), last)
            print(f"{'ok  ' if hit else 'BAD '} {label}: exit {code}")
            print(f"       {line}"[:400])
    if bad:
        print(f"SELF-CHECK FAILED: {bad} control(s) not refused for the reason they name.")
        return 1
    print(f"SELF-CHECK OK: 1 positive and {len(controls)} negative controls behaved.")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", default=".", help="repository root (default: .)")
    ap.add_argument("--t27c", default=None, help="t27c binary (default: $T27C_BIN, "
                    "then target/release/t27c)")
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="compare, write nothing")
    mode.add_argument("--self-check", action="store_true", help="run the negative controls")
    a = ap.parse_args(argv)
    if a.self_check:
        return self_check(a.root, a.t27c)
    return main_run(a.root, a.t27c, a.check)


if __name__ == "__main__":
    sys.exit(main())
