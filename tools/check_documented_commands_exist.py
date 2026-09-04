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

Scope, stated because it is narrower than the title sounds. This reads the FIRST
token after the binary name and nothing after it. `tri skill seal` and
`tri skill commit` both pass, because `skill` resolves -- and neither exists:
`tri skill` offers check, refs, claims, renumber, begin, end. Measured on the
README's own nine-step cycle, which every change is told to follow: this gate
can see 2 dead steps and there are 4. Second-level resolution is available
(each command's own --help lists its subcommands) and is not built here; naming
the gap is honest, half-building it is not.

Exit 0 clean, 1 on a finding, 2 when it could not be run at all.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# A backticked invocation: `t27c sub`, `./tri sub`, `scripts/tri sub`.
HIT = re.compile(
    r"`(?:\./)?(?:scripts/)?(t27c|tri) ([a-z][a-z0-9]*(?:-[a-z0-9]+)*)"
)

# Inside a fenced block there are no backticks to key on, and the fenced surface
# is the one a reader COPIES. Measured on the tree this was extended for: 276
# fenced invocations, 52 distinct names, 19 of them dead -- 36.5% against 18.5%
# on the backticked surface the first version gated. The README's own Quick
# Start was in the 19.
# ANCHORED at the start of the line, after an optional prompt. A fenced block
# holds prose as often as commands -- a table row, a quoted sentence, a shell
# comment -- and a matcher that reads anywhere in the line reported `t27c was`,
# `t27c is` and `tri binary` on its first run: 110 findings of which the
# majority were English. What a reader COPIES starts the line.
FENCED_HIT = re.compile(
    r"^\s*(?:[$>] )?(?:\./)?(?:scripts/)?(t27c|tri) ([a-z][a-z0-9]*(?:-[a-z0-9]+)*)"
)
FENCE = re.compile(r"^\s*(```|~~~)")

# The document says, in its own words, that this one is not built yet.
DECLARED = re.compile(
    r"(?i)not implemented|not built|does not exist|never existed|no such subcommand|"
    r"has ever existed|new subcommand|proposed|planned|long-term|future|"
    r"would be|was to |do not assume it exists"
)

# English corrects AFTER it names the thing: "`t27c gen-zig` ... There is no
# such subcommand." A window of the line alone reads the name and misses the
# sentence that retracts it two lines down. This gate went red on master for
# exactly that, on the skill section describing this gate -- so the window is
# the mention's own paragraph, not its own line.
DECLARE_LOOKAHEAD = 3

EXCLUDED_PREFIXES = ("docs/now/", "docs/reports/")
EXCLUDED_FILES = ("docs/theory/IGLA-FORMAL-RESULTS.md",)

# The ruler is checked before it is used. These six were run by hand against the
# binary this gate was written for; gen-zig is the one that started it.
MUST_EXIST = ("parse", "gen", "seal", "corpus", "backlog", "parse-complete")
MUST_NOT_EXIST = ("gen-zig",)

# `tri` resolves through four surfaces and the last one is a fallthrough, so a
# name is real if ANY of them answers. Calibrated the same way: five that must
# resolve, four that must not -- and the must-resolve list is what caught the
# ruler the first time this was run, when the binaries were not where
# scripts/tri looks and every name read as dead.
# The `tri` half is REPORTED, not enforced, and held by a down-only ceiling.
# Measured when it was wired: 101 live mentions of 24 names that resolve on none
# of the four surfaces, spread across 13 document families -- `tri git` 23,
# `tri spec` 14, `tri queen` 9. They are not typos: docs/nona-03-manifest and
# .claude/skills/tri describe an intended product CLI. Landing a gate red by 101
# is how a gate gets muted, and excluding those families by path would be an
# exclusion made by argument. So: the list prints every run, and the number can
# only fall. A new dead `tri` name still fails.
MAX_DEAD_TRI = 97

TRI_MUST_EXIST = ("now", "wave", "gen", "skill", "seal")
TRI_MUST_NOT_EXIST = ("gen-zig", "gen-dir", "spec", "git")


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


def subcommands_of(exe: Path) -> set[str]:
    """The names in a clap `Commands:` block. Shared by both binaries."""
    try:
        out = subprocess.run(
            [str(exe), "--help"], capture_output=True, text=True, timeout=120
        )
    except (OSError, subprocess.SubprocessError):
        return set()
    text = out.stdout + out.stderr
    if "Commands:" not in text:
        return set()
    body = text.split("Commands:", 1)[1]
    names = set()
    for line in body.splitlines():
        m = re.match(r"^  ([a-z][a-z0-9-]*)(?:\s|$)", line)
        if m:
            names.add(m.group(1))
    return names


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


def tri_surfaces(t27c_real: set[str]) -> set[str]:
    """Every name `tri X` can resolve to, from all four surfaces.

    scripts/tri forwards anything it does not handle to t27c, so a t27c
    subcommand IS a working `tri` name. Missing that fallthrough would report
    155 false deaths.
    """
    names = set(t27c_real)

    front = ROOT / "scripts" / "tri"
    if not front.is_file():
        refuse("scripts/tri is missing, so `tri` names cannot be resolved.")
    for line in front.read_text(encoding="utf-8", errors="replace").splitlines():
        m = re.match(r"^  ([a-z][a-z0-9-]*)\)", line)
        if m:
            names.add(m.group(1))

    loop = ROOT / "scripts" / "tri_loop"
    if loop.is_dir():
        for f in loop.glob("*.py"):
            names.add(f.stem.replace("_", "-"))

    exe = os.environ.get("TRI_BIN")
    if exe:
        # An override that is wrong must refuse, not fall through to whatever is
        # on disk. Falling through describes a tree the caller is not using --
        # the same defect `tri which` had, caught there by its own control.
        p = Path(exe)
        if not p.is_file() or not os.access(p, os.X_OK):
            refuse(f"TRI_BIN points at {exe}, which is not an executable file.")
        cands = [p]
    else:
        cands = []
    cands += [ROOT / "target" / "release" / "tri", ROOT / "target" / "debug" / "tri"]
    for c in cands:
        if c.is_file() and os.access(c, os.X_OK):
            names |= subcommands_of(c)
            break
    else:
        refuse(
            "the Rust `tri` binary is not built (looked for target/release/tri).\n"
            "  Build it with `cargo build --release -p tri`, or set TRI_BIN.\n"
            "  Without it 37 working subcommands would read as dead."
        )
    return names


def calibrate_tri(real: set[str]) -> None:
    missing = [c for c in TRI_MUST_EXIST if c not in real]
    present = [c for c in TRI_MUST_NOT_EXIST if c in real]
    if missing:
        refuse(
            f"the `tri` ruler is wrong before the docs are read: "
            f"{', '.join(missing)} did not resolve, and each of those works. "
            "A binary that is not where scripts/tri looks makes every name dead."
        )
    if present:
        refuse(f"`tri` resolves {', '.join(present)}, which do not exist; the parse is too loose.")


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


def declared_near(lines: list[str], idx: int) -> bool:
    """The document declares this one unbuilt, within its own paragraph.

    Stops at a blank line: a retraction belongs to the sentence that names the
    command, and a window that runs past the paragraph would excuse a live
    instruction sitting above an unrelated one.
    """
    for j in range(idx, min(idx + DECLARE_LOOKAHEAD + 1, len(lines))):
        if j > idx and not lines[j].strip():
            break
        if DECLARED.search(lines[j]):
            return True
    return False


def scan(paths: list[Path], real: dict[str, set[str]]) -> tuple[list[tuple], int, int]:
    findings, excused, seen = [], 0, 0
    for p in paths:
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            refuse(f"could not read {p}: {exc}")
        in_fence = False
        fence_opened_at = 0
        for i, line in enumerate(lines):
            if FENCE.match(line):
                if not in_fence:
                    fence_opened_at = i
                in_fence = not in_fence
                continue
            if in_fence:
                m = FENCED_HIT.match(line)
                hits = [(m.group(1), m.group(2), "fenced")] if m else []
                # Prose introducing a block sits above the fence, not inside it.
                declare_at = fence_opened_at
            else:
                hits = [(b, c, "inline") for b, c in HIT.findall(line)]
                declare_at = i
            for which, sub, where in hits:
                seen += 1
                if sub in real[which]:
                    continue
                if (
                    declared_near(lines, declare_at)
                    or declared_near(lines, i)
                    or DECLARED.search(heading_above(lines, i))
                ):
                    excused += 1
                    continue
                findings.append(
                    (p.relative_to(ROOT).as_posix(), i + 1, which, sub, where, line.strip())
                )
    return findings, excused, seen


def nearest(sub: str, real: set[str]) -> str:
    stem = sub.split("-")[0]
    near = sorted(c for c in real if c.startswith(stem))
    return f"  nearest real: {', '.join(near[:4])}" if near else ""


def main() -> int:
    self_check = "--self-check" in sys.argv
    exe = binary()
    t27c_real = real_subcommands(exe)
    calibrate(t27c_real)
    tri_real = tri_surfaces(t27c_real)
    calibrate_tri(tri_real)
    real = {"t27c": t27c_real, "tri": tri_real}
    print(f"t27c --help lists {len(t27c_real)} subcommands  ({exe})")
    print(f"`tri` resolves {len(tri_real)} names across four surfaces")

    if self_check:
        # A checker that cannot fail is a green light with no bulb behind it.
        hits = [c for b, c in HIT.findall("`t27c gen-zig` is how you generate Zig.")]
        ok_finds = hits == ["gen-zig"]
        fenced = [c for b, c in FENCED_HIT.findall("./scripts/tri gen-zig specs/x.t27")]
        ok_fenced = fenced == ["gen-zig"]
        ok_excuse = bool(DECLARED.search("`t27c gen-zig` -- proposed, not implemented"))
        para = ["`t27c gen-zig` is named in the whitepaper.", "There",
                "is no such subcommand."]
        ok_para = declared_near(para, 0)
        stops = declared_near(["`t27c gen-zig` is named here.", "",
                               "There is no such subcommand."], 0)
        ok_fwd = "parse" in tri_real  # the forward-anything fallthrough
        for label, ok in (
            ("finds a dead command", ok_finds),
            ("finds one inside a fence", ok_fenced),
            ("excuses a declared one", ok_excuse),
            ("reads the paragraph, not the line", ok_para),
            ("stops at the blank line", not stops),
            ("`tri` inherits t27c's names", ok_fwd),
        ):
            print(f"  self-check  {label:36} {'ok' if ok else 'BROKEN'}")
        every = ok_finds and ok_fenced and ok_excuse and ok_para and not stops and ok_fwd
        return 0 if every else 2

    live, excluded = population()
    dropped, _, _ = scan(excluded, real)
    findings, excused, seen = scan(live, real)

    if not live:
        refuse("no live documents were found to scan.")
    if seen == 0:
        refuse(
            f"{len(live)} live document(s) were read and not one names a t27c or "
            "tri subcommand. That is the matcher failing, not the docs being clean."
        )

    print(f"live documents scanned:  {len(live)}")
    print(
        f"records not scanned:     {len(excluded)}  "
        f"(docs/now, docs/reports, IGLA-FORMAL-RESULTS) -- "
        f"{len(dropped)} dead-command mention(s) inside them, left alone"
    )
    print(f"invocations read:        {seen}   excused as declared: {excused}")

    if not findings:
        print("\nok: every t27c and tri subcommand a live document names exists.")
        return 0 if MAX_DEAD_TRI == 0 else 1

    t27c_bad = [f for f in findings if f[2] == "t27c"]
    tri_bad = [f for f in findings if f[2] == "tri"]

    if tri_bad:
        names = sorted({f[3] for f in tri_bad})
        print(
            f"\n`tri` names that resolve nowhere: {len(tri_bad)} mention(s), "
            f"{len(names)} distinct  (ceiling {MAX_DEAD_TRI})"
        )
        for rel, ln, _w, sub, where, text in tri_bad:
            print(f"  {rel}:{ln}  `tri {sub}`  [{where}]")
        print("  names: " + ", ".join(names))

    if len(tri_bad) != MAX_DEAD_TRI:
        verb = "rose" if len(tri_bad) > MAX_DEAD_TRI else "fell"
        print(
            f"\nFAIL: dead `tri` mentions {verb} {MAX_DEAD_TRI} -> {len(tri_bad)}.\n"
            "  Up: a document names a `tri` command that resolves on none of the\n"
            "  four surfaces. Down: good -- lower MAX_DEAD_TRI in the same commit."
        )
        return 1

    if not t27c_bad:
        print(
            f"\nok: every t27c subcommand a live document names exists, and the "
            f"{len(tri_bad)} dead `tri` mentions are the recorded ceiling."
        )
        return 0

    findings = t27c_bad
    inline = sum(1 for f in findings if f[4] == "inline")
    print(
        f"\nFAIL: {len(findings)} live mention(s) of a subcommand that does not "
        f"exist  ({inline} inline, {len(findings) - inline} inside a fence).\n"
    )
    for rel, ln, which, sub, where, text in findings:
        print(f"  {rel}:{ln}  `{which} {sub}`  [{where}]")
        print(f"    {text[:150]}")
        tip = nearest(sub, real[which])
        if tip:
            print(tip)
    print(
        "\nEither the name is wrong, or the document should say the command is\n"
        "not built yet -- on the line, in the paragraph, or in the heading above it."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
