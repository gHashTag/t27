#!/usr/bin/env python3
"""Draft the acceptance criteria an escalated review card is missing -- DRY RUN.

WHY

A finished bee whose issue states no criterion is escalated by the review
policy (`QueenReviewDecision`: `guard totalCriteria > 0 else .escalate`), and an
escalation has no automatic exit. On the public board those cards sit in
"in review" for good. `releaseStaleContracts` (queen-tick.ts) is the one road
out that needs no person: when the ISSUE's criteria become non-empty and differ
from the ones frozen on the dispatch row, the dispatch is released and the
issue is handed to a fresh bee with a contract it can be judged against.

So the cheapest repair is text: append the sections the Queen's parser reads.
This tool works out, per card, what that text would be -- and says so when it
cannot be derived from the issue's own words, instead of inventing a bar.

WHAT THE QUEEN READS (trios agent-server, read 2026-10-03)

  queen-core/Sources/QueenCore/QueenSpecQuality.swift
    L75-78    hasSection: lowercased body contains "## <name>" (substring)
    L81-86    hasAcceptanceScenario: a GIVEN marker and a THEN marker anywhere
    L94-98    hasObligation: `\\bFR-\\d{3}\\b`, or " must " / RU "dolzhen " /
              RU "obyazan " in the lowercased body
    L105-115  hasMeasurableOutcome: four regexes over the WHOLE body
    L117-159  judge: boundary / scenarios / requirements / success criteria
    L176-187  criteriaHeadings (ten, EN + RU)
    L221-227  criteriaWithSource: stated bullets, else FR lines, else none
    L230-253  bullets: "- " or "* " lines (not "1.") under a heading whose
              title contains a criteria heading, up to the next "#" line
    L256-270  requirements: every line holding FR-nnn
  queen-core/Sources/QueenCore/QueenIssueBoundary.swift
    L34-36    "## Boundary" / RU "## Granitsy" prefix, case-sensitive
    L66-88    pathToken: first whitespace token holding "/" or ".ext"
    L98-118   paths: split on the CHARACTER "\\n" -- "\\r\\n" is one Swift
              Character, so a CRLF body is one line and declares no boundary
  queen-core/Sources/queend/main.swift L296-310: the skip sentences;
    apps/server/src/api/routes/queen-public-status.ts L195-198 buckets
    "delegatable but not yet a spec" as incompleteSpec and "not yet a spec" /
    "declares no boundary" as missingBoundary.
  apps/server/src/api/services/queen-criteria-run.ts
    L52-53    CONNECTOR: prints | prints at least | prints more than |
              does not print, between two code spans
    L84-122   parseCriterionChecks; L136-150 programs; ~L160-210 t27c
              subcommands; L469-530 commandSafety
  apps/server/src/api/services/queen-tick.ts
    L2671-2710 releaseStaleContracts; L3728-3736 acceptedOnBaseTruthAlone

The twins below are PINNED to those lines by `--self-test`. The heading list
and the board address are imported, not copied: `feed_roadmap.CRITERIA_HEADINGS`
and `refile.STATUS`.

CATEGORIES

  A  a merged PR into master closes it (closing keyword, the GitHub closing
     reference, or the bee's own `queen-N` branch): propose CLOSE, not text.
     Adding criteria to landed work releases it to a FRESH bee whose
     criteria pass at the merge base -- `acceptedOnBaseTruthAlone` escalates
     that again.
  B  some of scenarios / requirements / success criteria already present:
     append only the missing sections.
  C  none of the three: append the full block.
  D  no criterion can be derived from the issue's own words that is either
     stated by its author or measurable and not already true on master. Only
     a person can write it; nothing is proposed.

The boundary is not one of the B/C sections: a card in review was dispatched,
so it had one. Its absence now is reported, never drafted.

Usage:
    python3 tools/queen/criteria_backfill.py --self-test
    python3 tools/queen/criteria_backfill.py --dry-run
    python3 tools/queen/criteria_backfill.py --dry-run --board-file /tmp/qboard.json --limit 10

Write mode is NOT implemented: this reads GitHub and writes two local files.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import unicodedata
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from feed_roadmap import CRITERIA_HEADINGS  # noqa: E402  the Queen's list, held once
from refile import STATUS as BOARD_URL  # noqa: E402  the public board, held once

REPO = "gHashTag/t27"
DEFAULT_BRANCH = "master"
ROOT = HERE.parent.parent
REPORT_MD = ROOT / ".trinity" / "criteria-backfill-dryrun.md"
REPORT_JSON = Path("/tmp/criteria-backfill-dryrun.json")
CACHE_DIR = Path("/tmp/criteria-backfill-cache")
CACHE_TTL_SECONDS = 3600

# Non-ASCII parser tokens, escaped so this source stays ASCII (law L3).
RU_BOUNDARY = "## \u0413\u0440\u0430\u043d\u0438\u0446\u044b"
RU_SCENARIOS = "\u0421\u0446\u0435\u043d\u0430\u0440\u0438\u0438"
RU_REQUIREMENTS = "\u0422\u0440\u0435\u0431\u043e\u0432\u0430\u043d\u0438\u044f"
RU_GIVEN = ("\u0434\u0430\u043d\u043e", "\u0435\u0441\u043b\u0438 ")
RU_THEN = ("\u0442\u043e\u0433\u0434\u0430", "\u0442\u043e ")
RU_MUST = ("\u0434\u043e\u043b\u0436\u0435\u043d ", "\u043e\u0431\u044f\u0437\u0430\u043d ")

SCENARIO_HEADINGS = ("User Scenarios", "Scenarios", RU_SCENARIOS)
REQUIREMENT_HEADINGS = ("Requirements", RU_REQUIREMENTS)
SPEC_SECTIONS = ("scenarios", "requirements", "success criteria")


# --------------------------------------------------------------------------
# Twin of QueenSpecQuality.swift and QueenIssueBoundary.swift
# --------------------------------------------------------------------------

# Foundation's CharacterSet.newlines: U+000A-U+000D, U+0085, U+2028, U+2029.
SWIFT_NEWLINES = re.compile("[\n\x0b\x0c\r\x85\u2028\u2029]")
# Character.isWhitespace, for the boundary token split.
SWIFT_IS_WHITESPACE = re.compile(
    "[\t\n\x0b\x0c\r \x85\xa0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000]+"
)
FR_RE = re.compile(r"\bFR-\d{3}\b")
EXT_RE = re.compile(r"\.\w{1,10}$")
MEASURABLE_RES = tuple(
    re.compile(p, re.IGNORECASE)
    for p in (
        r"\b\d+\s*(ms|s|MB|GB|%|files?|lines?|tests?|attempts?)\b",
        r"`[^`]+\.(swift|ts|rs|md|json|zig|v)`",
        r"\bexit(s)? (0|non-zero)\b",
        r"`(make|swift|bun|cargo|git) [^`]+`",
    )
)


def swift_trim(text: str) -> str:
    """`trimmingCharacters(in: .whitespaces)`: tab and category Zs only."""
    def blank(c: str) -> bool:
        return c == "\t" or unicodedata.category(c) == "Zs"
    i, j = 0, len(text)
    while i < j and blank(text[i]):
        i += 1
    while j > i and blank(text[j - 1]):
        j -= 1
    return text[i:j]


def swift_newline_components(body: str) -> list[str]:
    """`components(separatedBy: .newlines)`: every newline scalar splits."""
    return SWIFT_NEWLINES.split(body)


def swift_split_lf(body: str) -> list[str]:
    """`split(separator: "\\n")` on a Swift String: "\\r\\n" is ONE Character."""
    return re.split(r"(?<!\r)\n", body)


def has_section(body: str, names) -> bool:
    lower = body.lower()
    return any(("## " + name.lower()) in lower for name in names)


def has_acceptance_scenario(body: str) -> bool:
    lower = body.lower()
    given = ("**given**", "given ") + RU_GIVEN
    then = ("**then**", "then ") + RU_THEN
    return any(g in lower for g in given) and any(t in lower for t in then)


def has_obligation(body: str) -> bool:
    if FR_RE.search(body):
        return True
    lower = body.lower()
    return " must " in lower or any(m in lower for m in RU_MUST)


def has_measurable_outcome(body: str) -> bool:
    return any(p.search(body) for p in MEASURABLE_RES)


def is_boundary_heading(trimmed: str) -> bool:
    return trimmed.startswith("## Boundary") or trimmed.startswith(RU_BOUNDARY)


def path_token(line: str) -> str | None:
    for raw in SWIFT_IS_WHITESPACE.split(line):
        cleaned = raw
        changed = True
        while changed and cleaned:
            changed = False
            if cleaned and cleaned[0] in "`\"'(":
                cleaned = cleaned[1:]
                changed = True
            if cleaned and cleaned[-1] in "`\"'.,;:!?)":
                cleaned = cleaned[:-1]
                changed = True
        if not cleaned:
            continue
        if "/" in cleaned or EXT_RE.search(cleaned):
            return cleaned
    return None


def boundary_paths(body: str) -> list[str] | None:
    """`QueenIssueBoundary.paths(from:)`: None when no boundary heading."""
    in_bounds, found, paths = False, False, []
    for line in swift_split_lf(body):
        trimmed = swift_trim(line)
        if trimmed.startswith("## "):
            if in_bounds:
                break
            in_bounds = is_boundary_heading(trimmed)
            if in_bounds:
                found = True
            continue
        if not in_bounds or not trimmed:
            continue
        token = path_token(trimmed)
        if token:
            paths.append(token)
    return paths if found else None


def judge(body: str) -> dict:
    """`QueenSpecQuality.judge(body:)`."""
    paths = boundary_paths(body)
    checks = {
        "boundary": bool(paths),
        "scenarios": has_section(body, SCENARIO_HEADINGS) or has_acceptance_scenario(body),
        "requirements": has_section(body, REQUIREMENT_HEADINGS) and has_obligation(body),
        "success criteria": has_section(body, CRITERIA_HEADINGS) and has_measurable_outcome(body),
    }
    return {
        "checks": checks,
        "missing": [name for name, met in checks.items() if not met],
        "delegatable": bool(paths),
        "isSpec": all(checks.values()),
    }


def bullets(body: str, headings) -> list[str]:
    collecting, found = False, []
    for raw in swift_newline_components(body):
        line = swift_trim(raw)
        if line.startswith("#"):
            title = swift_trim(line.lstrip("#")).lower()
            collecting = any(h in title for h in headings)
            continue
        if not collecting or not (line.startswith("- ") or line.startswith("* ")):
            continue
        item = line[2:]
        for marker in ("[x] ", "[X] ", "[ ] "):
            if item.startswith(marker):
                item = item[len(marker):]
        cleaned = swift_trim(item)
        if cleaned:
            found.append(cleaned)
    return found


def requirement_lines(body: str) -> list[str]:
    found = []
    for raw in swift_newline_components(body):
        line = swift_trim(raw)
        if not FR_RE.search(line):
            continue
        item = line
        for prefix in ("- ", "* "):
            if item.startswith(prefix):
                item = item[len(prefix):]
        cleaned = swift_trim(item)
        if cleaned:
            found.append(cleaned)
    return found


def criteria_with_source(body: str) -> tuple[list[str], str]:
    """`QueenSpecQuality.criteriaWithSource(from:)`."""
    stated = bullets(body, CRITERIA_HEADINGS)
    if stated:
        return stated, "stated"
    fallback = requirement_lines(body)
    if fallback:
        return fallback, "requirements"
    return [], "none"


# --------------------------------------------------------------------------
# Twin of queen-criteria-run.ts: which criterion lines the runner can measure
# --------------------------------------------------------------------------

CONNECTOR = re.compile(r"^\s+(prints at least|prints more than|prints|does not print)\s+$")
CRITERION_PROGRAMS = frozenset(
    {"t27c", "grep", "wc", "head", "tail", "sort", "uniq", "cut", "tr", "cat", "test", "echo"}
)
# CRITERION_T27C_SUBCOMMANDS (queen-criteria-run.ts ~L160-210): what the
# runner admits in a criterion an AUTHOR wrote. Used only to decide whether a
# line lifted from an issue is measurable.
RUNNER_T27C_SUBCOMMANDS = frozenset({
    "spec-status", "impl-status", "classify", "parse", "parse-complete", "parse-conform",
    "typecheck", "test-report", "symbols", "inspect", "outline", "exports", "depends",
    "test", "tree", "strings", "count", "loc", "size", "metrics", "depth", "stack", "hash",
    "coverage", "lint", "deadcode", "orphans", "spellcheck", "validate", "validate-vacuity",
    "todo", "check-deps", "deps-tree", "analyze", "xref", "api-diff", "diff", "eval", "gen",
    "gen-c", "gen-rust", "gen-verilog", "frozen-digest", "version", "--version",
})
# The read-only subset this tool ever DRAFTS (and runs locally to measure).
DRAFT_T27C_SUBCOMMANDS = frozenset({"spec-status", "parse"})
SCRATCH_PREFIX = "/tmp/t27-"
SCRATCH_TARGET = re.compile(r"^/tmp/t27-[A-Za-z0-9._-]+$")
PATH_SAFE = re.compile(r"^[A-Za-z0-9._/-]+$")


def normalise_criterion_command(cmd: str) -> str:
    out = cmd.strip()
    out = re.sub(r"^cd\s+(?:/Users/[^/\s'\"]+/t27|~/t27)/?\s*&&\s*", "", out)
    out = re.sub(
        r"(^|[\s|&])(?:/Users/[^/\s'\"]+/t27/target/release/t27c|\./target/release/t27c)(?=\s|$)",
        r"\1t27c",
        out,
    )
    return out


def parse_criterion_checks(criterion: str) -> list[dict]:
    parts = criterion.split("`")
    checks: list[dict] = []
    i = 1
    while i + 2 < len(parts):
        connector = CONNECTOR.match(parts[i + 1])
        if connector:
            cmd = normalise_criterion_command(parts[i])
            expected = parts[i + 2]
            if cmd:
                word = connector.group(1)
                if word == "prints":
                    checks.append({"cmd": cmd, "op": "equals", "expected": expected})
                elif word == "does not print" and expected:
                    checks.append({"cmd": cmd, "op": "notContains", "expected": expected})
                elif word == "prints at least" and re.fullmatch(r"\d+", expected.strip()):
                    checks.append({"cmd": cmd, "op": "atLeast", "expected": expected.strip()})
                elif word == "prints more than" and re.fullmatch(r"\d+", expected.strip()):
                    checks.append(
                        {"cmd": cmd, "op": "atLeast", "expected": str(int(expected.strip()) + 1)}
                    )
            i += 2
        i += 2
    return checks


def _read_quoted(cmd: str, i: int):
    c = cmd[i]
    if c == "\\":
        nxt = cmd[i + 1] if i + 1 < len(cmd) else None
        if nxt is None or nxt in "\n\r\0":
            return {"reason": "a trailing backslash"}
        return {"value": nxt, "end": i + 2}
    if c == "'":
        end = cmd.find("'", i + 1)
        if end < 0:
            return {"reason": "an unterminated single quote"}
        literal = cmd[i + 1:end]
        if re.search(r"[\n\r\0]", literal):
            return {"reason": "a newline"}
        return {"value": literal, "end": end + 1}
    if c != '"':
        return None
    value = ""
    j = i + 1
    while j < len(cmd):
        d = cmd[j]
        if d == '"':
            return {"value": value, "end": j + 1}
        if d in "\n\r\0":
            return {"reason": "a newline"}
        if d in "$`":
            return {"reason": d + " inside double quotes (an expansion)"}
        if d == "\\" and j + 1 < len(cmd) and cmd[j + 1] in "$`\"\\":
            value += cmd[j + 1]
            j += 2
            continue
        value += d
        j += 1
    return {"reason": "an unterminated double quote"}


def _read_operator(cmd: str, i: int, in_word: bool):
    c = cmd[i]
    nxt = cmd[i + 1] if i + 1 < len(cmd) else ""
    if (c == "2" and not in_word and cmd.startswith("2>&1", i)
            and (i + 4 == len(cmd) or cmd[i + 4] in " \t|")):
        return {"op": "2>&1", "end": i + 4}
    if c == "|":
        return {"reason": "||"} if nxt == "|" else {"op": "|", "end": i + 1}
    if c == "&":
        return {"op": "&&", "end": i + 2} if nxt == "&" else {"reason": "& (a background job)"}
    if c == ">":
        if nxt in (">", "&", "|") and nxt:
            return {"reason": ">" + nxt + " (only > to /tmp/t27-... is allowed)"}
        return {"op": ">", "end": i + 1}
    return None


def tokenize(cmd: str):
    tokens: list[dict] = []
    state = {"buf": "", "raw": "", "in_word": False, "quoted": False}

    def flush():
        if state["in_word"]:
            tokens.append({"kind": "word", "value": state["buf"],
                           "quoted": state["quoted"], "raw": state["raw"]})
        state.update(buf="", raw="", in_word=False, quoted=False)

    i = 0
    while i < len(cmd):
        c = cmd[i]
        if c in " \t":
            flush()
            i += 1
            continue
        text = _read_quoted(cmd, i)
        if text:
            if "reason" in text:
                return False, text["reason"]
            state["buf"] += text["value"]
            state["raw"] += cmd[i:text["end"]]
            state["in_word"] = True
            state["quoted"] = True
            i = text["end"]
            continue
        if (c == ">" and state["in_word"] and not state["quoted"]
                and re.fullmatch(r"\d+", state["buf"])):
            return False, "a file-descriptor redirect other than 2>&1"
        op = _read_operator(cmd, i, state["in_word"])
        if op:
            if "reason" in op:
                return False, op["reason"]
            flush()
            tokens.append({"kind": "op", "op": op["op"]})
            i = op["end"]
            continue
        if c in "\n\r\0":
            return False, "a newline"
        if c in ";<()`$*?[]{}~#!":
            return False, "an unquoted " + c
        state["buf"] += c
        state["raw"] += c
        state["in_word"] = True
        i += 1
    flush()
    return True, tokens


def _argument_problem(token: dict, program: str) -> str:
    value, raw = token["value"], token["raw"]
    candidates = [value]
    if "=" in value:
        candidates.append(value[value.index("=") + 1:])
    if re.match(r"^-[A-Za-z]", value) and not value.startswith("--"):
        candidates.append(value[2:])
    for candidate in candidates:
        if candidate.startswith("/") and not SCRATCH_TARGET.match(candidate):
            return "an absolute path outside /tmp/t27- (" + value + ")"
        if SCRATCH_TARGET.match(candidate) and SCRATCH_PREFIX not in raw:
            return "/tmp/t27- spelled with quotes or escapes (" + value + ")"
    if re.search(r"(^|[/=])\.\.($|/)", value):
        return "a path out of the checkout (" + value + ")"
    if value.startswith("--files0-from"):
        return value + " reads names from a file"
    if program == "sort" and value.startswith("--c"):
        return value + " (sort --compress-program runs a program)"
    return ""


def command_safety(cmd: str, t27c_subcommands=None) -> dict:
    """`commandSafety`. `t27c_subcommands` defaults to the drafting subset."""
    allowed_subs = DRAFT_T27C_SUBCOMMANDS if t27c_subcommands is None else t27c_subcommands
    ok, lexed = tokenize(cmd)
    if not ok:
        return {"safe": False, "reason": lexed, "t27c_calls": []}
    calls: list[tuple[str, str]] = []
    expect_program, program, args = True, "", []

    def end_segment():
        if program == "t27c" and len(args) >= 2:
            file = next((a for a in args[1:] if a.endswith(".t27")), None)
            if file and not args[0].startswith("-"):
                calls.append((args[0], file))

    k = 0
    while k < len(lexed):
        token = lexed[k]
        if token["kind"] == "op":
            if expect_program:
                return {"safe": False, "reason": token["op"] + " with no command before it",
                        "t27c_calls": []}
            if token["op"] in ("|", "&&"):
                end_segment()
                program, args, expect_program = "", [], True
                k += 1
                continue
            if token["op"] == "2>&1":
                k += 1
                continue
            target = lexed[k + 1] if k + 1 < len(lexed) else None
            if not (target and target["kind"] == "word" and SCRATCH_TARGET.match(target["value"])
                    and SCRATCH_PREFIX in target["raw"]):
                return {"safe": False, "reason": "a redirect to anything but /tmp/t27-<name>",
                        "t27c_calls": []}
            k += 2
            continue
        if expect_program:
            if token["value"] not in CRITERION_PROGRAMS:
                return {"safe": False, "reason": (token["value"] or "an empty word")
                        + " is not an allowed program", "t27c_calls": []}
            if token["value"] == "t27c":
                sub = lexed[k + 1] if k + 1 < len(lexed) else None
                if not sub or sub["kind"] != "word":
                    return {"safe": False, "reason": "t27c with no subcommand", "t27c_calls": []}
                if sub["value"] not in allowed_subs:
                    return {"safe": False, "reason": "t27c " + sub["value"]
                            + " is not a read-only subcommand", "t27c_calls": []}
            program, expect_program = token["value"], False
            k += 1
            continue
        problem = _argument_problem(token, program)
        if problem:
            return {"safe": False, "reason": problem, "t27c_calls": []}
        args.append(token["value"])
        k += 1
    if expect_program:
        return {"safe": False, "reason": "no command after the last operator", "t27c_calls": []}
    end_segment()
    return {"safe": True, "reason": "", "t27c_calls": calls}


# --------------------------------------------------------------------------
# Reading GitHub, slowly
# --------------------------------------------------------------------------

class Gh:
    """`gh`, read-only, throttled, retried, cached under /tmp."""

    MIN_GAP = 0.4
    SEARCH_GAP = 2.2  # the search API admits 30 requests a minute

    def __init__(self, refresh: bool):
        self.refresh = refresh
        self.last = 0.0
        self.last_search = 0.0
        self.calls = 0
        self.cached = 0
        self.failures: list[str] = []
        CACHE_DIR.mkdir(parents=True, exist_ok=True)

    def _cache_path(self, argv: list[str]) -> Path:
        key = hashlib.sha1("\0".join(argv).encode()).hexdigest()[:20]
        return CACHE_DIR / (key + ".json")

    def run(self, argv: list[str], search: bool = False) -> tuple[int, str, str]:
        path = self._cache_path(argv)
        if not self.refresh and path.exists():
            try:
                saved = json.loads(path.read_text())
                if time.time() - saved["at"] < CACHE_TTL_SECONDS and saved["argv"] == argv:
                    self.cached += 1
                    return saved["rc"], saved["out"], saved["err"]
            except (ValueError, KeyError):
                pass
        rc, out, err = 1, "", ""
        for attempt in range(4):
            gap = self.SEARCH_GAP if search else self.MIN_GAP
            since = time.time() - (self.last_search if search else self.last)
            if since < gap:
                time.sleep(gap - since)
            proc = subprocess.run(["gh", *argv], capture_output=True, text=True)
            now = time.time()
            self.last = now
            if search:
                self.last_search = now
            self.calls += 1
            rc, out, err = proc.returncode, proc.stdout, proc.stderr
            if rc == 0:
                break
            lower = err.lower()
            if "rate limit" in lower or "abuse" in lower:
                time.sleep(60)
                continue
            if self.is_answer(err):
                break  # GitHub said no: an answer, not a failure
            time.sleep(2 ** (attempt + 1))
        if rc == 0 or self.is_answer(err):
            path.write_text(json.dumps({"argv": argv, "rc": rc, "out": out, "err": err,
                                        "at": time.time()}))
        else:
            self.failures.append(" ".join(argv[:4]) + ": " + err.strip()[:160])
        return rc, out, err

    @staticmethod
    def is_answer(err: str) -> bool:
        lower = err.lower()
        return any(word in lower for word in ("could not resolve", "not found", "http 404",
                                              "http 422", "no commit found"))

    def json(self, argv: list[str], search: bool = False):
        rc, out, _ = self.run(argv, search=search)
        if rc != 0:
            return None
        try:
            return json.loads(out)
        except ValueError:
            return None


def fetch_board(board_file: str | None) -> dict:
    if board_file:
        return json.loads(Path(board_file).read_text())
    with urllib.request.urlopen(BOARD_URL, timeout=30) as response:
        return json.loads(response.read().decode())


def select_cards(board: dict) -> list[dict]:
    return [
        card for card in board.get("cards", [])
        if card.get("column") == "review"
        and card.get("verdict") == "escalate"
        and card.get("criteria") == 0
    ]


def remote_queen_branches() -> dict[int, str]:
    proc = subprocess.run(["git", "ls-remote", "--heads", "origin", "queen-*"],
                          capture_output=True, text=True, cwd=ROOT)
    heads: dict[int, str] = {}
    for line in proc.stdout.splitlines():
        parts = line.split()
        if len(parts) != 2:
            continue
        match = re.fullmatch(r"refs/heads/queen-(\d+)", parts[1])
        if match:
            heads[int(match.group(1))] = parts[0]
    return heads


def closing_ref(text: str, number: int) -> bool:
    pattern = (r"\b(close[sd]?|fix(e[sd])?|resolve[sd]?)\s*:?\s+(gHashTag/t27)?#"
               + str(number) + r"\b")
    return re.search(pattern, text or "", re.IGNORECASE) is not None


def mentions(text: str, number: int) -> bool:
    return re.search(r"#" + str(number) + r"\b", text or "") is not None


def fetch_facts(gh: Gh, card: dict, branches: dict[int, str]) -> dict:
    number = card["number"]
    fields = "number,title,body,state,labels,closedByPullRequestsReferences"
    issue = gh.json(["issue", "view", str(number), "--repo", REPO, "--json", fields])
    if issue is None:  # an older gh, or the field refused: ask for less
        issue = gh.json(["issue", "view", str(number), "--repo", REPO,
                         "--json", "number,title,body,state,labels"])
    if issue is None:
        return {"number": number, "card": card, "error": "gh issue view failed"}

    pr_fields = "number,title,body,state,mergedAt,headRefName,baseRefName"
    found = gh.json(["pr", "list", "--repo", REPO, "--state", "merged", "--limit", "30",
                     "--search", str(number) + " in:body", "--json", pr_fields],
                    search=True) or []
    from_branch = gh.json(["pr", "list", "--repo", REPO, "--state", "all", "--limit", "10",
                           "--head", "queen-" + str(number), "--json", pr_fields]) or []
    by_number = {pr["number"]: pr for pr in found + from_branch}
    for ref in issue.get("closedByPullRequestsReferences") or []:
        ref_number = ref.get("number")
        if ref_number and ref_number not in by_number:
            view = gh.json(["pr", "view", str(ref_number), "--repo", REPO, "--json", pr_fields])
            if view:
                by_number[ref_number] = view
    closing_numbers = {ref.get("number") for ref in issue.get("closedByPullRequestsReferences") or []}

    prs = []
    for pr in sorted(by_number.values(), key=lambda p: p["number"]):
        text = (pr.get("title") or "") + "\n" + (pr.get("body") or "")
        how = []
        if pr["number"] in closing_numbers:
            how.append("closing-reference")
        if pr.get("headRefName") == "queen-" + str(number):
            how.append("bee-branch")
        if closing_ref(text, number):
            how.append("closing-keyword")
        if not how and mentions(text, number):
            how.append("mention")
        if not how:
            continue  # "N in:body" also matches the bare number in prose
        prs.append({
            "number": pr["number"],
            "title": pr.get("title"),
            "state": pr.get("state"),
            "mergedAt": pr.get("mergedAt"),
            "base": pr.get("baseRefName"),
            "head": pr.get("headRefName"),
            "how": how,
        })

    compare = None
    if number in branches:
        compare = gh.json(["api", "repos/" + REPO + "/compare/" + DEFAULT_BRANCH
                           + "...queen-" + str(number), "--jq",
                           "{status: .status, ahead_by: .ahead_by, behind_by: .behind_by,"
                           " files: [.files[]?.filename]}"])
    judged = card.get("judgedHead")
    judged_reachable = None
    if judged:
        if branches.get(number) == judged:
            judged_reachable = True
        else:  # the branch moved: is the judged commit still in the repository at all?
            rc, out, _ = gh.run(["api", "repos/" + REPO + "/commits/" + judged, "--jq", ".sha"])
            judged_reachable = rc == 0 and out.strip() == judged
    return {
        "number": number,
        "card": card,
        "judged_reachable": judged_reachable,
        "title": issue.get("title") or card.get("title") or "",
        "body": issue.get("body") or "",
        "state": issue.get("state"),
        "labels": [label.get("name") for label in issue.get("labels") or []],
        "prs": prs,
        "branch": branches.get(number),
        "compare": compare,
    }


# --------------------------------------------------------------------------
# What exists on master, and what a drafted check prints there
# --------------------------------------------------------------------------

class Master:
    """The local checkout at origin/master, read-only."""

    def __init__(self, t27c: str | None):
        self.rev = self._git(["rev-parse", "origin/" + DEFAULT_BRANCH]).strip() or "HEAD"
        head = self._git(["rev-parse", "HEAD"]).strip()
        self.worktree_is_master = head == self.rev
        self.t27c = t27c if t27c and Path(t27c).exists() else shutil.which("t27c")
        self.scratch = tempfile.mkdtemp(prefix="criteria-backfill-")
        self._exists: dict[str, bool] = {}

    @staticmethod
    def _git(argv: list[str]) -> str:
        proc = subprocess.run(["git", *argv], capture_output=True, text=True, cwd=ROOT)
        return proc.stdout if proc.returncode == 0 else ""

    def exists(self, path: str) -> bool:
        if path not in self._exists:
            proc = subprocess.run(["git", "cat-file", "-e", self.rev + ":" + path],
                                  capture_output=True, cwd=ROOT)
            self._exists[path] = proc.returncode == 0
        return self._exists[path]

    def measure(self, check: dict) -> dict:
        """Run ONE drafted check the way the runner would. Never an issue's own."""
        if not self.worktree_is_master:
            return {"status": "unmeasured", "reason": "the worktree is not at origin/master"}
        if not self.t27c:
            return {"status": "unmeasured", "reason": "no t27c binary"}
        return run_check(check, ROOT, self.t27c, self.scratch)


def run_check(check: dict, cwd, t27c: str, scratch: str, subcommands=None, env=None,
              timeout: int = 60, wrap=()) -> dict:
    """Run ONE criterion check in `cwd` the way the Queen's runner does.

    `command_safety` admits it first (`subcommands`: which t27c subcommands; the
    drafting subset by default), every `/tmp/t27-` target lands in `scratch`, and
    each t27c call in it must also exit 0 on its own. Used for drafted checks on
    master here, and by tools/bees/reviewer.py for an issue's own checks on a
    pull request's head; `wrap` (an argv prefix such as a sandbox) goes in front
    of every process it starts."""
    safety = command_safety(check["cmd"], subcommands)
    if not safety["safe"]:
        return {"status": "unrunnable", "reason": "unsafe: " + safety["reason"]}
    if env is None:
        env = dict(os.environ)
    env = dict(env, PATH=str(Path(t27c).parent) + os.pathsep + env.get("PATH", "/usr/bin:/bin"))
    command = check["cmd"].replace(SCRATCH_PREFIX, scratch + "/t27-")
    seen = []
    try:
        ran = subprocess.run([*wrap, "bash", "-c", command], capture_output=True, text=True,
                             cwd=str(cwd), env=env, timeout=timeout)
        out = ran.stdout.strip()
        seen += [ran.stdout, ran.stderr]
        for sub, file in safety["t27c_calls"]:
            alone = subprocess.run([*wrap, t27c, sub, file], capture_output=True, text=True,
                                   cwd=str(cwd), env=env, timeout=timeout)
            seen += [alone.stdout, alone.stderr]
            if alone.returncode != 0:
                return machine_blocked(seen) or {
                    "status": "failed", "reason": "t27c " + sub + " " + file
                    + " exits " + str(alone.returncode), "output": out[:120]}
    except subprocess.TimeoutExpired:
        return {"status": "unrunnable", "reason": f"timed out after {timeout} s"}
    op, expected = check["op"], check["expected"]
    if op == "equals":
        passed = out == expected.strip()
    elif op == "atLeast":
        passed = bool(re.fullmatch(r"-?\d+", out)) and int(out) >= int(expected)
    else:
        passed = expected not in ran.stdout
    if not passed and (blocked := machine_blocked(seen)):
        return blocked
    return {"status": "passed" if passed else "failed", "output": out[:120]}


# The machine running a check, not the code under it: a tool missing from PATH.
# Measured 2026-10-04 on #5756: with PATH=/usr/bin:/bin, `t27c test-report`
# prints "BLOCKED  zig not on PATH" and a `grep -c BLOCKED` criterion "fails".
MACHINE_BLOCKED = re.compile(r"[^\n]*(?:not on PATH|command not found)[^\n]*")


def machine_blocked(outputs) -> dict | None:
    m = MACHINE_BLOCKED.search("\n".join(o or "" for o in outputs))
    return m and {"status": "unrunnable", "reason": "this machine, not the code: " + m.group(0).strip()[:120]}


# --------------------------------------------------------------------------
# Drafting, from the issue's own words only
# --------------------------------------------------------------------------

# Headings under which an author states what done looks like, beyond the ten
# the Queen reads. A list under one of these is the author's own bar, written
# in a shape the parser cannot see; moving it is not inventing it.
EXTRA_DONE_HEADINGS = (
    "definition of done", "exit criteria", "exit criterion", "done means",
    "done if", "how to verify", "verification", "verify", "test plan",
    "expected outcome", "expected result", "success",
)
NOT_DONE_WORDS = ("already", "not done", "out of scope", "non-goal")
TOOLBELT_HEADINGS = ("the instruments",)
LIST_ITEM = re.compile(r"^(?:[-*+]|\d+[.)])\s+(?:\[[ xX]\]\s+)?(.*)$")
TITLE_TICKS = re.compile(r"`")


def scan_sections(body: str) -> list[dict]:
    """The body as (heading, lines) runs, fences kept apart and marked."""
    sections = [{"heading": "", "level": 0, "lines": []}]
    fenced = False
    for raw in body.replace("\r\n", "\n").replace("\r", "\n").split("\n"):
        stripped = raw.strip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            fenced = not fenced
            sections[-1]["lines"].append(("fence", raw))
            continue
        if not fenced and re.match(r"^#{1,6}\s", stripped):
            level = len(stripped) - len(stripped.lstrip("#"))
            sections.append({"heading": stripped.lstrip("#").strip(), "level": level,
                             "lines": []})
            continue
        sections[-1]["lines"].append(("code" if fenced else "text", raw))
    return sections


def done_heading(title: str) -> bool:
    """The Queen's ten match anywhere in the title (her rule, L230-253).

    The extra phrases match only at the START of the title: "verify" and
    "success" are common words, and "## The seal finding -- 730 seals, 0
    verify" is a finding, not a bar.
    """
    lower = title.lower()
    if any(word in lower for word in NOT_DONE_WORDS):
        return False
    if any(h in lower for h in CRITERIA_HEADINGS):
        return True
    lead = re.sub(r"^[^a-z\u0400-\u04ff]+", "", lower)
    return any(re.match(re.escape(h) + r"\b", lead) for h in EXTRA_DONE_HEADINGS)


def author_stated(body: str) -> list[dict]:
    """Items under a done-heading that the Queen's parser does not collect.

    A numbered list ("1."), a "+" list, or a prose paragraph. Each becomes a
    "- " bullet verbatim (a paragraph is joined onto one line).
    """
    found = []
    for section in scan_sections(body):
        if (not section["heading"] or section["level"] < 2
                or not done_heading(section["heading"])):
            continue
        paragraph: list[str] = []
        open_item: list[dict] = []

        def close_paragraph():
            if paragraph:
                found.append({"text": " ".join(paragraph), "from": section["heading"],
                              "shape": "paragraph"})
                paragraph.clear()

        for kind, raw in section["lines"]:
            if kind != "text":
                close_paragraph()
                open_item.clear()
                continue
            line = raw.strip()
            if not line:
                close_paragraph()
                open_item.clear()
                continue
            item = LIST_ITEM.match(line)
            if item:
                close_paragraph()
                open_item.clear()
                if item.group(1).strip():
                    entry = {"text": item.group(1).strip(), "from": section["heading"],
                             "shape": "list"}
                    found.append(entry)
                    open_item.append(entry)
                continue
            if open_item and raw[:1] in (" ", "\t"):
                # An indented line right under a list item continues it.
                open_item[0]["text"] += " " + line
                continue
            open_item.clear()
            if paragraph or not line.startswith(">"):
                paragraph.append(line)
        close_paragraph()
    return found


def lifted_checks(body: str) -> list[dict]:
    """Lines the runner could measure, written by the author outside a done-heading.

    Lifted ONLY when the line says what it prints today as well, e.g.
    "`cmd` prints `0` (today: 477)": a bare "`cmd` prints `477`" in a
    `## Measured` section is the defect's present state, not its target.
    """
    found = []
    for section in scan_sections(body):
        heading = section["heading"].lower()
        if any(h in heading for h in TOOLBELT_HEADINGS) or done_heading(section["heading"]):
            continue
        for kind, raw in section["lines"]:
            if kind != "text":
                continue
            line = raw.strip()
            item = LIST_ITEM.match(line)
            text = item.group(1).strip() if item else line
            checks = parse_criterion_checks(text)
            if not checks or "today" not in text.lower():
                continue
            if all(command_safety(c["cmd"], RUNNER_T27C_SUBCOMMANDS)["safe"] for c in checks):
                found.append({"text": text, "from": section["heading"] or "(body)"})
    return found




def obligation_sentences(body: str) -> list[str]:
    """The author's own MUST sentences, outside code and the toolbelt."""
    found = []
    for section in scan_sections(body):
        if any(h in section["heading"].lower() for h in TOOLBELT_HEADINGS):
            continue
        for kind, raw in section["lines"]:
            if kind != "text":
                continue
            line = raw.strip()
            item = LIST_ITEM.match(line)
            text = item.group(1).strip() if item else line
            if " must " not in (" " + text.lower() + " "):
                continue
            for sentence in re.split(r"(?<=[.!?])\s+", text):
                if " must " in (" " + sentence.lower() + " ") and len(sentence) > 12:
                    found.append(sentence.strip())
    seen, unique = set(), []
    for sentence in found:
        if sentence not in seen:
            seen.add(sentence)
            unique.append(sentence)
    return unique[:5]


def safe_path(path: str) -> bool:
    return bool(PATH_SAFE.match(path)) and not path.startswith("/") and ".." not in path.split("/")


def code_list(paths: list[str], limit: int = 4) -> str:
    shown = ", ".join("`" + p + "`" for p in paths[:limit])
    if len(paths) > limit:
        shown += " and " + str(len(paths) - limit) + " more"
    return shown


def draft(facts: dict, master: Master) -> dict:
    """The proposal for one non-landed issue: category B, C or D, and the text."""
    number, title, body = facts["number"], facts["title"], facts["body"]
    verdict = judge(body)
    paths = boundary_paths(body) or []
    criteria_now, _ = criteria_with_source(body)
    missing = [s for s in SPEC_SECTIONS if s in verdict["missing"]]
    present = [s for s in SPEC_SECTIONS if s not in verdict["missing"]]
    clean_title = TITLE_TICKS.sub("'", title).strip()
    notes = []
    if boundary_paths(body) is None:
        notes.append("no boundary section the Swift parser can see"
                     + (" (the body is CRLF: Swift does not split \\r\\n)" if "\r\n" in body else ""))
    absent = [p for p in paths if not master.exists(p)]
    if absent:
        notes.append("boundary paths absent on master: " + ", ".join(absent[:6]))

    # 1. The author's bar, in a shape the parser cannot see.
    stated = author_stated(body)
    # 2. Measurable lines the author wrote elsewhere, with their today-value.
    lifted = lifted_checks(body)
    # 3. Boundary specs: the parse guard, measured on master.
    guards, meaningful = [], []
    for path in paths:
        if not path.endswith(".t27") or not safe_path(path):
            continue
        line = ("`t27c spec-status " + path + "` does not print `NOPARSE` (the runner "
                "also fails it when `t27c spec-status` exits non-zero, as it does for "
                "a missing file)")
        check = parse_criterion_checks(line)[0]
        measured = master.measure(check)
        entry = {"text": line, "path": path, "on_master": measured}
        if measured["status"] == "failed":
            meaningful.append(entry)
        else:
            guards.append(entry)

    sources = {
        "author_stated": stated,
        "lifted_checks": lifted,
        "boundary_checks_failing_on_master": meaningful,
        "boundary_guards_true_on_master": guards,
    }
    has_heading = (has_section(body, SCENARIO_HEADINGS) or has_section(body, REQUIREMENT_HEADINGS)
                   or has_section(body, CRITERIA_HEADINGS)
                   or any(done_heading(s["heading"]) for s in scan_sections(body)))
    base = {
        "missing_now": verdict["missing"],
        "present_sections": present,
        "criteria_now": len(criteria_now),
        "boundary": paths,
        "sources": sources,
        "notes": notes,
    }
    if not (stated or lifted or meaningful):
        reason = ["no list or paragraph under a done-heading",
                  "no measurable line with its today-value"]
        if not paths:
            reason.append("no boundary to measure")
        elif guards:
            reason.append("every boundary spec already parses on master (a guard, "
                          "base-true: acceptedOnBaseTruthAlone would escalate it)")
        else:
            reason.append("no `.t27` in the boundary for `t27c` to measure")
        return dict(base, category="D", append=None,
                    reason="no criterion derivable: " + "; ".join(reason))

    category = "B" if (present or has_heading) else "C"
    blocks = []
    if "scenarios" in missing:
        where = code_list(paths) if paths else "its files"
        blocks.append("\n".join([
            "## User Scenarios & Testing",
            "",
            "- **Given** master without this change, **when** the commit for #"
            + str(number) + " lands on " + where
            + ", **then** every line under `## Success Criteria` below holds.",
        ]))
    if "requirements" in missing:
        lines = ["## Requirements", ""]
        index = 1
        if paths:
            lines.append("- FR-%03d: The change MUST stay inside `## Boundary` (%s)."
                         % (index, code_list(paths)))
            index += 1
        lines.append("- FR-%03d: The change MUST do what the title asks, within the scope "
                     "the body sets: \"%s\"." % (index, clean_title))
        index += 1
        for sentence in obligation_sentences(body):
            lines.append("- FR-%03d: %s" % (index, TITLE_TICKS.sub("'", sentence)
                                             if parse_criterion_checks(sentence) else sentence))
            index += 1
        blocks.append("\n".join(lines))
    criteria_lines = ["## Success Criteria", ""]
    for item in stated:
        criteria_lines.append("- " + item["text"])
    for item in lifted:
        criteria_lines.append("- " + item["text"])
    for item in meaningful + guards:
        criteria_lines.append("- " + item["text"])
    criteria_lines.append("- A reviewer reading the commit can name the change that settles "
                          "the title: \"" + clean_title + "\".")
    blocks.append("\n".join(criteria_lines))
    appended = "\n\n".join(blocks) + "\n"
    return dict(base, category=category, append=appended, reason=None)


def verify_append(body: str, appended: str) -> dict:
    """What the Queen would read from the body once the block is appended."""
    new_body = body.rstrip("\n") + "\n\n" + appended
    verdict = judge(new_body)
    items, source = criteria_with_source(new_body)
    before = judge(body)
    allowed = {"boundary"} if "boundary" in before["missing"] else set()
    problems = []
    if not items:
        problems.append("no criteria after appending")
    if source != "stated":
        problems.append("criteria source is " + source + ", not stated")
    extra = [m for m in verdict["missing"] if m not in allowed]
    if extra:
        problems.append("still missing: " + ", ".join(extra))
    machine = 0
    for item in items:
        checks = parse_criterion_checks(item)
        for check in checks:
            if not command_safety(check["cmd"], RUNNER_T27C_SUBCOMMANDS)["safe"]:
                problems.append("unsafe command: " + check["cmd"])
        machine += 1 if checks else 0
    try:
        appended.encode("ascii")
        ascii_only = True
    except UnicodeEncodeError:
        ascii_only = False
    return {
        "ok": not problems,
        "problems": problems,
        "criteria_after": len(items),
        "machine_checkable": machine,
        "missing_after": verdict["missing"],
        "ascii": ascii_only,
    }


def landed(facts: dict) -> list[dict]:
    return [
        pr for pr in facts.get("prs", [])
        if pr.get("mergedAt") and pr.get("base") == DEFAULT_BRANCH
        and any(h in pr["how"] for h in ("closing-reference", "closing-keyword", "bee-branch"))
    ]


def classify(facts: dict, master: Master) -> dict:
    card = facts["card"]
    compare = facts.get("compare") or {}
    record = {
        "number": facts["number"],
        "title": facts.get("title") or card.get("title"),
        "state": facts.get("state"),
        "has_bee_commit": bool(card.get("judgedHead")),
        "judged_head": card.get("judgedHead"),
        "judged_head_is_branch_tip": bool(card.get("judgedHead"))
        and facts.get("branch") == card.get("judgedHead"),
        "judged_head_reachable": facts.get("judged_reachable"),
        "branch": "queen-" + str(facts["number"]) if facts.get("branch") else None,
        "branch_ahead_of_master": compare.get("ahead_by"),
        "branch_files": (compare.get("files") or [])[:12],
        "board_needs": card.get("needs"),
        "merged_prs": [pr for pr in facts.get("prs", []) if pr.get("mergedAt")],
        "open_prs": [pr for pr in facts.get("prs", []) if pr.get("state") == "OPEN"],
    }
    if facts.get("error"):
        return dict(record, category="D", reason=facts["error"], append=None)
    twin_missing = judge(facts["body"])["missing"]
    record["twin_missing"] = twin_missing
    record["twin_agrees_with_board"] = (
        card.get("needs") is None or sorted(card.get("needs")) == sorted(twin_missing))
    proofs = landed(facts)
    if proofs:
        refs = ", ".join("#%d (merged %s, %s)" % (p["number"], p["mergedAt"][:10],
                                                  "+".join(p["how"])) for p in proofs)
        return dict(record, category="A", append=None, reason=None,
                    proposal="CLOSE #%d as completed: landed in %s." % (facts["number"], refs))
    proposal = draft(facts, master)
    record.update(proposal)
    if proposal.get("append"):
        record["verify"] = verify_append(facts["body"], proposal["append"])
    return record


# --------------------------------------------------------------------------
# Reports
# --------------------------------------------------------------------------

def write_reports(records: list[dict], meta: dict) -> None:
    counts = {k: sum(1 for r in records if r["category"] == k) for k in "ABCD"}
    with_commit = {k: sum(1 for r in records if r["category"] == k and r["has_bee_commit"])
                   for k in "ABCD"}
    payload = {"meta": meta, "counts": counts, "with_bee_commit": with_commit,
               "issues": records}
    REPORT_JSON.write_text(json.dumps(payload, indent=1, ensure_ascii=False))

    out = [
        "# Criteria backfill -- dry run",
        "",
        "Generated by `tools/queen/criteria_backfill.py --dry-run` at %s. Nothing was "
        "written to GitHub." % meta["generated"],
        "",
        "Cards: board column `review`, verdict `escalate`, `criteria == 0`. "
        "Board fetched from `%s`; master measured at `%s`." % (meta["board"], meta["master"]),
        "",
        "| category | meaning | issues | with a bee commit |",
        "|---|---|---|---|",
        "| A | landed via a merged PR into master: propose CLOSE | %d | %d |" % (counts["A"], with_commit["A"]),
        "| B | some spec sections present: append the missing ones | %d | %d |" % (counts["B"], with_commit["B"]),
        "| C | no spec sections: append the full block | %d | %d |" % (counts["C"], with_commit["C"]),
        "| D | no criterion derivable: a person must write one | %d | %d |" % (counts["D"], with_commit["D"]),
        "",
        "Twin check: the Python twin of `QueenSpecQuality.judge` agrees with the board's "
        "`needs` on %d of %d cards." % (meta["twin_agrees"], len(records)),
        "",
    ]
    for category in "ABCD":
        group = [r for r in records if r["category"] == category]
        if not group:
            continue
        out += ["## Category " + category, ""]
        for r in group:
            out.append("### #%d %s" % (r["number"], r["title"]))
            out.append("")
            judged = ("no" if not r["has_bee_commit"] else
                      "yes, the branch tip" if r.get("judged_head_is_branch_tip") else
                      "yes, but %s is no longer the branch tip%s" % (
                          (r.get("judged_head") or "")[:9],
                          "" if r.get("judged_head_reachable") else " and is gone from GitHub"))
            out.append("- state: %s; bee commit: %s; branch: %s (ahead of master: %s)" % (
                r.get("state"), judged, r.get("branch"), r.get("branch_ahead_of_master")))
            merged = r.get("merged_prs") or []
            out.append("- merged PR refs: " + (", ".join(
                "#%d %s->%s [%s]" % (p["number"], p.get("head"), p.get("base"), "+".join(p["how"]))
                for p in merged) or "none"))
            if r.get("open_prs"):
                out.append("- open PRs: " + ", ".join("#%d" % p["number"] for p in r["open_prs"]))
            if r.get("board_needs") is not None:
                out.append("- board needs: %s; twin: %s" % (", ".join(r["board_needs"]),
                                                             ", ".join(r.get("twin_missing", []))))
            for note in r.get("notes") or []:
                out.append("- note: " + note)
            if r.get("proposal"):
                out.append("- proposal: " + r["proposal"])
            if r.get("reason"):
                out.append("- " + r["reason"])
            if r.get("append"):
                v = r.get("verify") or {}
                out.append("- after appending: %d criteria (%d machine-checkable), missing %s, "
                           "parse check %s" % (v.get("criteria_after", 0), v.get("machine_checkable", 0),
                                               v.get("missing_after"), "ok" if v.get("ok") else
                                               "FAILED " + "; ".join(v.get("problems", []))))
                out += ["", "Text that would be appended:", "", "````markdown",
                        r["append"].rstrip("\n"), "````"]
            out.append("")
    REPORT_MD.parent.mkdir(parents=True, exist_ok=True)
    REPORT_MD.write_text("\n".join(out) + "\n")


def dry_run(args) -> int:
    board = fetch_board(args.board_file)
    cards = select_cards(board)
    if args.only:
        wanted = {int(n) for n in args.only.split(",")}
        cards = [c for c in cards if c["number"] in wanted]
    if args.limit:
        cards = cards[:args.limit]
    print("cards selected: %d" % len(cards), file=sys.stderr)
    gh = Gh(refresh=args.refresh)
    branches = remote_queen_branches()
    master = Master(args.t27c)
    records = []
    for index, card in enumerate(cards, 1):
        facts = fetch_facts(gh, card, branches)
        records.append(classify(facts, master))
        if index % 10 == 0:
            print("  %d/%d (gh calls %d, cached %d)" % (index, len(cards), gh.calls, gh.cached),
                  file=sys.stderr)
    meta = {
        "generated": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "board": args.board_file or BOARD_URL,
        "master": master.rev,
        "t27c": master.t27c,
        "gh_calls": gh.calls,
        "gh_cached": gh.cached,
        "gh_failures": gh.failures,
        "twin_agrees": sum(1 for r in records if r.get("twin_agrees_with_board")),
    }
    write_reports(records, meta)
    counts = {k: sum(1 for r in records if r["category"] == k) for k in "ABCD"}
    bad = [r["number"] for r in records if r.get("append") and not r["verify"]["ok"]]
    print("A=%(A)d B=%(B)d C=%(C)d D=%(D)d" % counts)
    print("report: %s\njson:   %s" % (REPORT_MD, REPORT_JSON))
    if gh.failures:
        print("gh failures: %d" % len(gh.failures), file=sys.stderr)
    if bad:
        print("drafts that FAIL the parse twin: %s" % bad, file=sys.stderr)
        return 1
    return 0


# --------------------------------------------------------------------------
# Self-test: no network, inline fixtures
# --------------------------------------------------------------------------

class FakeMaster(Master):
    def __init__(self, existing: set[str], parses: set[str]):
        self.rev = "fixture"
        self.existing = existing
        self.parses = parses

    def exists(self, path: str) -> bool:
        return path in self.existing

    def measure(self, check: dict) -> dict:
        path = check["cmd"].split()[-1]
        return {"status": "passed" if path in self.parses else "failed"}


def self_test() -> int:
    failures: list[str] = []

    def expect(name: str, got, want) -> None:
        if got != want:
            failures.append("%s: got %r, want %r" % (name, got, want))

    # The imported heading list is the Queen's ten (QueenSpecQuality.swift L176-187).
    expect("criteria headings", len(CRITERIA_HEADINGS), 10)
    expect("criteria headings EN", CRITERIA_HEADINGS[:4],
           ("success criteria", "acceptance criteria", "acceptance", "done when"))
    expect("criteria headings RU", CRITERIA_HEADINGS[-1], "\u043a\u0440\u0438\u0442\u0435\u0440\u0438\u0438")

    # QueenIssueBoundary: loop-strip, first path token, section end, nil vs [].
    expect("token comma", path_token("- `a/b.rs`,"), "a/b.rs")
    expect("token tab", path_token("-\trings/SR-00/Foo.swift"), "rings/SR-00/Foo.swift")
    expect("token ext", path_token("the file README.md here"), "README.md")
    expect("no boundary", boundary_paths("## Why\n- a/b.rs\n"), None)
    expect("empty boundary", boundary_paths("## Boundary\n\n## Next\n- a/b.rs"), [])
    expect("boundary ends", boundary_paths("## Boundary\n- `x/y.t27`\n## Next\n- a/b.rs"), ["x/y.t27"])
    expect("boundary RU", boundary_paths(RU_BOUNDARY + "\n- x/y.zig\n"), ["x/y.zig"])
    expect("boundary lowercase refused", boundary_paths("## boundary\n- x/y.zig\n"), None)
    expect("boundary CRLF is one Swift line", boundary_paths("## Why\r\nx\r\n## Boundary\r\n- a/b.rs\r\n"), None)
    expect("bullets CRLF still split", bullets("## Done when\r\n- a\r\n", CRITERIA_HEADINGS), ["a"])

    # QueenSpecQuality: the four checks and the criteria extraction.
    expect("measurable exits", has_measurable_outcome("it exits 0 then"), True)
    expect("measurable t27 path", has_measurable_outcome("`specs/x.t27`"), False)
    expect("measurable rs path", has_measurable_outcome("`bootstrap/src/x.rs`"), True)
    expect("measurable percent before space", has_measurable_outcome("50% done"), False)
    expect("measurable tests", has_measurable_outcome("11 tests"), True)
    expect("measurable cargo", has_measurable_outcome("`cargo test -p t27c`"), True)
    expect("scenario words", has_acceptance_scenario("given a then b"), True)
    expect("obligation must", has_obligation("it must hold"), True)
    expect("obligation MUST at line start", has_obligation("MUST hold"), False)
    expect("numbered list is not a bullet", bullets("## Success Criteria\n1. `a` prints `b`\n",
                                                    CRITERIA_HEADINGS), [])
    expect("checklist marker", bullets("### Acceptance\n- [x] done it\n", CRITERIA_HEADINGS), ["done it"])
    expect("bullets stop at heading", bullets("## Done when\n- a\n# Other\n- b\n", CRITERIA_HEADINGS), ["a"])
    # Drafting: which headings are a bar, and how a wrapped list item is read.
    expect("done heading: extra at start", done_heading("Verification"), True)
    expect("done heading: finding is not a bar", done_heading("The seal finding -- 730 seals, 0 verify"), False)
    expect("done heading: Queen's ten anywhere", done_heading("Wave 9 -- done when"), True)
    expect("done heading: not done", done_heading("Deliberately NOT done: the re-seal"), False)
    expect("author_stated skips H1", author_stated("# Fix, then verify\nprose\n"), [])
    expect("author_stated joins wrap",
           [x["text"] for x in author_stated("## Exit criteria\n- a b\n  c d\n- e\n")],
           ["a b c d", "e"])
    expect("fallback FR", criteria_with_source("x\n- FR-001: it MUST y\n"), (["FR-001: it MUST y"], "requirements"))
    expect("none", criteria_with_source("plain"), ([], "none"))
    full = ("## Boundary\n- a/b.rs\n## User Scenarios\n**Given** x **then** y\n"
            "## Requirements\n- FR-001: MUST z\n## Success Criteria\n- exits 0\n")
    expect("full spec", judge(full)["missing"], [])
    expect("full spec criteria", criteria_with_source(full), (["exits 0"], "stated"))

    # queen-criteria-run: the check grammar and the command allowlist.
    expect("parse prints", parse_criterion_checks("`t27c spec-status a.t27` prints `IMPLEMENTED`"),
           [{"cmd": "t27c spec-status a.t27", "op": "equals", "expected": "IMPLEMENTED"}])
    expect("parse more than", parse_criterion_checks("`grep -c x f` prints more than `3`")[0]["expected"], "4")
    expect("parse prose", parse_criterion_checks("`a` reports `b`"), [])
    expect("parse laptop path", parse_criterion_checks(
        "`./target/release/t27c parse a.t27` prints `ok`")[0]["cmd"], "t27c parse a.t27")
    expect("safe spec-status", command_safety("t27c spec-status specs/a.t27")["safe"], True)
    expect("safe calls", command_safety("t27c spec-status specs/a.t27")["t27c_calls"],
           [("spec-status", "specs/a.t27")])
    expect("unsafe cargo", command_safety("cargo test")["safe"], False)
    expect("unsafe battery", command_safety("t27c battery x.t27", RUNNER_T27C_SUBCOMMANDS)["safe"], False)
    expect("runner admits lint", command_safety("t27c lint x.t27", RUNNER_T27C_SUBCOMMANDS)["safe"], True)
    expect("unsafe battery in drafts", command_safety("t27c battery")["safe"], False)
    expect("scratch redirect", command_safety("t27c parse a.t27 > /tmp/t27-x && echo ok")["safe"], True)
    expect("other redirect", command_safety("echo x > /tmp/x")["safe"], False)
    expect("absolute path", command_safety("cat /etc/passwd")["safe"], False)
    expect("dotdot", command_safety("cat ../x")["safe"], False)
    expect("semicolon", command_safety("echo a; echo b")["safe"], False)

    # The classifier and the drafts, on fixtures that mirror the live shapes.
    master = FakeMaster(existing={"bootstrap/tests/a.rs", "specs/old.t27"}, parses={"specs/old.t27"})
    base_card = {"column": "review", "verdict": "escalate", "criteria": 0}

    def facts(number, body, title="Fix the thing", prs=(), judged=True):
        card = dict(base_card, number=number, title=title,
                    needs=[m for m in judge(body)["missing"]])
        if judged:
            card["judgedHead"] = "f" * 40
        return {"number": number, "card": card, "title": title, "body": body,
                "state": "OPEN", "labels": [], "prs": list(prs), "branch": "f" * 40,
                "compare": {"ahead_by": 1, "files": []}}

    landed_pr = {"number": 9, "title": "x", "state": "MERGED", "mergedAt": "2026-10-01T00:00:00Z",
                 "base": "master", "head": "queen-1", "how": ["bee-branch"]}
    a = classify(facts(1, "## Boundary\n- specs/new.t27\n", prs=[landed_pr]), master)
    expect("A category", a["category"], "A")
    expect("A proposal", a["proposal"].startswith("CLOSE #1"), True)
    mention_only = dict(landed_pr, head="other", how=["mention"])
    expect("mention is not landing", classify(facts(2, "## Boundary\n- specs/new.t27\n",
                                                    prs=[mention_only]), master)["category"], "C")
    other_base = dict(landed_pr, base="wave-loop-1")
    expect("merged elsewhere is not landing", classify(
        facts(3, "## Boundary\n- specs/new.t27\n", prs=[other_base]), master)["category"], "C")

    c = classify(facts(4, "Port it.\n\n## Boundary\n- specs/new.t27\n",
                       title="Port x.zig (Zig, 3 functions) to `specs/new.t27`"), master)
    expect("C category", c["category"], "C")
    expect("C verifies", c["verify"]["ok"], True)
    expect("C machine-checkable", c["verify"]["machine_checkable"] >= 1, True)
    expect("C ascii", c["verify"]["ascii"], True)
    expect("C title ticks neutralised", "`specs/new.t27`\"" in c["append"], False)

    b_body = ("The gate is wrong.\n\n## Done when\n\n1. `grep -c foo bootstrap/tests/a.rs` prints `0`\n"
              "2. the README says so\n\n## Boundary\n- bootstrap/tests/a.rs\n")
    b = classify(facts(5, b_body), master)
    expect("B category", b["category"], "B")
    expect("B verifies", b["verify"]["ok"], True)
    expect("B carries the author's numbered items", b["verify"]["criteria_after"], 3)
    expect("B two machine-checkable", b["verify"]["machine_checkable"], 1)

    d = classify(facts(6, "Audit of wave 12. 477 refusals measured.\n\n## Boundary\n"
                          "- bootstrap/tests/a.rs\n- specs/old.t27\n"), master)
    expect("D category", d["category"], "D")
    expect("D proposes nothing", d["append"], None)
    expect("D says why", "base-true" in d["reason"], True)

    lifted_body = ("## Measured\n- `grep -c x bootstrap/tests/a.rs` prints `0` (today: 12)\n"
                   "- `grep -c y bootstrap/tests/a.rs` prints `12`\n## Boundary\n- bootstrap/tests/a.rs\n")
    lifted = classify(facts(7, lifted_body), master)
    expect("lifted category", lifted["category"], "C")
    expect("lifted only the today-line", len(lifted["sources"]["lifted_checks"]), 1)
    expect("lifted verifies", lifted["verify"]["ok"], True)

    toolbelt = ("x\n\n## Boundary\n- bootstrap/tests/a.rs\n\n## The instruments\n"
                "- `t27c spec-status <spec>` prints `IMPLEMENTED` (today: PARTIAL)\n")
    expect("toolbelt is not the author", classify(facts(8, toolbelt), master)["category"], "D")

    # A block appended to a body whose boundary is last must not swallow it.
    tail = verify_append("x\n## Boundary\n- specs/new.t27", c["append"])
    expect("boundary-last verifies", tail["ok"], True)

    for line in failures:
        print("FAIL " + line)
    print("self-test: %d failure(s)" % len(failures))
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--dry-run", action="store_true", help="read GitHub, write the local report")
    parser.add_argument("--self-test", action="store_true", help="no network; fixtures only")
    parser.add_argument("--board-file", help="read the board from a saved JSON instead")
    parser.add_argument("--only", help="comma-separated issue numbers")
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--refresh", action="store_true", help="ignore the /tmp gh cache")
    parser.add_argument("--t27c", default=str(ROOT / "target" / "release" / "t27c"),
                        help="the compiler used to measure drafted checks on master")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if not args.dry_run:
        print("refusing: only --dry-run is implemented. Write mode (editing issues, closing "
              "them) is not implemented yet.", file=sys.stderr)
        return 2
    return dry_run(args)


if __name__ == "__main__":
    sys.exit(main())
