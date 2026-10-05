#!/usr/bin/env python3
"""tri lab-parse -- parse WORKING-TREE specs on the Railway lab, show what the parser drops
Usage: tri lab-parse <spec.t27>... [--lines N]

Ships the files as they are on disk (uncommitted edits included) to the t27c
lab and runs `t27c parse-complete --show` and `t27c parse` on each. Prints the
dropped lines per spec and a PARSE-FAIL line for any spec that no longer
parses. Exit 1 when any spec discards tokens or fails to parse.

Why: t27c runs on the Railway lab, never on this machine (owner's rule,
2026-10-04). Without this, every native-dialect rewrite (skill
specs-native-dialect) meant a hand-rolled tar | base64 | railway ssh, with
two traps hit on 2026-10-05: zsh does not word-split "$FILES" into tar
arguments, and macOS tar adds ._ AppleDouble members that t27c then reports
as "stream did not contain valid UTF-8".

What this does NOT establish: typecheck, generation, or the suite ratchet.
`nothing discarded` means the parser kept every token; push and read the
lab's suite/specs-generate gates for the rest. The lab binary is whatever
/data/target holds now (the last sha the lab built), not your branch's
parser -- a parser change in your diff is not exercised here.

Lab address: the T27C_LAB_* variables of `tri t27b gen-check` (t27b.py
gen_check_env). From a worktree linked to another Railway project, `-s
t27c-lab` answers "Service not found"; set T27C_LAB_PROJECT plus
T27C_LAB_ENV / T27C_LAB_SERVICE to ids and they are passed as -p/-e/-s.
"""
import base64
import io
import os
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


def main(argv):
    lines = 40
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
        f"{binp} parse {q(r)} >/dev/null 2>&1 || echo {MARK} PARSE-FAIL {q(r)};"
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
        if line.startswith(MARK + " PARSE-FAIL"):
            print("PARSE-FAIL " + line.split(" ", 2)[2])
            bad += 1
            continue
        if "DISCARDED" in line:
            bad += 1
        print(line)
    print(f"tri lab-parse: {len(rels) - bad if bad <= len(rels) else 0}/{len(rels)} clean")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
