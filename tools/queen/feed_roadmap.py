#!/usr/bin/env python3
"""Feed the swarm from the roadmap: one port task per hand-written file, stage by stage.

WHY THIS EXISTS

Measured 2026-09-27 on the live deployment: 0 bees of 4 lanes, and every tick
answering `refusal="nothing to choose"` against 497 candidates - every open
issue in this repository. The roadmap tab names eight stages (#4543-#4550), the
whole stack rewritten in `.t27`, and only stage 1 had a single task filed
against it. Stages 2-8 had none. The Queen files nothing herself, by design
(queen-tick.ts: "IT DOES NOT FILE ANYTHING"), so a stage nobody feeds is a stage
nobody works.

The same day showed the second half of the problem. A bee sees ONE checkout:
this repository. When #4813-#4832 sent bees to `src/hslm/cli.zig`,
`apps/queen/Package.swift` and `src/tri-api/session_store.zig` - files that live
in gHashTag/trinity - every one of them came back as an empty branch, and the
issues were held again. A task about another repository's file is only
workable if the file travels with the task.

WHAT IT FILES

One issue per source file, in the shape of the `Port tools/<file>.py` issues
that stage 1 already closed 23 of:

  * the source, VERBATIM, with a line-number gutter, inside the issue - the bee
    cannot see the repository it came from. The gutter is not decoration: the
    Queen reads acceptance criteria as the bullets under any heading line, and
    a heading is any line that starts with `#`. Python comments start with `#`
    and C comments continue with ` * `; behind a gutter neither can be read as
    a heading or a criterion.
  * the roadmap brief, `docs/ROADMAP_BRIEF.md`: the goal of the game, where this
    file sits in it, and how to rewrite code into `.t27`;
  * the language and the instruments, from `docs/BEE_TOOLBELT.md`;
  * one Boundary: the `.t27` the task creates.

WHAT IT REFUSES, AND SAYS SO

  stage 8                a decision, not a task: t27c has no interface target
  an unreadable repo     stages 3 and 4 live in private repositories; a run
                         without ROADMAP_READ_TOKEN reports them and moves on
  generated files        a header that says so, or a generated/vendored tree
  tests                  they become `test` blocks of the port, not ports
  no decision            a `todo!()` stub, or a `main` that only prints: no
                         branch, loop, comparison or arithmetic for a test
  0 or > MAX_FNS units   nothing to port / left for a change that can chunk
  too big to quote       MAX_LINES / MAX_CHARS
  an existing target     the `.t27` is on master, or an issue names it already
  anything task_shape refuses

Usage, from the repository root:

    python3 tools/queen/feed_roadmap.py --self-test
    python3 tools/queen/feed_roadmap.py --dry-run --limit 7
    python3 tools/queen/feed_roadmap.py --dry-run --stage 2 --limit 3
    python3 tools/queen/feed_roadmap.py --when-idle --limit 7 --runway auto
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))

REPO = "gHashTag/t27"
WORK = os.environ.get("T27_REPO_ROOT") or os.getcwd()
CACHE = os.environ.get("ROADMAP_CACHE") or os.path.join(tempfile.gettempdir(), "roadmap-sources")
OUTD = os.path.join(tempfile.gettempdir(), "queen-feed-roadmap")
BRIEF_DOC = "docs/ROADMAP_BRIEF.md"

# The ceilings of the other feeders, for the same reason: a task a bee can
# finish in one turn is a lane that frees again. Above them a file is counted
# and left for a change that can chunk it, never silently dropped.
MAX_FNS = 8
MAX_LINES = 400
MAX_CHARS = 16000

LANG = {
    ".py": "Python", ".sh": "Shell", ".ts": "TypeScript", ".mts": "TypeScript",
    ".js": "JavaScript", ".mjs": "JavaScript", ".rs": "Rust", ".go": "Go",
    ".zig": "Zig", ".c": "C", ".gleam": "Gleam", ".v": "Verilog", ".sv": "Verilog",
}
FENCE_LANG = {
    "Python": "python", "Shell": "sh", "TypeScript": "ts", "JavaScript": "js",
    "Rust": "rust", "Go": "go", "Zig": "zig", "C": "c", "Gleam": "gleam",
    "Verilog": "verilog",
}


@dataclass
class Stage:
    number: int
    goal: int
    title: str
    # (repository[@branch], [path prefixes in priority order]); "" matches
    # everything left. No branch means the repository's default branch.
    sources: list[tuple[str, list[str]]]
    exts: tuple[str, ...]
    target: str
    skip_dirs: tuple[str, ...] = ()
    refuse: str = ""


# The roadmap, as filed in #4543-#4550 and #4858. The goal issues are the authority; this
# is their machine-readable index. The prefixes put each stage's innermost ring
# first, which is the order its goal issue asks for.
STAGES = [
    Stage(1, 4543, "The swarm's own tools: Python and shell in t27",
          [("gHashTag/t27", ["tools/", "scripts/", "formal/", ""])],
          (".py", ".sh"), "t27c gen-rust"),
    # BrowserOS's default branch (`dev`) is upstream and has no trios/ at all.
    # The Queen runs from this branch: it is what the api.t27.ai service deploys
    # (Railway, trios-agent-server, read 2026-09-27). When that changes, this does.
    Stage(2, 4544, "The Queen and her bees: api.t27.ai",
          [("gHashTag/BrowserOS@fix/queen-worker-provider-and-prompt-size", [
              "trios/agent-server/apps/server/src/api/services/queen-",
              "trios/agent-server/apps/server/src/api/routes/queen-",
              "trios/agent-server/apps/server/src/",
          ])],
          (".ts", ".go"), "t27c gen-rust"),
    Stage(3, 4545, "The app and the bot: app.t27.ai backends",
          [("gHashTag/999-multibots-telegraf", [""])],
          (".ts", ".js", ".rs"), "t27c gen-rust (servers); UI stays for stage 8",
          skip_dirs=("components", "pages", "app", "ui", "public", "frontend", "web")),
    Stage(4, 4546, "VIBEE services on Fly.io",
          [("gHashTag/vibee-gleam", [""])],
          (".ts", ".js", ".gleam"), "t27c gen-rust"),
    Stage(5, 4547, "The Zig core: hand-written Zig becomes generated Zig",
          [("gHashTag/trinity", ["src/vsa", "src/vm", "src/", ""]),
           ("gHashTag/trios", [""]),
           ("gHashTag/t27-github-collab", [""])],
          (".zig", ".c"), "t27c gen (Zig), gen-c"),
    Stage(6, 4548, "trios Rust rings and trios-railway",
          [("gHashTag/trios", ["rings/", ""]),
           ("gHashTag/trios-railway", [""])],
          (".rs",), "t27c gen-rust"),
    Stage(7, 4549, "Silicon: Verilog from .t27",
          [("gHashTag/t27", ["fpga/", "rtl/", ""]),
           ("gHashTag/trinity", ["fpga/", ""])],
          (".v", ".sv"), "t27c gen-verilog",
          skip_dirs=("sim", "tb", "testbench", "formal")),
    Stage(8, 4550, "Interfaces: the open question", [], (), "undecided",
          refuse="t27c has no interface target yet; the goal asks for a decision first"),
    # The whole browser and every third-party package: opens after stage 8, and
    # its goal asks for two measurements and three decisions before any task.
    Stage(9, 4858, "Endgame: the browser and every dependency", [], (), "t27c gen-rust, gen-c",
          refuse="opens after stage 8; the goal asks for a measurement and decisions first"),
]

# THE RAID OF THE DAY. One sector per UTC day, in turn, is fed first and
# twice as often. gHashTag/trinity apps/website/src/lib/roadmapGame.ts
# computes the same sector from goals.json (stages 1-7 with nothing locking
# them) and draws it on the ROADMAP tab; qa/roadmap-game-contract.mjs pins
# that list to this one. Unlocking a stage there means adding it here.
RAID_STAGES = (1, 2, 5, 6, 7)


def raid_stage(day: int) -> int:
    """The raid sector for a UTC day number (days since the epoch)."""
    return RAID_STAGES[day % len(RAID_STAGES)]


# Trees nobody writes by hand, or that are not this project's to rewrite.
SKIP_DIRS = {
    "node_modules", "vendor", "vendored", "dist", "build", "out", "target", "generated",
    "gen", "var", "external", "third_party", "third-party", ".zig-cache", "zig-cache",
    "zig-out", "test", "tests", "__tests__", "testdata", "fixtures", "examples",
    "bootstrap", "specs", "outputs", "datasets", "test-ledger", "migrations", ".github",
    "contrib", "deps", "legacy", "archive", "__pycache__", "site-packages",
}
SKIP_FILE = re.compile(
    r"(^test_|_test\.(py|go|zig|rs)$|\.(test|spec)\.[mc]?[jt]sx?$|\.d\.ts$|\.min\.js$"
    r"|^conftest\.py$|^setup\.py$|_tb\.s?v$|^tb_.*\.s?v$)")
# "Generated from <spec>" too: gHashTag/trinity's src/vsa_simple/gen_vsa.zig
# says so in its first line, and it was filed beside vsa.zig, the same code.
GENERATED = re.compile(r"(?i)(auto-?generated|generated by|code generated|do not edit|@generated"
                       r"|generated from \S+\.(tri|t27|vibee)\b)")
SAFE_PATH = re.compile(r"^[A-Za-z0-9_./-]+$")

# A port carries the DECISIONS in a file, and a test asserts on them. A file
# with no branch, no loop, no comparison and no arithmetic has nothing a test
# can assert on: measured 2026-09-27, 6 of the first 14 tasks a dry run chose
# were `todo!()` stubs or a function that only prints. Read with comments and
# string literals removed, so a docstring saying "if" is not a decision.
DECISION = re.compile(r"\b(if|elif|else|match|switch|case|for|while|loop|and|or|not)\b"
                      r"|==|!=|<=|>=|&&|\|\||[\w)\]]\s*[-+*/%]\s*[\w(]"
                      # a builtin that decides: `max(lo, min(x, hi))` is a clamp
                      r"|\b(min|max|abs|sorted|any|all|filter)\s*\("
                      r"|\.(filter|find|some|every|includes|contains|startswith|endswith"
                      r"|startsWith|endsWith)\s*\(")
NOISE = {
    "#": re.compile(r'"""[\s\S]*?"""|\'\'\'[\s\S]*?\'\'\'|#[^\n]*|"(?:\\.|[^"\\\n])*"'
                    r"|'(?:\\.|[^'\\\n])*'"),
    "//": re.compile(r"/\*[\s\S]*?\*/|//[^\n]*|`(?:\\.|[^`\\])*`|\"(?:\\.|[^\"\\\n])*\""
                     r"|'(?:\\.|[^'\\\n])*'"),
}
COMMENT_STYLE = {"Python": "#", "Shell": "#"}


def has_decision(text: str, lang: str) -> bool:
    """True when the code, without its comments, strings and tests, decides something.

    The tests go too: a Rust `#[cfg(test)]` module or a Zig `test` block
    compares, and the code under test may still be `0` returned for any input
    (trinity's src/tri/gen_hash_sha256.zig is a SHA-256 of 32 zero bytes).
    """
    if lang == "Verilog":
        return True  # a module is wiring as much as logic; its criteria are its own
    if lang == "Rust":
        text = text.split("#[cfg(test)]")[0]
    elif lang == "Zig":
        text = re.sub(r"(?ms)^test\b.*?^\}", "", text)
    return bool(DECISION.search(NOISE[COMMENT_STYLE.get(lang, "//")].sub(" ", text)))

# What the parser takes at the top level, plus the words a name may not be.
KEYWORDS = {
    "const", "var", "fn", "enum", "struct", "test", "invariant", "bench", "pub", "use",
    "module", "return", "if", "else", "while", "for", "in", "switch", "match", "break",
    "continue", "true", "false", "null", "and", "or", "not", "type", "import",
}


# --------------------------------------------------------------------------
# Units: what a port must carry across, by name.
# --------------------------------------------------------------------------

@dataclass
class Unit:
    line: int
    signature: str
    name: str       # the name the .t27 must declare
    original: str   # the name in the source, for the issue text


def _t27_name(name: str) -> str:
    name = re.sub(r"[^A-Za-z0-9_]", "_", name)
    if name[:1].isdigit():
        name = "_" + name
    return name + "_" if name in KEYWORDS else name


def _unit(units: list[Unit], seen: set, line: int, text: str, original: str, owner: str = ""):
    name = _t27_name(f"{owner}_{original}" if owner else original)
    if name in seen:
        return
    seen.add(name)
    shown = f"{owner}.{original}" if owner else original
    units.append(Unit(line, text.strip()[:200], name, shown))


def units_python(text: str) -> list[Unit]:
    units, seen, owner = [], set(), ""
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^class\s+([A-Za-z_]\w*)", line):
            owner = m.group(1)
            continue
        if re.match(r"^\S", line) and not line.startswith(("def ", "async def ", "@", "#")):
            owner = ""
        if m := re.match(r"^(async\s+)?def\s+([A-Za-z_]\w*)\s*\(", line):
            owner = ""
            _unit(units, seen, i, line, m.group(2))
        elif owner and (m := re.match(r"^ {4}(async\s+)?def\s+([A-Za-z_]\w*)\s*\(", line)):
            _unit(units, seen, i, line, m.group(2).strip("_") or "call", owner)
    return units


def units_shell(text: str) -> list[Unit]:
    units, seen = [], set()
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^(?:function\s+)?([A-Za-z_][\w-]*)\s*\(\)\s*\{?\s*$", line):
            _unit(units, seen, i, line, m.group(1))
    # A script is a function too: its top level runs, and a port needs a name for it.
    _unit(units, seen, 1, "(the script's top level)", "main")
    return units


_TS_NOT_METHODS = {"if", "for", "while", "switch", "catch", "function", "return",
                   "constructor", "super", "else", "do", "try", "with"}


def units_ts(text: str) -> list[Unit]:
    units, seen, owner = [], set(), ""
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^(?:export\s+)?(?:default\s+)?(?:abstract\s+)?class\s+([A-Za-z_$][\w$]*)", line):
            owner = m.group(1)
            continue
        if line.startswith("}"):
            owner = ""
        if m := re.match(r"^(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s*\*?\s*"
                         r"([A-Za-z_$][\w$]*)\s*[<(]", line):
            _unit(units, seen, i, line, m.group(1))
        elif m := re.match(r"^(?:export\s+)?(?:const|let)\s+([A-Za-z_$][\w$]*)\s*(?::[^=]+)?="
                           r"\s*(?:async\s+)?(?:\([^)]*\)|[A-Za-z_$][\w$]*)\s*(?::\s*[^=]+)?=>", line):
            _unit(units, seen, i, line, m.group(1))
        elif owner and (m := re.match(
                r"^ {2}(?:(?:public|private|protected|static|async|readonly|override)\s+)*"
                r"([A-Za-z_$][\w$]*)\s*(?:<[^>]*>)?\(.*\)\s*(?::\s*[^{;]+)?\{\s*$", line)):
            if m.group(1) not in _TS_NOT_METHODS:
                _unit(units, seen, i, line, m.group(1), owner)
    return units


def units_rust(text: str) -> list[Unit]:
    units, seen, owner = [], set(), ""
    for i, line in enumerate(text.split("\n"), 1):
        if re.match(r"^#\[cfg\(test\)\]", line):
            break
        if m := re.match(r"^impl(?:<[^>]*>)?\s+(?:[\w:<>, ]+\s+for\s+)?([A-Za-z_]\w*)", line):
            owner = m.group(1)
            continue
        if line.startswith("}"):
            owner = ""
        fn = r"(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?" \
             r"(?:extern\s+\"[^\"]*\"\s+)?fn\s+([A-Za-z_]\w*)"
        if m := re.match("^" + fn, line):
            _unit(units, seen, i, line, m.group(1))
        elif owner and (m := re.match("^ {4}" + fn, line)):
            _unit(units, seen, i, line, m.group(1), owner)
    return units


def units_zig(text: str) -> list[Unit]:
    units, seen, owner = [], set(), ""
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^(?:pub\s+)?const\s+([A-Za-z_]\w*)\s*=\s*(?:extern\s+|packed\s+)?struct\b", line):
            owner = m.group(1)
            continue
        if line.startswith("}"):
            owner = ""
        fn = r"(?:pub\s+)?(?:export\s+)?(?:inline\s+)?fn\s+([A-Za-z_]\w*)\s*\("
        if m := re.match("^" + fn, line):
            _unit(units, seen, i, line, m.group(1))
        elif owner and (m := re.match("^ {4}" + fn, line)):
            _unit(units, seen, i, line, m.group(1), owner)
    return units


def units_go(text: str) -> list[Unit]:
    units, seen = [], set()
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^func\s+(?:\(\s*\w*\s*\*?\s*([A-Za-z_]\w*)[^)]*\)\s*)?([A-Za-z_]\w*)\s*[\[(]", line):
            _unit(units, seen, i, line, m.group(2), m.group(1) or "")
    return units


def units_c(text: str) -> list[Unit]:
    units, seen = [], set()
    lines = text.split("\n")
    for i, line in enumerate(lines, 1):
        m = re.match(r"^(?!static\s+inline\b)(?:[A-Za-z_][\w]*[\s\*]+)+([A-Za-z_]\w*)\s*\([^;]*\)\s*\{?\s*$", line)
        if not m or m.group(1) in ("if", "for", "while", "switch", "return", "sizeof"):
            continue
        nxt = lines[i].strip() if i < len(lines) else ""
        if line.rstrip().endswith("{") or nxt.startswith("{"):
            _unit(units, seen, i, line, m.group(1))
    return units


def units_gleam(text: str) -> list[Unit]:
    units, seen = [], set()
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^(?:pub\s+)?fn\s+([A-Za-z_]\w*)\s*\(", line):
            _unit(units, seen, i, line, m.group(1))
    return units


def units_verilog(text: str) -> list[Unit]:
    units, seen = [], set()
    for i, line in enumerate(text.split("\n"), 1):
        if m := re.match(r"^\s*module\s+([A-Za-z_]\w*)", line):
            _unit(units, seen, i, line, m.group(1))
    return units


EXTRACT = {
    "Python": units_python, "Shell": units_shell, "TypeScript": units_ts,
    "JavaScript": units_ts, "Rust": units_rust, "Zig": units_zig, "Go": units_go,
    "C": units_c, "Gleam": units_gleam, "Verilog": units_verilog,
}


# --------------------------------------------------------------------------
# Where a port lands, and what the issue says.
# --------------------------------------------------------------------------

def target_for(repo: str, path: str) -> str:
    """`specs/port/<repo>/<path>.t27`; this repository's own files keep stage 1's
    existing shape, `specs/port/tools/<file>.t27`, so its closed issues still count."""
    stem = os.path.splitext(path)[0]
    if repo == REPO:
        return f"specs/port/{stem}.t27"
    return f"specs/port/{repo.split('/')[1].lower()}/{stem}.t27"


def title_for(repo: str, path: str, lang: str, units: list[Unit], target: str) -> str:
    what = "modules" if lang == "Verilog" else "functions"
    what = what[:-1] if len(units) == 1 else what
    where = path if repo == REPO else f"{repo}:{path}"
    return f"Port {where} ({lang}, {len(units)} {what}) to {target}"


def gutter(text: str) -> str:
    """Every line behind a number. See the module docstring: without it a Python
    comment is a heading and a C comment line is a bullet, to the Queen."""
    lines = text.rstrip("\n").split("\n")
    width = len(str(len(lines)))
    return "\n".join(f"{n:>{width}}| {line}".rstrip() for n, line in enumerate(lines, 1))


def fence(text: str) -> str:
    """A code fence longer than any backtick run inside the text."""
    longest = max((len(m) for m in re.findall(r"`+", text)), default=0)
    return "`" * max(3, longest + 1)


def section(text: str, heading: str) -> str:
    """The lines under one `## heading`, up to the next `## `."""
    out, inside = [], False
    for line in text.split("\n"):
        if line.startswith("## "):
            inside = line.strip() == heading
            continue
        if inside:
            out.append(line)
    return "\n".join(out).strip("\n")


def roadmap_brief(root: str) -> tuple[str, str]:
    """(the goal of the game, how to rewrite code), from docs/ROADMAP_BRIEF.md."""
    try:
        with open(os.path.join(root, BRIEF_DOC), encoding="utf-8") as handle:
            text = handle.read()
    except OSError:
        return "", ""
    return section(text, "## The goal of the game"), section(text, "## How to rewrite code into .t27")


def build(stage: Stage, repo: str, sha: str, path: str, text: str, lang: str,
          units: list[Unit], root: str, belt: str) -> tuple[str, str]:
    target = target_for(repo, path)
    title = title_for(repo, path, lang, units, target)
    names = [u.name for u in units]
    n = len(units)
    lines = text.count("\n") + 1
    goal, howto = roadmap_brief(root)
    alt = "|".join(names)
    kind = "module" if lang == "Verilog" else "function"
    in_tree = repo == REPO

    listed = []
    for k, u in enumerate(units, 1):
        word = "module" if lang == "Verilog" else "fn"
        renamed = "" if u.name == u.original else f" - write it as `{word} {u.name}`"
        listed.append(f"{k}. line {u.line} - `{u.signature}`{renamed}")

    if lang == "Verilog":
        # One Verilog module per file (`pick` refuses the rest): `t27c gen-verilog`
        # emits one module per `.t27`, named by its `module` line.
        declared = [f"- 2. `grep -cE '^\\s*(pub )?module {alt}\\b' {target}` prints `1` - the "
                    "`.t27` module carries the original's name"]
        gen = [f"- 3. `t27c gen-verilog {target} | grep -cE '^module {alt} ?\\('` prints `1` - "
               "the generated Verilog is a module of the same name"]
        tests_needed = 1
    else:
        declared = [f"- 2. `grep -cE '^\\s*(pub )?fn ({alt})\\(' {target}` prints `{n}` - every "
                    f"{kind} above is ported under its own name"]
        tests_needed = n
        gen = [f"- 3. `t27c gen {target} > /tmp/t27-gen.zig && grep -c 'not yet implemented' "
               f"/tmp/t27-gen.zig` prints `0`, and `t27c gen {target} | wc -l` prints more than "
               "`12` - an absent or empty file makes the grep print `0` on its own, so both "
               "halves are required"]
    criteria = [
        "## Acceptance criteria",
        "",
        f"- 1. `test -f {target} && echo present` prints `present` (today the file does not exist)",
        *declared,
        *gen,
        f"- 4. `t27c spec-status {target}` does not print `NOPARSE` - the file parses",
        f"- 5. `grep -cE '^[[:space:]]*test[[:space:]]+(\"|[A-Za-z_])' {target}` prints at least `{tests_needed}`",
        f"- 6. `t27c test-report {target} 2>&1 | grep -c BLOCKED` prints `0` - the generated "
        "code must COMPILE and its tests must run",
        "",
    ]

    if in_tree:
        read = [
            "## Read the original first",
            "",
            "It is in this checkout, and it is also quoted in full at the end of this issue:",
            "",
            "```",
            f"$ wc -l {path}",
            f"$ sed -n '1,120p' {path}",
            "```",
            "",
        ]
    else:
        read = [
            "## Read the original first",
            "",
            f"`{path}` is in **{repo}**, not in this checkout - do not look for it on disk, "
            f"it is not there. Its full text, as of `{sha[:12]}`, is quoted at the end of this "
            "issue, one line per numbered row: the number and the `|` are not part of the source.",
            "",
        ]

    what_verilog = [
        f"Create `{target}` as `module {units[0].name} {{ ... }}`: its constants, structs, enums "
        "and functions describe what the original computes, and `t27c gen-verilog` turns it into "
        "the module. Quoted verbatim from the original:",
        "",
        *listed,
        "",
        "Look at `specs/fpga/apb_bridge.t27` and `specs/fpga/cts.t27` for the shape the Verilog "
        "backend lowers. Add `test` blocks for the functions that compute the module's outputs, "
        "asserting on the values the original produces for inputs you choose.",
    ]
    body = [
        f"# {title}",
        "",
        f"**Roadmap stage {stage.number} of {len(STAGES)}: {stage.title}** (goal #{stage.goal}). "
        f"Target of the stage: {stage.target}.",
        "",
        f"`{path}` ({repo}) is {lines} lines of hand-written {lang}. Re-author it as `{target}`, "
        "so that the code is generated from `.t27` instead of written by hand. The original stays "
        "where it is: this issue adds the `.t27` source it should have been written in.",
        "",
        "## Boundary",
        "",
        target,
        "",
        # Its own heading, not a paragraph under Boundary: the Queen takes the
        # first path-like token of EVERY line in that section, and `/tmp` is one.
        "## Only this file",
        "",
        "Write this one file and nothing else. Anything you need to try out goes under `/tmp`, "
        "never into the repository: a scratch file, a notes file or a `debug_output.txt` in the "
        "worktree is outside the boundary, it is reported, and it stops the work being published.",
        "",
        *read,
        "## What to write",
        "",
        *(what_verilog if lang == "Verilog" else [
            f"Create `{target}` with one `.t27` {kind} per {kind} below, under the name given. The "
            "name is how the port is checked. Quoted verbatim from the original:",
            "",
            *listed,
            "",
            f"Add at least {n} `test` blocks, asserting on the behaviour you read in the original: "
            "the values it returns for inputs you choose, and the edge cases it handles. Where a "
            "function only moves data in or out - a database call, a request, a file, a process - "
            "port the DECISION inside it as a pure helper and test the helper; the function itself "
            "keeps its name with an `undefined;` body that no test calls (the brief below, \"Port "
            "the decision, not the plumbing\"). A ported body with nothing asserting on it is a "
            "claim, not a result - the review compiles the generated code and runs exactly those "
            "tests.",
        ]),
        "",
        *criteria,
        "## User Scenarios & Testing",
        "",
        f"- **Given** `{target}` does not exist and `{path}` ({repo}) is quoted below,",
        f"  **when** its {n} {kind}{'s' if n != 1 else ''} are ported and the commands under "
        "Acceptance criteria are run from the repository root,",
        "  **then** each of those commands prints the value stated beside it.",
        "",
        "## Requirements",
        "",
        f"- FR-001: `{target}` MUST declare every {kind} listed above under the name given, "
        "carrying the original's decisions as code; only data moving in or out may be an "
        "`undefined;` body, and no test may call one.",
        f"- FR-002: the file MUST parse and compile: `t27c parse {target}` and "
        f"`t27c test-report {target}` are the checks.",
        "- FR-003: the change MUST be this one file - no other spec, no scratch file, no edit "
        "to the original.",
        "- FR-004: each test MUST assert something the original does; a test that asserts "
        "`true` ports nothing.",
        "",
    ]
    if goal:
        body += ["## The goal of the game", "", goal, ""]
    if howto:
        body += ["## How to rewrite code into .t27", "", howto, ""]
    if belt:
        body += ["## The language and the instruments", "", belt, ""]
    quoted = gutter(text)
    marks = fence(quoted)
    body += [
        f"## The original: {path}",
        "",
        f"{repo} at `{sha[:12]}`, {lines} lines, quoted verbatim behind a line-number gutter.",
        "",
        marks + FENCE_LANG.get(lang, ""),
        quoted,
        marks,
        "",
    ]
    return title, "\n".join(body)


# --------------------------------------------------------------------------
# Twins of the Queen's readers, so a body is checked the way she will read it.
# --------------------------------------------------------------------------

CRITERIA_HEADINGS = ("success criteria", "acceptance criteria", "acceptance", "done when",
                     "готово, когда", "готово когда", "критерии успеха", "критерии приёмки",
                     "критерии приемки", "критерии")


def criteria_twin(body: str) -> list[str]:
    """Pinned twin of `QueenSpecQuality.bullets(in:under:)` (queen-core)."""
    collecting, found = False, []
    for raw in body.split("\n"):
        line = raw.strip()
        if line.startswith("#"):
            title = line.lstrip("#").strip().lower()
            collecting = any(h in title for h in CRITERIA_HEADINGS)
            continue
        if not collecting or not line.startswith(("- ", "* ")):
            continue
        item = line[2:]
        for marker in ("[x] ", "[X] ", "[ ] "):
            if item.startswith(marker):
                item = item[len(marker):]
        if item.strip():
            found.append(item.strip())
    return found


def shape_problems(body: str) -> list[str]:
    """What the Queen would refuse, or read wrongly, in this body."""
    from task_shape import boundary_paths, problems
    found = list(problems(body, set()))
    paths = boundary_paths(body)
    if len(paths) != 1 or not paths[0].startswith("specs/port/"):
        found.append(f"the boundary is not exactly one specs/port path: {paths}")
    criteria = criteria_twin(body)
    if len(criteria) != 6 or not all(c[:2] in {f"{k}." for k in range(1, 7)} for c in criteria):
        found.append(f"the Queen would read {len(criteria)} criteria, not the 6 written")
    for need in ("## User Scenarios", "## Requirements", " MUST "):
        if need not in body:
            found.append(f"missing {need.strip()}")
    if len(body) > 60000:
        found.append(f"body is {len(body)} characters; GitHub takes 65536")
    return found


# --------------------------------------------------------------------------
# Sources
# --------------------------------------------------------------------------

def git(args: list[str], cwd: str, timeout: int = 600) -> tuple[int, str]:
    done = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True,
                          timeout=timeout, stdin=subprocess.DEVNULL)
    return done.returncode, (done.stdout or "")


def checkout_of(repo: str, ref: str = "") -> str | None:
    """A shallow clone of `repo` holding only its small files, or this checkout
    for t27. None when unreadable. The token, when there is one, is never logged.

    `--filter=blob:limit` and not `blob:none`: with no blobs at all, asking for a
    file's SIZE fetches that blob, one round trip per file, and listing a tree
    the size of trinity's took longer than ten minutes. With the limit, every
    file small enough to quote arrives in the one clone, and a file that did not
    arrive is by that fact too big to quote.
    """
    if repo == REPO and not ref:
        return WORK
    dest = os.path.join(CACHE, re.sub(r"[^A-Za-z0-9_.-]", "__", f"{repo}@{ref}"))
    if os.path.isdir(os.path.join(dest, ".git")):
        return dest
    os.makedirs(CACHE, exist_ok=True)
    token = os.environ.get("ROADMAP_READ_TOKEN", "")
    url = (f"https://x-access-token:{token}@github.com/{repo}" if token
           else f"https://github.com/{repo}")
    env = dict(os.environ, GIT_TERMINAL_PROMPT="0")
    branch = ["--branch", ref] if ref else []
    done = subprocess.run(["git", "clone", "-q", "--depth", "1", f"--filter=blob:limit={MAX_CHARS * 2}",
                           "--no-checkout", *branch, url, dest], capture_output=True, text=True,
                          timeout=1800, env=env, stdin=subprocess.DEVNULL)
    return dest if done.returncode == 0 else None


def files_of(checkout: str) -> list[tuple[int, str]]:
    """(size, path) for every blob at HEAD that is present locally.

    Sizes come from the objects already on disk (`--batch-all-objects` never
    fetches), so a partial clone is never asked for a blob it left out.
    """
    _, present = git(["cat-file", "--batch-all-objects",
                      "--batch-check=%(objectname) %(objecttype) %(objectsize)"], checkout)
    sizes = {}
    for line in present.split("\n"):
        parts = line.split()
        if len(parts) == 3 and parts[1] == "blob":
            sizes[parts[0]] = int(parts[2])
    _, out = git(["ls-tree", "-r", "HEAD"], checkout)
    found = []
    for line in out.split("\n"):
        meta, _, path = line.partition("\t")
        parts = meta.split()
        if len(parts) == 3 and parts[1] == "blob" and parts[2] in sizes:
            found.append((sizes[parts[2]], path))
    return found


def wanted(stage: Stage, path: str) -> bool:
    ext = os.path.splitext(path)[1]
    if ext not in stage.exts or not SAFE_PATH.match(path):
        return False
    parts = path.split("/")
    if any(p in SKIP_DIRS or p in stage.skip_dirs for p in parts[:-1]):
        return False
    return not SKIP_FILE.search(parts[-1])


def candidates(stage: Stage, repo: str, prefixes: list[str], checkout: str):
    """Paths in priority order: by prefix, then smallest first."""
    pool = [(size, path) for size, path in files_of(checkout)
            if size <= MAX_CHARS * 2 and wanted(stage, path)]
    taken = set()
    for prefix in prefixes:
        for size, path in sorted(p for p in pool if p[1].startswith(prefix) and p[1] not in taken):
            taken.add(path)
            yield path


def log(message: str) -> None:
    import datetime
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"{stamp} {message}", flush=True)


def claimed_targets(issues_json: str = "") -> set[str] | None:
    """Every specs/port target that an issue in ANY state already names.

    Any state, on purpose: a port closed as done is done, and one closed as not
    planned was closed by somebody - re-opening that decision is not a feeder's.
    `issues_json` reads a saved `[{title, body}]` list instead of asking gh, for
    a dry run on a machine without it.
    """
    import json
    if issues_json:
        with open(issues_json, encoding="utf-8") as handle:
            listed = json.load(handle)
    else:
        done = subprocess.run(["gh", "issue", "list", "--repo", REPO, "--state", "all",
                               "--limit", "1000", "--search", "Port in:title",
                               "--json", "number,title,body"],
                              capture_output=True, text=True, timeout=300)
        if done.returncode != 0:
            log("gh issue list failed: " + (done.stderr or "")[:200])
            return None
        listed = json.loads(done.stdout or "[]")
    names = set()
    for issue in listed:
        text = issue.get("title", "") + "\n" + (issue.get("body") or "")
        names.update(re.findall(r"specs/port/[\w./-]+?\.t27", text))
    return names


def pick(stage: Stage, claimed: set[str], counts: dict, root: str, belt: str):
    """The next file of this stage that is a finishable task, as (title, body)."""
    for source, prefixes in stage.sources:
        repo, _, ref = source.partition("@")
        checkout = checkout_of(repo, ref)
        if checkout is None:
            counts["unreadable"].add(source)
            continue
        _, sha = git(["rev-parse", "HEAD"], checkout)
        for path in candidates(stage, repo, prefixes, checkout):
            target = target_for(repo, path)
            if target in claimed or os.path.exists(os.path.join(WORK, target)):
                counts["claimed"] += 1
                continue
            code, text = git(["show", f"HEAD:{path}"], checkout)
            if code != 0:
                continue
            claimed.add(target)  # one look per run, whatever the verdict
            lang = LANG[os.path.splitext(path)[1]]
            if GENERATED.search(text[:800]):
                counts["generated"] += 1
                continue
            if text.count("\n") + 1 > MAX_LINES or len(text) > MAX_CHARS:
                counts["too big"] += 1
                continue
            units = EXTRACT[lang](text)
            if not units:
                counts["no units"] += 1
                continue
            if not has_decision(text, lang):
                counts["no decision"] += 1
                continue
            if len(units) > MAX_FNS:
                counts["over the ceiling"] += 1
                continue
            if lang == "Verilog" and len(units) != 1:
                # gen-verilog emits one module per `.t27`, so a file of several
                # modules is several tasks, and one Boundary cannot hold them.
                counts["several modules"] += 1
                continue
            title, body = build(stage, repo, sha.strip(), path, text, lang, units, root, belt)
            problems = shape_problems(body)
            if problems:
                counts["refused"] += 1
                log(f"REFUSED {title[:80]}: {problems[0]}")
                continue
            return title, body
    return None


# --------------------------------------------------------------------------

def self_test() -> int:
    failures = []

    def check(name, got, want):
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    py = "import os\n\nclass Box:\n    def __init__(self, x):\n        self.x = x\n    def get(self):\n" \
         "        return self.x\n\ndef clamp(x, lo, hi):\n    return max(lo, min(x, hi))\n\n" \
         "async def fetch(url):\n    def inner():\n        pass\n    return inner\n\ndef test(x):\n    return x\n"
    check("python units", [u.name for u in units_python(py)],
          ["Box_init", "Box_get", "clamp", "fetch", "test_"])
    ts = "export function add(a: number, b: number): number {\n  return a + b\n}\n" \
         "export const mul = (a: number, b: number) => a * b\n" \
         "export class Lease {\n  constructor(x: number) {\n  }\n  async renew(ms: number): Promise<void> {\n" \
         "    if (ms > 0) {\n    }\n  }\n}\nfunction helper<T>(x: T) {\n  return x\n}\n"
    check("ts units", [u.name for u in units_ts(ts)], ["add", "mul", "Lease_renew", "helper"])
    rs = "pub fn open(p: &str) -> u32 { 0 }\nimpl Ring {\n    pub fn spin(&self) {}\n}\n" \
         "#[cfg(test)]\nmod tests {\n    fn t() {}\n}\n"
    check("rust units", [u.name for u in units_rust(rs)], ["open", "Ring_spin"])
    zig = "pub fn bind(a: u8) u8 { return a; }\nconst Vm = struct {\n    pub fn step(self: *Vm) void {}\n};\n" \
          "test \"bind\" {}\n"
    check("zig units", [u.name for u in units_zig(zig)], ["bind", "Vm_step"])
    go = "func Plan(x int) int { return x }\nfunc (q *Queen) Tick() error { return nil }\n"
    check("go units", [u.name for u in units_go(go)], ["Plan", "Queen_Tick"])
    c = "#include <x.h>\nstatic int add(int a, int b)\n{\n  return a + b;\n}\nint decl(void);\n"
    check("c units", [u.name for u in units_c(c)], ["add"])
    v = "module tern_mac #(parameter W=8) (\n);\nendmodule\nmodule top;\nendmodule\n"
    check("verilog units", [u.name for u in units_verilog(v)], ["tern_mac", "top"])
    sh = "set -e\nusage() {\n  echo hi\n}\nfunction run-it() {\n  :\n}\n"
    check("shell units", [u.name for u in units_shell(sh)], ["usage", "run_it", "main"])

    check("t27 target", target_for(REPO, "tools/check_x.py"), "specs/port/tools/check_x.t27")
    check("other target", target_for("gHashTag/BrowserOS", "trios/a/queen-lease.ts"),
          "specs/port/browseros/trios/a/queen-lease.t27")
    check("wanted: test file", wanted(STAGES[1], "trios/agent-server/x/queen-a.test.ts"), False)
    check("wanted: vendored", wanted(STAGES[4], "src/external/x.zig"), False)
    check("wanted: plain", wanted(STAGES[4], "src/vsa/bind.zig"), True)
    check("wanted: unsafe path", wanted(STAGES[4], "src/[id].zig"), False)
    check("fence", fence("a ```` b"), "`````")
    # The six that a dry run chose on 2026-09-27, and what a port can test.
    check("no decision: a todo! stub", has_decision(
        'pub fn run() {\n    todo!("check module: individual check runners for CI")\n}\n', "Rust"), False)
    check("no decision: a main that prints", has_decision(
        '//! Build script\n\nfn main() {\n    println!("cargo:rerun-if-changed=../.");\n}\n', "Rust"), False)
    check("no decision: a setter that prints", has_decision(
        "use anyhow::Result;\n\npub struct LangCmd {\n    pub lang: String,\n}\n\n"
        "pub fn run(cmd: LangCmd) -> Result<()> {\n    println!(\"Language set to: {}\", cmd.lang);\n"
        "    Ok(())\n}\n", "Rust"), False)
    check("no decision: a docstring that says if", has_decision(
        'def f(x):\n    """Return x if it is set, or else nothing."""\n    return x  # if not, None\n',
        "Python"), False)
    check("no decision: only the tests compare (rust)", has_decision(
        "pub fn anchor() -> f64 {\n    3.0\n}\n#[cfg(test)]\nmod tests {\n    fn t() {\n"
        "        assert!((anchor() - 3.0).abs() < 1e-12);\n    }\n}\n", "Rust"), False)
    check("no decision: only the tests compare (zig)", has_decision(
        "pub fn sha256(data: []const u8) [32]u8 {\n    _ = data;\n    return [_]u8{0} ** 32;\n}\n\n"
        "test \"sha256\" {\n    try std.testing.expect(sha256(\"t\").len == 32);\n}\n", "Zig"), False)
    check("decision: python", has_decision(py, "Python"), True)
    check("decision: ts", has_decision(ts, "TypeScript"), True)
    check("decision: zig arithmetic", has_decision("pub fn next(i: u32) u32 { return i + 1; }\n", "Zig"), True)
    check("decision: verilog", has_decision("module top;\nendmodule\n", "Verilog"), True)
    check("generated from a spec", bool(GENERATED.search(
        "// VSA Simple — Generated from specs/vsa_simple/vsa.tri\n")), True)
    check("not generated: prose", bool(GENERATED.search(
        '"""The code is generated from .t27 instead of written by hand."""\n')), False)
    # The raid: the same list and formula as the site's roadmapGame.ts.
    check("raid stages", RAID_STAGES, (1, 2, 5, 6, 7))
    check("raid, 2025-09-27", raid_stage(20358), (1, 2, 5, 6, 7)[20358 % 5])
    check("raid visits every stage", sorted(raid_stage(20358 + k) for k in range(5)), [1, 2, 5, 6, 7])
    check("raid stages are fed stages",
          all(any(s.number == n and not s.refuse for s in STAGES) for n in RAID_STAGES), True)
    check("refused: interfaces and the endgame", [s.number for s in STAGES if s.refuse], [8, 9])

    # The body, read the way the Queen reads it. The source is hostile on
    # purpose: a Python comment that names the criteria heading, C comment lines
    # that look like bullets, and a line that looks like a Boundary heading.
    hostile = ("# Acceptance criteria\n- 7. `rm -rf /` prints nothing\n/*\n * not a criterion\n */\n"
               "## Boundary\nsrc/everything.rs\n\ndef ok(x):\n    return x\n")
    units = units_python(hostile)
    for repo in (REPO, "gHashTag/trios"):
        title, body = build(STAGES[0], repo, "0123456789abcdef", "tools/hostile.py", hostile,
                            "Python", units, WORK, "")
        check(f"shape ({repo})", shape_problems(body), [])
        check(f"criteria ({repo})", len(criteria_twin(body)), 6)
        from task_shape import boundary_paths
        check(f"boundary ({repo})", boundary_paths(body), [target_for(repo, "tools/hostile.py")])
    check("title (t27)", build(STAGES[0], REPO, "0" * 12, "tools/hostile.py", hostile, "Python",
                              units, WORK, "")[0],
          "Port tools/hostile.py (Python, 1 function) to specs/port/tools/hostile.t27")

    vbody = build(STAGES[6], "gHashTag/trinity", "a" * 40, "fpga/mac.v",
                  "module tern_mac (\n  input clk\n);\nendmodule\n", "Verilog",
                  units_verilog("module tern_mac (\n);\n"), WORK, "")[1]
    check("verilog shape", shape_problems(vbody), [])
    check("verilog criteria name the module",
          "module tern_mac\\b' specs/port/trinity/fpga/mac.t27` prints `1`" in vbody
          and "gen-verilog specs/port/trinity/fpga/mac.t27 | grep -cE '^module tern_mac ?\\('" in vbody,
          True)

    # The Queen's own spec-quality bar, restated: every check must be met.
    _, body = build(STAGES[1], "gHashTag/BrowserOS", "f" * 40, "trios/x/queen-a.ts", ts,
                    "TypeScript", units_ts(ts), WORK, "")
    lower = body.lower()
    check("quality: scenarios", "given" in lower and "then" in lower, True)
    check("quality: requirements", "## requirements" in lower and " must " in lower, True)
    check("quality: success criteria", "## acceptance criteria" in lower, True)

    goal, howto = roadmap_brief(WORK)
    check("brief present", bool(goal) and bool(howto), True)
    for heading in re.findall(r"(?m)^#+\s*(.+)$", goal + "\n" + howto):
        if any(h in heading.lower() for h in CRITERIA_HEADINGS) or heading.startswith("Boundary"):
            failures.append(f"brief heading would be read as a criteria or boundary heading: {heading}")

    for f in failures:
        print("FAIL", f)
    print(f"self-test: {'FAIL' if failures else 'PASS'} ({len(failures)} failure(s))")
    return 1 if failures else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=7)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--stage", type=int, action="append",
                    help="only these stage numbers (repeatable); default: every stage")
    ap.add_argument("--when-idle", action="store_true")
    ap.add_argument("--runway", default="0")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--issues-json", default="",
                    help="read the issue list from this file instead of gh (dry runs only)")
    args = ap.parse_args()
    if args.issues_json and not args.dry_run:
        ap.error("--issues-json is for dry runs: a saved list cannot tell a new duplicate")

    if args.self_test:
        return self_test()

    from feed_empty_bodies import queue_idle
    from toolbelt import brief as toolbelt_brief

    if args.when_idle:
        want = queue_idle(args.runway)
        if want is None:
            log("could not read the swarm, so nothing was fed - and this run is RED")
            return 2
        if want <= 0:
            log("swarm busy - nothing added")
            return 0
        args.limit = min(args.limit, want)

    stages = [s for s in STAGES if not args.stage or s.number in args.stage]
    for s in stages:
        if s.refuse:
            log(f"stage {s.number} (#{s.goal}) is not fed: {s.refuse}")
    stages = [s for s in stages if not s.refuse]

    claimed = claimed_targets(args.issues_json)
    if claimed is None:
        log("could not list the issues, so a duplicate cannot be told from a new task - nothing filed")
        return 2
    belt = toolbelt_brief(WORK)
    os.makedirs(OUTD, exist_ok=True)
    from collections import Counter
    counts: dict = Counter()
    counts["unreadable"] = set()
    made, exhausted = 0, set()
    # Round robin: every stage gets its share, so every stage moves - and the
    # raid of the day goes first, twice a turn. The site draws the same raid.
    raid = raid_stage(int(time.time() // 86400))
    log(f"raid of the day: stage {raid}")
    turn = [s for s in stages if s.number == raid] * 2 + [s for s in stages if s.number != raid]
    while made < args.limit and len(exhausted) < len(stages):
        for stage in turn:
            if made >= args.limit:
                break
            if stage.number in exhausted:
                continue
            found = pick(stage, claimed, counts, WORK, belt)
            if not found:
                exhausted.add(stage.number)
                continue
            title, body = found
            slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")[:120]
            path = os.path.join(OUTD, slug + ".md")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(body)
            if args.dry_run:
                log(f"dry-run: stage {stage.number}: {title} ({len(body)} chars) -> {path}")
                made += 1
                continue
            done = subprocess.run(["gh", "issue", "create", "--repo", REPO, "--title", title,
                                   "--body-file", path], capture_output=True, text=True, timeout=120)
            if done.returncode != 0:
                log(f"create FAILED for {title[:80]}: {(done.stderr or '').strip()[:200]}")
                continue
            log(f"created {done.stdout.strip()} stage {stage.number}: {title}")
            made += 1
    unreadable = sorted(counts.pop("unreadable"))
    if unreadable:
        log(f"could not read {', '.join(unreadable)}: private, or gone. Their stages are fed "
            "only with ROADMAP_READ_TOKEN set to a token that can read them.")
    skipped = ", ".join(f"{k} {v}" for k, v in sorted(counts.items()) if v)
    log(f"done: {made} issue(s) {'would be ' if args.dry_run else ''}created; passed over: {skipped or 'none'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
