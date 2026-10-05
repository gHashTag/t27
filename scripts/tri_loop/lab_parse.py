#!/usr/bin/env python3
"""tri lab-parse -- parse WORKING-TREE specs on the Railway lab, show what the parser drops
Usage: tri lab-parse <spec.t27>... [--lines N] [--bodies] [--why]

Ships the files as they are on disk (uncommitted edits included) to the t27c
lab and runs `t27c parse-complete --show` and `t27c parse` on each. Prints the
dropped lines per spec, a PARSE-FAIL line for any spec that no longer
parses, and a VACUOUS line for invariants the generator could not lower
(the suite's no-vacuous-invariant phase, which a discard hides: fixing the
discard is what exposes it -- 2026-10-05, multi_lang_harness) with the
names of those invariants, and a
TYPECHECK-FAIL / GEN-VERILOG-FAIL line for the phases after that, marked NEW
only when the lab's checkout of the same path passes. Exit 1 on any
discard, parse failure, vacuous invariant, or NEW later-phase failure.

Why: t27c runs on the Railway lab, never on this machine (owner's rule,
2026-10-04). Without this, every native-dialect rewrite (skill
specs-native-dialect) meant a hand-rolled tar | base64 | railway ssh, with
two traps hit on 2026-10-05: zsh does not word-split "$FILES" into tar
arguments, and macOS tar adds ._ AppleDouble members that t27c then reports
as "stream did not contain valid UTF-8".

What this does NOT establish: Zig/C generation, seals, or the suite ratchet.
`nothing discarded` means the parser kept every token; push and read the
lab's suite/specs-generate gates for the rest. The lab binary is whatever
/data/target holds now (the last sha the lab built), not your branch's
parser -- a parser change in your diff is not exercised here.

--bodies prints each vacuous invariant's block from the working tree under
its name (every declaration of it -- a name may be declared twice), so a
rewrite needs no second pass through the file.

Lab address: the T27C_LAB_* variables of `tri t27b gen-check` (t27b.py
gen_check_env). From a worktree linked to another Railway project, `-s
t27c-lab` answers "Service not found"; set T27C_LAB_PROJECT plus
T27C_LAB_ENV / T27C_LAB_SERVICE to ids and they are passed as -p/-e/-s.
"""
import base64
import io
import os
import re
import shlex
import subprocess
import sys
import tarfile
import uuid
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from t27b import gen_check_env  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
CHUNK = 64000  # one ssh argument stays under MAX_ARG_STRLEN (see t27b.py gen-check)
MARK = "T27-LAB-PARSE"
# Phases the suite runs after no-vacuous-invariant. Each failure is compared
# with the lab's own checkout of the same path: NEW means this edit broke it.
LATER = (("typecheck", "TYPECHECK-FAIL"), ("gen-verilog", "GEN-VERILOG-FAIL"))
NOISE = ("config as code", "railway config migrate", "existing files keep working",
         "using ssh key")


def lab_env():
    env = gen_check_env()
    if not os.path.isdir(env["dir"]):
        env["dir"] = str(ROOT / "infra" / "t27c-lab")
    env["project"] = os.environ.get("T27C_LAB_PROJECT", "")
    return env


def ssh(env, cmd):
    argv = [env["railway"], "ssh"]
    if env["project"]:
        argv += ["-p", env["project"]]
    argv += ["-s", env["service"], "-e", env["environment"], cmd]
    for attempt in range(3):
        try:
            p = subprocess.run(argv, cwd=env["dir"], capture_output=True, text=True, timeout=180)
        except (OSError, subprocess.SubprocessError) as e:
            err = str(e)
            continue
        out = "\n".join(l for l in p.stdout.splitlines()
                        if not any(n in l.lower() for n in NOISE))
        if p.returncode == 0 or MARK in out:
            return p.returncode, out
        err = p.stderr.strip()
    sys.exit(f"tri lab-parse: lab unreachable after 3 tries: {err[:300]}")


def bundle(rels):
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz", format=tarfile.USTAR_FORMAT) as tar:
        for rel in rels:
            data = (ROOT / rel).read_bytes()
            info = tarfile.TarInfo(rel)
            info.size = len(data)
            tar.addfile(info, io.BytesIO(data))  # no xattrs, no ._ members
    return base64.b64encode(buf.getvalue()).decode("ascii")


def invariant_blocks(path, name):
    """Every `invariant <name>` block in path: header plus its indented body."""
    src = path.read_text(errors="replace").splitlines()
    head = re.compile(r"^\s*invariant\s+" + re.escape(name) + r"\s*:?\s*$")
    out = []
    for i, line in enumerate(src):
        if not head.match(line):
            continue
        ind = len(line) - len(line.lstrip())
        j = i + 1
        while j < len(src) and src[j].strip() and len(src[j]) - len(src[j].lstrip()) > ind:
            j += 1
        out.append((i + 1, src[i:j]))
    return out


def main(argv):
    lines = 40
    bodies = "--bodies" in argv
    why = "--why" in argv
    argv = [a for a in argv if a not in ("--bodies", "--why")]
    if "--lines" in argv:
        i = argv.index("--lines")
        lines = int(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.split("\n\n")[0].split("\n", 1)[1])
        return 0 if argv else 2
    rels = []
    for a in argv:
        p = Path(a).resolve()
        if not p.is_file():
            sys.exit(f"tri lab-parse: no such file: {a}")
        rels.append(str(p.relative_to(ROOT)))
    env = lab_env()
    b64 = bundle(rels)
    d = f"/tmp/tri-lab-parse-{uuid.uuid4().hex[:12]}"
    q = shlex.quote
    parts = [b64[i:i + CHUNK] for i in range(0, len(b64), CHUNK)]
    for part in parts[:-1]:
        ssh(env, f"mkdir -p {d} && printf %s {q(part)} >> {d}/b64")
    binp = q(env["bin"])
    head = f"head -{lines}" if lines > 0 else "cat"
    per = " ".join(
        f"{binp} parse-complete --show {q(r)} 2>&1 | {head}; "
        f"{binp} parse {q(r)} >/dev/null 2>&1 || echo {MARK} PARSE-FAIL {q(r)}; "
        f"echo {MARK} VACUOUS {q(r)} $({binp} gen {q(r)} 2>/dev/null | awk '/NOT CHECKED -- body was not lowered/{{print $3}}' | tr '\\n' ' ');"
        + "".join(
            f" {binp} {sub} {q(r)} >/dev/null 2>&1 || echo {MARK} LATER {tag} {q(r)}"
            f" $(cd {q(env['src'])} && {binp} {sub} {q(r)} >/dev/null 2>&1 && echo NEW || echo OLD);"
            # the first N bullets after "FAILED (N errors, M warnings):" are the errors
            + (f" {binp} {sub} {q(r)} 2>&1 | awk '/FAILED \\(/{{sub(/\\(/,\"\",$3); n=$3+0; next}}"
               f" n>0 && /^  - / && !/^  - warning:/{{print; n--}}' | cut -c1-240 | head -8 | sed 's/^/{MARK} WHY /';"
               if why else "")
            for sub, tag in LATER)
        for r in rels)
    rc, out = ssh(env,
                  f"trap 'rm -rf {d}' EXIT; mkdir -p {d}/w && printf %s {q(parts[-1])} >> {d}/b64 && "
                  f"base64 -d {d}/b64 | tar xzf - -C {d}/w && cd {d}/w && "
                  f"echo {MARK} bin $(git -C {q(env['src'])} log -1 --format=%h 2>/dev/null); {per}")
    bad = 0
    for line in out.splitlines():
        if line.startswith(MARK + " bin"):
            print(f"(lab parser built at {line.split()[-1] if len(line.split()) > 2 else '?'})")
            continue
        if line.startswith(MARK + " VACUOUS"):
            parts = line.split()
            rel, names = parts[2], parts[3:]
            if names:
                print(f"VACUOUS {rel}: {len(names)} invariant(s) not lowered (suite phase no-vacuous-invariant)")
                for name in names:
                    print(f"    {name}")
                    if bodies:
                        for ln, blk in invariant_blocks(ROOT / rel, name):
                            for k, t in enumerate(blk):
                                print(f"      {ln + k:5}| {t}")
                bad += 1
            continue
        if line.startswith(MARK + " LATER"):
            _, _, tag, rel, age = line.split(" ", 4)
            if age.strip() == "NEW":
                print(f"{tag} {rel}: NEW -- passes at the lab checkout, fails with this edit")
                bad += 1
            else:
                print(f"{tag} {rel}: also fails at the lab checkout (not this edit)")
            continue
        if line.startswith(MARK + " WHY "):
            print("    " + line[len(MARK) + 5:].strip())
            continue
        if line.startswith(MARK + " PARSE-FAIL"):
            print("PARSE-FAIL " + line.split(" ", 2)[2])
            bad += 1
            continue
        if "DISCARDED" in line:
            bad += 1
        print(line)
    print(f"tri lab-parse: {bad} problem(s) across {len(rels)} spec(s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
