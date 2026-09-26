#!/usr/bin/env python3
"""Port feeder: every hand-written file should have a .t27 counterpart.

WHY THIS EXISTS

The swarm's queue was fed by a scheduled task on the operator's laptop.
Measured 2026-09-17: between 18:03 and 05:43 that task fired **twice instead
of 36 times** — the machine was asleep — and the bees sat idle through the
night with nothing to eat.

The fuel it generates is the project's own stated goal: **every hand-written
file in this repository should be authored in `.t27`**, with the other
languages generated from it (the rule is written down in
`tools/oracle/run.sh`). Measured on master `6e8a494`: **870 hand-written
Python, Rust, TypeScript and C files have no `.t27` counterpart**. Unlike the
empty-bodies line, this one does not run dry.

Each issue it creates names one source file, quotes its function signatures
verbatim, and asks for `specs/port/<path>.t27` with a test per function.

EXCLUDED ON PURPOSE

`bootstrap/` — it is the stage-0 compiler, and `bootstrap/src/compiler.rs`
is under the `FROZEN_HASH` seal (FROZEN.md §5). Porting it is a human
ceremony, not a bee's turn. Also skipped: `target/`, `gen/`, `external/`,
`node_modules/`, `build/`, `outputs/`, `datasets/`, `test-ledger/`.

WHAT IT DELIBERATELY DOES NOT CLAIM

The acceptance criteria it writes are **structural** — the file exists, it
declares the same function names, the codegen emits no panic, the parser
recovers nothing, every function carries a test. Structural criteria cannot
express correctness: a `sha256` Sigma0 with the wrong operator once satisfied
all six of its criteria. The correctness gate is the review-side oracle,
which compiles the generated Zig and runs its tests. This script feeds; the
oracle judges.

VALIDATION ALREADY DONE

Three issues from this generator were created by hand as a probe — #3974,
#3975, #3976 — and all three were picked up by the Queen and dispatched to
bees, which confirms it accepts a `## Boundary` naming a file that does not
exist yet.

Run it from the repository root with a built t27c on PATH (or T27C_BIN):

    python3 tools/queen/queen_feed_port.py --dry-run --limit 3
    python3 tools/queen/queen_feed_port.py --when-idle --limit 8
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys

# ONE implementation of the machinery, not two. The empty-bodies feeder already
# knows how to run a command as the bee will run it, how to refuse an issue
# whose numbers do not reproduce, which files an open issue already claims, and
# how much room the swarm has. A second copy of that is the duplication this
# repository gates against in its specs.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from feed_empty_bodies import (  # noqa: E402
    BEE_T27C, OUTD, PREAMBLE, REPO, T27C, WORK,
    bee_shell, claims_hold, log, open_boundaries, queue_idle, run,
    scenario_and_requirements, sync_master,
)

# Language extensions we port
EXTENSIONS = {".py", ".rs", ".ts", ".c", ".h", ".cpp", ".cc", ".cxx", ".hpp", ".hh", ".hxx"}

# Directories to exclude (relative to repo root)
EXCLUDE_DIRS = {
    "bootstrap", "target", "gen", "external", "node_modules",
    "build", "outputs", "datasets", "test-ledger",
    ".git", ".github", "specs", "docs", "tools/queen", "tools/oracle",
}

# Files to exclude explicitly
EXCLUDE_FILES = {
    "bootstrap/src/compiler.rs",  # FROZEN_HASH seal (FROZEN.md §5)
}

# Maximum functions per issue (same ceiling as feed_untested.py)
MAX_FUNCTIONS = 8


def should_exclude(path: str) -> bool:
    """Check if a path should be excluded from porting."""
    # Check excluded directories
    parts = path.split(os.sep)
    for part in parts:
        if part in EXCLUDE_DIRS:
            return True
    # Check excluded files
    if path in EXCLUDE_FILES:
        return True
    return False


def has_t27_counterpart(rel_path: str) -> bool:
    """Check if a .t27 file exists for this source file."""
    # The .t27 counterpart would be at specs/port/<rel_path>.t27
    t27_path = os.path.join(WORK, "specs", "port", rel_path + ".t27")
    return os.path.exists(t27_path)


def extract_functions_python(text: str) -> list[tuple[str, str, int]]:
    """Extract function signatures from Python code.
    Returns list of (name, signature, line_number)."""
    # Match def and async def, with optional decorators
    pattern = r"(?m)^(\s*)(?:@\w+(?:\([^)]*\))?\n)*(\s*)(async\s+)?def\s+(\w+)\s*\([^)]*\)"
    results = []
    lines = text.splitlines(keepends=True)
    for i, line in enumerate(lines):
        m = re.match(pattern, line)
        if m:
            indent = m.group(1) + m.group(2)
            async_kw = m.group(3) or ""
            name = m.group(4)
            # Find the full signature (may span multiple lines)
            sig_lines = [line.rstrip()]
            depth = 0
            for ch in line:
                if ch == '(':
                    depth += 1
                elif ch == ')':
                    depth -= 1
            # If not closed on this line, keep reading
            j = i + 1
            while depth > 0 and j < len(lines):
                sig_lines.append(lines[j].rstrip())
                for ch in lines[j]:
                    if ch == '(':
                        depth += 1
                    elif ch == ')':
                        depth -= 1
                j += 1
            sig = " ".join(" ".join(sig_lines).split())
            results.append((name, sig, i + 1))
    return results


def extract_functions_rust(text: str) -> list[tuple[str, str, int]]:
    """Extract function signatures from Rust code."""
    # Match fn, pub fn, async fn, const fn, etc.
    pattern = r"(?m)^(\s*)(?:#\[[^\]]*\]\n)*(\s*)(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?(?:extern\s+\"\w+\"\s+)?fn\s+(\w+)\s*<[^>]*>\s*\([^)]*\)\s*(?:->\s*[^{;]+)?"
    results = []
    lines = text.splitlines(keepends=True)
    for i, line in enumerate(lines):
        m = re.match(pattern, line)
        if m:
            indent = m.group(1) + m.group(2)
            name = m.group(3)
            # Find the full signature
            sig_lines = [line.rstrip()]
            depth = 0
            angle_depth = 0
            for ch in line:
                if ch == '(':
                    depth += 1
                elif ch == ')':
                    depth -= 1
                elif ch == '<':
                    angle_depth += 1
                elif ch == '>':
                    angle_depth -= 1
            j = i + 1
            while (depth > 0 or angle_depth > 0) and j < len(lines):
                sig_lines.append(lines[j].rstrip())
                for ch in lines[j]:
                    if ch == '(':
                        depth += 1
                    elif ch == ')':
                        depth -= 1
                    elif ch == '<':
                        angle_depth += 1
                    elif ch == '>':
                        angle_depth -= 1
                j += 1
            sig = " ".join(" ".join(sig_lines).split())
            results.append((name, sig, i + 1))
    return results


def extract_functions_typescript(text: str) -> list[tuple[str, str, int]]:
    """Extract function signatures from TypeScript/JavaScript code."""
    # Match function declarations, arrow functions, method definitions
    pattern = r"(?m)^(\s*)(?:export\s+)?(?:async\s+)?(?:function\s+(\w+)\s*\([^)]*\)|const\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>|(\w+)\s*\([^)]*\)\s*:\s*[^{]+)"
    results = []
    lines = text.splitlines(keepends=True)
    for i, line in enumerate(lines):
        m = re.match(pattern, line)
        if m:
            indent = m.group(1)
            name = m.group(2) or m.group(3) or m.group(4)
            if name and not name.startswith("_"):
                # Find full signature
                sig_lines = [line.rstrip()]
                depth = 0
                for ch in line:
                    if ch == '(':
                        depth += 1
                    elif ch == ')':
                        depth -= 1
                j = i + 1
                while depth > 0 and j < len(lines):
                    sig_lines.append(lines[j].rstrip())
                    for ch in lines[j]:
                        if ch == '(':
                            depth += 1
                        elif ch == ')':
                            depth -= 1
                    j += 1
                sig = " ".join(" ".join(sig_lines).split())
                results.append((name, sig, i + 1))
    return results


def extract_functions_c(text: str) -> list[tuple[str, str, int]]:
    """Extract function signatures from C/C++ code."""
    # Match function definitions (not just declarations)
    pattern = r"(?m)^(\s*)(?:(?:static|inline|extern|__attribute__\s*\(\([^)]*\)\))\s+)*(\w+(?:\s*\*)?)\s+(\w+)\s*\([^)]*\)\s*(?:__attribute__\s*\(\([^)]*\)\)\s*)*\{?"
    results = []
    lines = text.splitlines(keepends=True)
    for i, line in enumerate(lines):
        m = re.match(pattern, line)
        if m:
            name = m.group(3)
            # Skip common non-function patterns
            if name in {"if", "while", "for", "switch", "catch", "return", "sizeof", "alignof", "typeof"}:
                continue
            # Find full signature
            sig_lines = [line.rstrip()]
            depth = 0
            for ch in line:
                if ch == '(':
                    depth += 1
                elif ch == ')':
                    depth -= 1
            j = i + 1
            while depth > 0 and j < len(lines):
                sig_lines.append(lines[j].rstrip())
                for ch in lines[j]:
                    if ch == '(':
                        depth += 1
                    elif ch == ')':
                        depth -= 1
                j += 1
            sig = " ".join(" ".join(sig_lines).split())
            results.append((name, sig, i + 1))
    return results


def extract_functions(rel_path: str, text: str) -> list[tuple[str, str, int]]:
    """Extract function signatures based on file extension."""
    ext = os.path.splitext(rel_path)[1].lower()
    if ext == ".py":
        return extract_functions_python(text)
    elif ext == ".rs":
        return extract_functions_rust(text)
    elif ext in {".ts", ".tsx", ".js", ".jsx"}:
        return extract_functions_typescript(text)
    elif ext in {".c", ".h", ".cpp", ".cc", ".cxx", ".hpp", ".hh", ".hxx"}:
        return extract_functions_c(text)
    return []


def find_portable_files() -> list[tuple[str, list[tuple[str, str, int]]]]:
    """Find all hand-written files without .t27 counterparts that have functions."""
    portable = []
    for root, dirs, files in os.walk(WORK):
        # Skip excluded directories
        dirs[:] = [d for d in dirs if not should_exclude(os.path.relpath(os.path.join(root, d), WORK))]
        
        for f in files:
            rel_root = os.path.relpath(root, WORK)
            rel_path = os.path.join(rel_root, f) if rel_root != "." else f
            
            if should_exclude(rel_path):
                continue
            
            ext = os.path.splitext(f)[1].lower()
            if ext not in EXTENSIONS:
                continue
            
            if has_t27_counterpart(rel_path):
                continue
            
            # Read file and extract functions
            abs_path = os.path.join(WORK, rel_path)
            try:
                with open(abs_path, "r", errors="replace") as handle:
                    text = handle.read()
            except Exception as e:
                log(f"skip {rel_path}: could not read: {e}")
                continue
            
            functions = extract_functions(rel_path, text)
            if functions:
                portable.append((rel_path, functions))
            else:
                log(f"note {rel_path}: no functions found")
    
    return portable


def build_issue(rel_path: str, functions: list[tuple[str, str, int]]) -> tuple[str, str] | None:
    """Build a single port issue for one source file."""
    names = [f[0] for f in functions]
    n = len(functions)
    
    # The target spec path
    spec_rel = f"specs/port/{rel_path}.t27"
    
    # Commands the bee will run to verify
    parse_cmd = f"{BEE_T27C} parse {spec_rel}"
    stubs_cmd = f"{BEE_T27C} gen {spec_rel} 2>&1 | grep -c 'not yet implemented'"
    have_cmd = f"{BEE_T27C} gen {spec_rel} 2>&1 | grep -cE '^fn " + "|".join(re.escape(n) for n in names) + r"\s*\('"
    test_cmd = f"grep -cE '^[[:space:]]*test[[:space:]]+(\"|[A-Za-z_])' {spec_rel}"
    status_cmd = f"{BEE_T27C} spec-status {spec_rel}"
    
    # The issue body
    title = f"Port {rel_path} to {spec_rel} ({n} function{'s' if n != 1 else ''})"
    
    L = [f"# {title}\n", "## Context\n",
         f"`{rel_path}` is a hand-written source file with {n} function{'s' if n != 1 else ''}. "
         f"The project rule (tools/oracle/run.sh) states that every hand-written file should be "
         f"authored in `.t27`, with other languages generated from it. This file has no `.t27` counterpart.\n",
         PREAMBLE,
         "## What to write\n",
         f"Create `{spec_rel}` declaring the same {n} function{'s' if n != 1 else ''}, "
         f"each with a test. Quoted verbatim from the source:\n"]
    
    for i, (nm, sig, line) in enumerate(functions, 1):
        L.append(f"{i}. line {line} - `{sig}`")
    
    L += ["",
          "Each function MUST have at least one `test` declaration. Either spelling is fine: "
          "`test name {` or `test \"name\" {`.\n",
          "## Acceptance criteria\n",
          f"- 1. `{parse_cmd}` prints nothing (parses cleanly)",
          f"- 2. `{stubs_cmd}` prints `0` (no `@panic(\"not yet implemented\")` in generated output)",
          f"- 3. `{have_cmd}` prints `{n}` (all {n} function names present in generated code)",
          f"- 4. `{test_cmd}` prints at least `{n}` (one test per function)",
          f"- 5. `{status_cmd}` prints `IMPLEMENTED`",
          "",
          "## Boundary\n", spec_rel]
    
    return title, "\n".join(L) + "\n"


def refuse_unshaped(title: str, body: str) -> list[str]:
    """Nothing here may open an issue the Queen could never dispatch."""
    from task_shape import problems
    return problems(body, set())


def population() -> list[tuple[str, list[tuple[str, str, int]]]]:
    """All portable files, sorted by function count (smallest first)."""
    portable = find_portable_files()
    portable.sort(key=lambda x: (len(x[1]), x[0]))
    return portable


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=8)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--when-idle", action="store_true")
    ap.add_argument("--runway", default="0",
                    help="top the queue up to this many dispatchable issues "
                         "whatever the lanes are doing; `auto` means twice the "
                         "lanes the swarm reports")
    args = ap.parse_args()

    if args.when_idle:
        want = queue_idle(args.runway)
        if want is None:
            log("could not read the swarm, so nothing was fed - and this run is RED "
                "rather than a green run that fed nothing")
            raise SystemExit(2)
        if want <= 0:
            log("swarm busy - nothing added")
            return
        args.limit = min(args.limit, want)

    sha = sync_master()
    portable = population()
    covered = open_boundaries()
    
    # Filter out files already covered by open issues
    todo = [(rel, fns) for rel, fns in portable if f"specs/port/{rel}.t27" not in covered]
    
    # Also filter by function count ceiling
    todo = [(rel, fns) for rel, fns in todo if len(fns) <= MAX_FUNCTIONS]
    
    # Report oversized files
    oversized = [(rel, len(fns)) for rel, fns in portable if len(fns) > MAX_FUNCTIONS]
    for rel, n in oversized:
        log(f"oversize {rel}: {n} functions (ceiling is {MAX_FUNCTIONS})")
    
    if not todo:
        log("no portable files left")
        return
    
    log(f"found {len(portable)} portable files, {len(covered)} already covered, "
        f"{len(todo)} todo, {len(oversized)} over ceiling")
    
    created = 0
    for rel, fns in todo[:args.limit]:
        issue = build_issue(rel, fns)
        if not issue:
            log(f"skip {rel}: could not build issue")
            continue
        title, body = issue
        
        # Validate issue shape
        problems = refuse_unshaped(title, body)
        if problems:
            log(f"skip {rel}: issue shape problems: {problems}")
            continue
        
        if args.dry_run:
            print(f"--- {title} ---")
            print(body)
            print()
        else:
            # Create issue via gh CLI
            issue_file = os.path.join(WORK, ".github", "ISSUE_TEMPLATE", "port.md")
            # Write to temp file and use gh
            import tempfile
            with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as tmp:
                tmp.write(body)
                tmp_path = tmp.name
            try:
                subprocess.run(["gh", "issue", "create", "--repo", REPO, "--title", title, "--body-file", tmp_path], check=True)
                log(f"created {title}")
            except subprocess.CalledProcessError as e:
                log(f"failed to create issue for {rel}: {e}")
            finally:
                os.unlink(tmp_path)
        
        created += 1
    
    log(f"fed {created} issue{'s' if created != 1 else ''}")


if __name__ == "__main__":
    main()