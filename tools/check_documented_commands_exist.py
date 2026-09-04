#!/usr/bin/env python3
"""Every `t27c <sub>` a live document tells you to run must be a real subcommand.

Written after `t27c gen-zig` -- named in `docs/TRI_NET_WHITEPAPER.md` and in a
`nona-01` mapping table -- cost a full corpus run. There is no such subcommand;
the Zig generator is `t27c gen`. clap exits 2 for an unknown subcommand, and 2
is also this repo's "could not run" code, so 650 specs "failing to generate"
read as a measurement instead of as 650 usage errors.

The truth is the binary's own `--help`, not a list in this file: a second list
drifts from the enum, and a gate whose ruler drifts reports the ruler.

Population. Live surface only:

    README.md, .claude/skills/**, docs/**

minus the families that are records of a dated moment rather than instructions
for now -- `docs/now/**` (dated in the filename), `docs/reports/**` (named by
wave) and `docs/theory/IGLA-FORMAL-RESULTS.md` (anchored per section). A record
that names a command which has since been renamed is not lying; a README is.
The exclusion is printed with the number of hits it drops, because a filter
nobody can see is indistinguishable from a filter that removes the finding.

Excused inside the population: a hit that the document itself declares is not
real yet -- on its own line, or in the nearest heading above it. "Proposed issue
spine #11-#25" over a table of `tri run` is not a false claim about today.

Exit 0 clean, 1 on a finding, 2 when it could not be run at all.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# A backticked invocation: `t27c sub`, `./t27c sub`, `scripts/t27c sub`.
HIT = re.compile(r"`(?:\./)?(?:scripts/)?t27c ([a-z][a-z0-9]*(?:-[a-z0-9]+)*)")

# The document says, in its own words, that this one is not built yet.
DECLARED = re.compile(
    r"(?i)not implemented|does not exist|never existed|no such subcommand|"
    r"new subcommand|proposed|planned|long-term|future|would be|was to |"
    r"do not assume it exists"
)

EXCLUDED_PREFIXES = ("docs/now/", "docs/reports/")
EXCLUDED_FILES = ("docs/theory/IGLA-FORMAL-RESULTS.md",)

# The ruler is checked before it is used. These six were run by hand against the
# binary this gate was written for; gen-zig is the one that started it.
MUST_EXIST = ("parse", "gen", "seal", "corpus", "backlog", "parse-complete")
MUST_NOT_EXIST = ("gen-zig",)


def refuse(msg: str) -> None:
    print(f"check_documented_commands_exist: {msg}", file=sys.stderr)
    print("  Exit 2 = could not run, not a clean tree.", file=sys.stderr)
    sys.exit(2)


def binary() -> Path:
    env = os.environ.get("TRI_T27C")
    if env:
        p = Path(env)
        if not p.is_file() or not os.access(p, os.X_OK):
            refuse(f"TRI_T27C points at {env}, which is not an executable file.")
        return p
    p = ROOT / "target" / "release" / "t27c"
    if not p.is_file():
        refuse(
            "t27c is not built (looked for target/release/t27c).\n"
            "  Build it with `cargo build --release -p t27c`, or set TRI_T27C."
        )
    return p


def real_subcommands(exe: Path) -> set[str]:
    try:
        out = subprocess.run(
            [str(exe), "--help"], capture_output=True, text=True, timeout=120
        )
    except (OSError, subprocess.SubprocessError) as exc:
        refuse(f"could not run `{exe} --help`: {exc}")
    text = out.stdout + out.stderr
    if "Commands:" not in text:
        refuse("`t27c --help` printed no `Commands:` block, so nothing was read.")
    body = text.split("Commands:", 1)[1]
    names = set()
    for line in body.splitlines():
        m = re.match(r"^  ([a-z][a-z0-9-]*)(?:\s|$)", line)
        if m:
            names.add(m.group(1))
    return names


def calibrate(real: set[str]) -> None:
    missing = [c for c in MUST_EXIST if c not in real]
    present = [c for c in MUST_NOT_EXIST if c in real]
    if missing:
        refuse(
            f"the ruler is wrong before the docs are read: {', '.join(missing)} "
            "was not parsed out of --help, and each of those is a real "
            "subcommand. A stale binary reads as a documentation defect."
        )
    if present:
        refuse(f"--help claims {', '.join(present)} exists; the parse is too loose.")


def population() -> tuple[list[Path], list[Path]]:
    live, excluded = [], []
    candidates = [ROOT / "README.md"]
    for base in (".claude/skills", "docs"):
        candidates += sorted((ROOT / base).rglob("*.md"))
    for p in candidates:
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith(EXCLUDED_PREFIXES) or rel in EXCLUDED_FILES:
            excluded.append(p)
        else:
            live.append(p)
    return live, excluded


def heading_above(lines: list[str], idx: int) -> str:
    for j in range(idx, -1, -1):
        if lines[j].startswith("#"):
            return lines[j]
    return ""


def scan(paths: list[Path], real: set[str]) -> tuple[list[tuple], int, int]:
    findings, excused, seen = [], 0, 0
    for p in paths:
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            refuse(f"could not read {p}: {exc}")
        for i, line in enumerate(lines):
            for sub in HIT.findall(line):
                seen += 1
                if sub in real:
                    continue
                if DECLARED.search(line) or DECLARED.search(heading_above(lines, i)):
                    excused += 1
                    continue
                findings.append((p.relative_to(ROOT).as_posix(), i + 1, sub, line.strip()))
    return findings, excused, seen


def nearest(sub: str, real: set[str]) -> str:
    stem = sub.split("-")[0]
    near = sorted(c for c in real if c.startswith(stem))
    return f"  nearest real: {', '.join(near[:4])}" if near else ""


def main() -> int:
    self_check = "--self-check" in sys.argv
    exe = binary()
    real = real_subcommands(exe)
    calibrate(real)
    print(f"t27c --help lists {len(real)} subcommands  ({exe})")

    if self_check:
        # A checker that cannot fail is a green light with no bulb behind it.
        fake = ["`t27c gen-zig` is how you generate Zig."]
        hits = [s for s in HIT.findall(fake[0]) if s not in real]
        ok_finds = hits == ["gen-zig"]
        excused_line = "`t27c gen-zig` -- proposed, not implemented"
        ok_excuse = bool(DECLARED.search(excused_line))
        print(f"  self-check  finds a dead command:      {'ok' if ok_finds else 'BROKEN'}")
        print(f"  self-check  excuses a declared one:    {'ok' if ok_excuse else 'BROKEN'}")
        return 0 if (ok_finds and ok_excuse) else 2

    live, excluded = population()
    dropped, _, _ = scan(excluded, real)
    findings, excused, seen = scan(live, real)

    # An empty population passes every rule ever written. A tree with no live
    # documents, or live documents that invoke t27c nowhere, is a tree this
    # gate did not read -- and "ok" would be the same word it prints when it
    # read 232 files and found nothing wrong.
    if not live:
        refuse("no live documents were found to scan.")
    if seen == 0:
        refuse(
            f"{len(live)} live document(s) were read and not one names a t27c "
            "subcommand. That is the matcher failing, not the docs being clean."
        )

    print(f"live documents scanned:  {len(live)}")
    print(
        f"records not scanned:     {len(excluded)}  "
        f"(docs/now, docs/reports, IGLA-FORMAL-RESULTS) -- "
        f"{len(dropped)} dead-command mention(s) inside them, left alone"
    )
    print(f"excused as declared:     {excused}")

    if not findings:
        print("\nok: every t27c subcommand a live document names exists.")
        return 0

    print(f"\nFAIL: {len(findings)} live mention(s) of a subcommand that does not exist.\n")
    for rel, ln, sub, text in findings:
        print(f"  {rel}:{ln}  `t27c {sub}`")
        print(f"    {text[:150]}")
        tip = nearest(sub, real)
        if tip:
            print(tip)
    print(
        "\nEither the name is wrong, or the document should say the command is\n"
        "not built yet -- on the line, or in the heading above it."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
