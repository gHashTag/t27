#!/usr/bin/env python3
"""tri t27b gen-check asks the t27c lab, and never guesses (#6231).

gen/c/tri/t27b/steward.c is generated (L2) by `t27c gen-c
specs/tri/t27b/steward.t27`. t27c runs on the Railway lab t27c-lab, never on
the Mac, so each t27b PR used to prove by hand, over `railway ssh`, that the
committed C is byte-identical to what master's t27c emits. `tri t27b
gen-check` does it: the spec goes to the lab as base64, the lab answers the
sha256 and length of its gen-c output, and the verdict is SAME (0), DIFFERS
(1) or UNREACHABLE (2).

No network here. A fake runner stands in for the lab:
  1. SAME when the lab's digest is the local file's;
  2. DIFFERS when it is not, including a same-length change;
  3. UNREACHABLE when ssh fails, when the lab answers without a result, and
     when gen-c itself exits non-zero -- never SAME or DIFFERS;
  4. the spec reassembled from the shipped chunks is the local spec, byte for
     byte, both in one command and split under a small chunk size, and the
     last command removes the scratch dir;
  5. the CLI exit codes 0/1/2 through env T27B_GEN_CHECK_FAKE fixtures;
  6. mutation control: a copy of gen_check that compares only lengths calls
     the same-length change SAME, and check 2 catches it.
"""

import base64
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import types
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "scripts/tri_loop/t27b.py"
sys.path.insert(0, str(TOOL.parent))
import t27b  # noqa: E402

SPEC = "specs/tri/t27b/steward.t27"
GEN = "gen/c/tri/t27b/steward.c"
MARK = t27b.GEN_CHECK_MARK
fails = 0


def check(ok, what):
    global fails
    print(("ok      " if ok else "FAIL    ") + what)
    fails += 0 if ok else 1


def lab_answer(content, rc=0, err=""):
    """What the lab prints for gen-c output `content`."""
    return (f"{MARK} bin /data/target/release/t27c\n{MARK} mtime 2026-10-04T18:21:13Z\n"
            f"{MARK} binsha {'f' * 64}\n{MARK} src {'8' * 40} 2026-10-04T18:18:36+00:00\n"
            f"{MARK} rc {rc}\n{MARK} sha {hashlib.sha256(content).hexdigest()}\n"
            f"{MARK} bytes {len(content)}\n{MARK} err {err}\n")


class FakeLab:
    """Records every command; reassembles the base64 the commands carry."""

    def __init__(self, content, rc=0, ssh_rc=0, answer=None):
        self.content, self.rc, self.ssh_rc, self.answer = content, rc, ssh_rc, answer
        self.cmds = []

    def __call__(self, cmd):
        self.cmds.append(cmd)
        if MARK not in cmd:
            return 0, "", ""
        if self.answer is not None:
            return self.ssh_rc, self.answer, "connection reset"
        return self.ssh_rc, lab_answer(self.content, self.rc, "parse error" if self.rc else ""), ""

    def shipped(self):
        parts = [m.group(1) for c in self.cmds for m in re.finditer(r"printf %s '?([A-Za-z0-9+/=]*)'? >>", c)]
        return base64.b64decode("".join(parts))


local = (ROOT / GEN).read_bytes()
spec = (ROOT / SPEC).read_bytes()

# 1. SAME
lab = FakeLab(local)
r = t27b.gen_check(SPEC, GEN, runner=lab)
check(r["verdict"] == "SAME", f"SAME when the lab's digest is the local file's ({r['verdict']})")
check(r["lab"]["src_commit"] == "8" * 40 and r["lab"]["mtime"] == "2026-10-04T18:21:13Z" and not r["lab"]["stale"],
      "the lab binary's provenance is read back (path, mtime, source commit)")

# 2. DIFFERS, also when the length is the same
flipped = local[:-2] + bytes([local[-2] ^ 1]) + local[-1:]
for name, content in (("one byte more", local + b"\n"), ("one byte flipped, same length", flipped)):
    r = t27b.gen_check(SPEC, GEN, runner=FakeLab(content))
    check(r["verdict"] == "DIFFERS", f"DIFFERS: {name} ({r['verdict']})")


# 3. UNREACHABLE, never a guess
def ssh_down(cmd):
    raise t27b.Unreadable("railway ssh: No such file or directory")


for name, runner in (("ssh cannot start", ssh_down),
                     ("ssh dropped before the result", FakeLab(local, ssh_rc=1, answer=f"{MARK} bin /x\n")),
                     ("gen-c exit 1 on the lab", FakeLab(local, rc=1)),
                     ("digest garbled", FakeLab(local, answer=lab_answer(local).replace(MARK + " sha ", MARK + " sha zz")))):
    r = t27b.gen_check(SPEC, GEN, runner=runner)
    check(r["verdict"] == "UNREACHABLE" and r.get("reason"), f"UNREACHABLE: {name} ({r['verdict']}: {r.get('reason')})")
r = t27b.gen_check(SPEC, "gen/c/tri/t27b/no-such.c", runner=FakeLab(local))
check(r["verdict"] == "UNREACHABLE", f"UNREACHABLE: the local gen file is missing ({r['verdict']})")

# 4. transport: one command, and chunked
for chunk, want in ((t27b.GEN_CHECK_CHUNK, 1), (1000, -(-len(base64.b64encode(spec)) // 1000))):
    lab = FakeLab(local)
    r = t27b.gen_check(SPEC, GEN, runner=lab, chunk=chunk, token="t0ken")
    check(len(lab.cmds) == want and lab.shipped() == spec and r["verdict"] == "SAME",
          f"chunk {chunk}: {len(lab.cmds)} command(s), the spec arrives byte for byte")
    check(all(len(c) < 100000 for c in lab.cmds), "every command stays under the 128 KiB argument cap")
    check("trap 'rm -rf /tmp/t27b-gen-check-t0ken' EXIT" in lab.cmds[-1]
          and f"gen-c {SPEC}" in lab.cmds[-1], "the last command runs gen-c and removes its scratch dir")

# 5. CLI exit codes through T27B_GEN_CHECK_FAKE
with tempfile.TemporaryDirectory() as tmp:
    for name, fx, code, word in (("same", {"stdout": lab_answer(local)}, 0, "SAME "),
                                 ("differs", {"stdout": lab_answer(flipped)}, 1, "DIFFERS lab "),
                                 ("unreachable", {"unreachable": "lab is down"}, 2, "UNREACHABLE lab is down")):
        p = os.path.join(tmp, name + ".json")
        with open(p, "w") as f:
            json.dump(fx, f)
        out = subprocess.run([sys.executable, str(TOOL), "gen-check"], capture_output=True, text=True,
                             env={**os.environ, "T27B_GEN_CHECK_FAKE": p})
        check(out.returncode == code and out.stdout.startswith(word),
              f"CLI {name}: exit {out.returncode}, {out.stdout.splitlines()[0] if out.stdout else out.stderr[:80]}")
    out = subprocess.run([sys.executable, str(TOOL), "gen-check", "--json"], capture_output=True, text=True,
                         env={**os.environ, "T27B_GEN_CHECK_FAKE": os.path.join(tmp, "same.json")})
    check(json.loads(out.stdout).get("verdict") == "SAME", "CLI --json answers a dict with the verdict")

# 6. mutation control: compare lengths only
src = TOOL.read_text()
needle = 'out["lab_sha256"] == out["local_sha256"] and '
check(src.count(needle) == 1, "mutation: the digest comparison is present exactly once")
mutant = types.ModuleType("t27b_mutant")
mutant.__file__ = str(TOOL)
exec(compile(src.replace(needle, ""), "t27b_mutant", "exec"), mutant.__dict__)
r = mutant.gen_check(SPEC, GEN, runner=FakeLab(flipped))
check(r["verdict"] == "SAME", f"mutation: a length-only comparison calls a flipped byte SAME ({r['verdict']}), "
      "which check 2 rejects")

if fails:
    print(f"{fails} check(s) failed")
    sys.exit(1)
print("every check held: tri t27b gen-check answers SAME, DIFFERS or UNREACHABLE, and never guesses")
