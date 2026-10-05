#!/usr/bin/env python3
"""tri lab-exec -- EXECUTE a spec's invariants on the Railway lab (gen Zig, zig test)
Usage: tri lab-exec <spec.t27>... [-v] [--raw] [--ratchet | --bless] [--from-log FILE]

Ships the working-tree specs to the t27c lab, runs `t27c gen` on each and
`zig test` on the result. The generator emits every invariant (and test
assert) as a `comptime { // invariant: NAME ... }` block whose failure is
`@compileError("assertion failed")`, so compiling the Zig evaluates the
claims. Errors are sorted into:

  FALSE      an assert evaluated to false -- the spec states something
             untrue. Fix the spec (keep the original as a comment). A
             runtime `test` failure counts too: the generated assert helper
             is rewritten (in the copy on the lab) to print and go on
             instead of panicking, and any other panic (index out of bounds)
             is reported with the note that later tests never ran.
  UNDECLARED the spec calls a name no spec defines, or reads a field its own
             struct lacks -- NOT CHECKED comment, or define it.
  CODEGEN    the generator emitted Zig that does not compile (bad escape,
             @intCast on f64, comptime division by zero...). compiler.rs,
             i.e. a FROZEN_HASH freeze ceremony -- owner only. Report, do
             not work around it in the spec.
  IMPORT     an error inside a `use`d module's Zig (generated beside it,
             transitively). Not this spec's; run lab-exec on that module.

Identical errors are grouped with the invariants/tests they hit (-v: all\nof them plus the Zig line; --raw: zig's own log).
Exit 1 if any FALSE or UNDECLARED line is printed (CODEGEN alone exits 0:
it is not the spec author's to fix).

Ratchet (2026-10-05): --ratchet compares each spec's FALSE + UNDECLARED
count with docs/reports/lab_exec_false.json and exits 1 only if a count
ROSE (a spec missing from the file counts as 0); counts that fell are
listed so the baseline can be lowered -- unless a CODEGEN/IMPORT error
kept the file from compiling (no "tests passed"): that spec is "masked",
never "better", and --bless keeps its old count. --bless writes this run's counts
into that file (zeros removed, other specs kept). --from-log FILE reads a
saved lab-exec output instead of calling the lab -- the 1151-spec sweep
seeds the baseline this way without a second hour on the lab. A whole
sweep has 100+ known-false specs; without the ratchet, "exit 1" said
nothing about whether an edit made things worse.

Why: the suite's no-vacuous-invariant phase only checks that an invariant
was LOWERED, never that it holds. Running zig on the lab output on
2026-10-05 found constants.t27 phi_golden_conjugate (PHI + PHI_INV ==
1/PHI_INV, i.e. sqrt(5) == phi) false since the file's first commit, and
benchmark's half-coverage claim false as stated. This is the QuickCheck /
property-execution step, done with no compiler change.

What this does NOT establish: that a spec with no FALSE line is true. Zig
stops analysing a comptime block at its first error, so a CODEGEN or
UNDECLARED error hides every assert after it in that block; a forall that
was lowered to samples checks only those samples; and the lab's t27c is the
last sha it built, not your branch's compiler. Runtime `test` blocks are
reported by zig only when the whole file compiles.
"""
import os
import re
import shlex
import sys
import uuid
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lab_parse import CHUNK, ROOT, bundle, lab_env, ssh  # noqa: E402

MARK = "T27-LAB-PARSE"  # lab_parse.ssh treats a line with this mark as an answer
ERR = re.compile(r"^(\S+?\.zig):(\d+):\d+: error: (.*)$")
NOTE = re.compile(r"^(\S+?\.zig):(\d+):\d+: note: called at comptime here$")
# A struct the spec declares has no such field or member fn (zig: "struct
# 'T' has no member named 'plus'"): the spec's claim, not codegen. search,
# not match -- the message starts with the struct name (2026-10-05:
# TernaryWeight::plus() was filed as CODEGEN 17 times).
FIELD = re.compile(r"no (field|member) named '")
OWNER = re.compile(r"^\s*// (invariant|test): ?(.*)$")


def owners(zig_lines):
    """zig line number -> 'invariant NAME' / 'test NAME' of the block it sits in."""
    out, cur = {}, None
    for i, line in enumerate(zig_lines, 1):
        m = OWNER.match(line)
        if m:
            cur = f"{m.group(1)} {m.group(2).strip()}"
        elif line.startswith("}"):
            out[i] = cur
            cur = None
            continue
        out[i] = cur
    return out


def classify(log, zig_lines, own_file):
    """[(kind, owner, message, zig source line)] from one zig test log."""
    own = owners(zig_lines)
    src = lambda n: zig_lines[n - 1].strip() if 0 < n <= len(zig_lines) else ""  # noqa: E731
    lines = log.splitlines()
    found = []
    for i, line in enumerate(lines):
        m = ERR.match(line)
        if not m:
            continue
        # Zig spells anonymous struct types out in full (one message ran to
        # 20 KB on weights.t27); the head is enough to group and read.
        n, msg = int(m.group(2)), m.group(3)[:200]
        if m.group(1) != own_file:
            # An imported module's Zig: not this spec's claim, and its line
            # numbers mean nothing here. Tallied, not blamed.
            found.append(("IMPORT", m.group(1), msg, ""))
            continue
        if msg == "assertion failed":
            # The site is the next "called at comptime here" note.
            for nxt in lines[i + 1:i + 12]:
                k = NOTE.match(nxt)
                if k:
                    n = int(k.group(2))
                    break
            found.append(("FALSE", own.get(n), msg, src(n)))
        elif msg.startswith("use of undeclared identifier") or FIELD.search(msg):
            found.append(("UNDECLARED", own.get(n), msg, src(n)))
        else:
            # A codegen error inside a helper fn: blame the assert that called it.
            for nxt in lines[i + 1:i + 8]:
                k = NOTE.match(nxt)
                if k:
                    n = int(k.group(2))
                    break
                if ERR.match(nxt):
                    break
            found.append(("CODEGEN", own.get(n), msg, src(n)))
    return found


# A failed runtime `test` assert panics (the helper is noreturn), and zig
# stops the whole test binary at the first one: 2026-10-05 ternary_inference
# compiled, one test panicked, and lab-exec said "0 problems". Rewrite the
# helper in the generated Zig (not the generator) so a runtime failure prints
# a marker and the run goes on; the comptime branch is untouched. The
# newline is [_]u8{10}: GNU sed turns a \\n in the replacement into a raw
# newline inside the Zig string literal.
RUNTIME_SED = ('s/^fn __t27_assert_fail(comptime fmt: \\[\\]const u8, args: anytype) noreturn {/'
               'fn __t27_assert_fail(comptime fmt: []const u8, args: anytype) void {/;'
               's/^        @panic("assertion failed");/        std.debug.print("T27-RUNTIME-FALSE" ++ [_]u8{10}, .{});/')
# Zig panics print "thread N panic: ..."; a signal (a deref of an undefined
# pointer, 2026-10-05 bellman_ford/self_attention) prints only the fault
# name, and lab-exec used to call those specs "compiles".
PANIC = re.compile(r"thread \d+ panic: (.*)$|(General protection exception.*|Segmentation fault.*"
                   r"|Illegal instruction.*|Bus error.*)$")
TESTHDR = re.compile(r"^\d+/\d+ \S+?\.test\.(\S+?)\.\.\.")


def runtime_false(log):
    """[(test name, failed-assert dump)] from the patched helper's markers."""
    out, cur, buf = [], None, []
    for line in log.splitlines():
        m = TESTHDR.match(line)
        if m:
            cur, buf = m.group(1), []
            line = line[m.end():]
        k = PANIC.search(line)
        if k:
            # Any other panic (index out of bounds, overflow) still ends the
            # binary: the tests after this one never ran.
            out.append((cur, f"panic: {(k.group(1) or k.group(2))[:160]} -- run stopped, later tests not executed"))
            break
        if line.strip() == "T27-RUNTIME-FALSE" or line.endswith("T27-RUNTIME-FALSE"):
            out.append((cur, " ".join(b.strip() for b in buf if b.strip())[:200]))
            buf = []
        elif cur:
            buf.append(line)
    return out


USE = re.compile(r"^\s*use\s+([A-Za-z0-9_:]+)\s*;?\s*$", re.M)


def imports(rels):
    """The `use`d specs of rels, transitively: {zig stem: repo-relative path}.
    The generator splices most imported fns but still emits
    `@import("<stem>.zig")` for some modules (benchmark -> dataset.zig), so
    those files must sit beside the spec's Zig or the compile stops there."""
    out, todo = {}, list(rels)
    while todo:
        text = (ROOT / todo.pop()).read_text(errors="replace")
        for mod in USE.findall(text):
            rel = "specs/" + mod.replace("::", "/") + ".t27"
            stem = mod.split("::")[-1]
            if (ROOT / rel).is_file() and stem not in out and rel not in rels:
                out[stem] = rel
                todo.append(rel)
    return out


BASELINE = ROOT / "docs" / "reports" / "lab_exec_false.json"
VERDICT = re.compile(r"^(specs/\S+\.t27): (.*)$")


def counts_from_log(text):
    """{spec: (FALSE + UNDECLARED, masked)} from lab-exec's per-spec verdict
    lines. masked: a CODEGEN/IMPORT error kept the file from compiling, so
    no runtime test ran and a low count proves nothing (2026-10-05:
    bellman_ford went 1 -> 0 that way and the ratchet called it better)."""
    out = {}
    for line in text.splitlines():
        m = VERDICT.match(line)
        if not m:
            continue
        v = m.group(2)
        n = sum(int(k) for k in re.findall(r"(\d+) (?:FALSE|UNDECLARED)\b", v))
        if "t27c gen FAILED" in v:
            n = 1
        masked = "tests passed" not in v and bool(re.search(r"CODEGEN|IMPORT|gen FAILED", v))
        out[m.group(1)] = (n, masked)
    return out


def ratchet(counts, bless):
    import json
    base = json.loads(BASELINE.read_text()) if BASELINE.is_file() else {}
    held = {k for k, (n, masked) in counts.items() if masked and n < base.get(k, 0)}
    if bless:
        base.update({k: n for k, (n, _) in counts.items() if k not in held})
        base = {k: v for k, v in sorted(base.items()) if v}
        BASELINE.write_text(json.dumps(base, indent=1) + "\n")
        print(f"tri lab-exec: blessed {len(counts) - len(held)} spec(s) ({len(held)} masked, kept); "
              f"baseline holds {len(base)} spec(s), {sum(base.values())} problem(s)")
        return 0
    rose = {k: (base.get(k, 0), n) for k, (n, _) in counts.items() if n > base.get(k, 0)}
    fell = {k: (base.get(k, 0), n) for k, (n, _) in counts.items() if n < base.get(k, 0) and k not in held}
    for k, (a, b) in sorted(rose.items()):
        print(f"RATCHET UP   {k}: {a} -> {b}")
    for k, (a, b) in sorted(fell.items()):
        print(f"ratchet down {k}: {a} -> {b} (lower the baseline: --bless)")
    for k in sorted(held):
        print(f"masked       {k}: {base[k]} -> {counts[k][0]} but CODEGEN/IMPORT stopped the compile; "
              f"not counted as better")
    print(f"tri lab-exec ratchet: {len(rose)} spec(s) worse, {len(fell)} better, {len(held)} masked, "
          f"{len(counts) - len(rose) - len(fell) - len(held)} unchanged")
    return 1 if rose else 0


def main(argv):
    raw = "--raw" in argv
    verbose = "-v" in argv
    mode = "ratchet" if "--ratchet" in argv else "bless" if "--bless" in argv else None
    argv = [a for a in argv if a not in ("--raw", "-v", "--ratchet", "--bless")]
    if "--from-log" in argv:
        i = argv.index("--from-log")
        counts = counts_from_log(Path(argv[i + 1]).read_text(errors="replace"))
        rest = [str(Path(a).resolve().relative_to(ROOT)) for a in argv[:i] + argv[i + 2:]]
        if rest:
            counts = {k: v for k, v in counts.items() if k in rest}
        return ratchet(counts, mode == "bless")
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.split("\n\n")[0].split("\n", 1)[1])
        return 0 if argv else 2
    rels = []
    for a in argv:
        p = Path(a).resolve()
        if not p.is_file():
            sys.exit(f"tri lab-exec: no such file: {a}")
        rels.append(str(p.relative_to(ROOT)))
    env = lab_env()
    deps = imports(rels)
    b64 = bundle(rels + sorted(set(deps.values())))
    d = f"/tmp/tri-lab-exec-{uuid.uuid4().hex[:12]}"
    q = shlex.quote
    parts = [b64[i:i + CHUNK] for i in range(0, len(b64), CHUNK)]
    for part in parts[:-1]:
        ssh(env, f"mkdir -p {d} && printf %s {q(part)} >> {d}/b64")
    binp = q(env["bin"])
    # Imported modules first, each to <stem>.zig beside the specs' Zig.
    pre = " ".join(f"{binp} gen {q(r)} > {d}/{q(stem)}.zig 2>/dev/null;" for stem, r in deps.items())
    per = pre + " " + " ".join(
        f"echo {MARK} SPEC {q(r)}; "
        f"{binp} gen {q(r)} > {d}/o{k}.zig 2>{d}/g{k}.err || {{ echo {MARK} GENFAIL; head -5 {d}/g{k}.err; }}; "
        f"sed -i {q(RUNTIME_SED)} {d}/o{k}.zig; "
        f"echo {MARK} ZIG; cat {d}/o{k}.zig; echo {MARK} LOG; "
        f"(cd {d} && timeout 240 zig test --cache-dir {d}/c --global-cache-dir {d}/gc o{k}.zig 2>&1 | head -1500); "
        f"echo {MARK} END;"
        for k, r in enumerate(rels))
    _, out = ssh(env,
                 f"trap 'rm -rf {d}' EXIT; export PATH=/opt/zig:$PATH; mkdir -p {d}/w && "
                 f"printf %s {q(parts[-1])} >> {d}/b64 && base64 -d {d}/b64 | tar xzf - -C {d}/w && "
                 f"cd {d}/w && echo {MARK} bin $(git -C {q(env['src'])} log -1 --format=%h 2>/dev/null); {per}")
    bad = 0
    verdicts = []
    blocks = re.split(rf"^{MARK} SPEC ", out, flags=re.M)
    head = blocks[0].strip().split()
    if len(head) > 2:
        print(f"(lab t27c built at {head[-1]})")
    for k, blk in enumerate(blocks[1:]):
        rel = blk.split("\n", 1)[0].strip()
        zig = blk.split(f"{MARK} ZIG\n", 1)[-1].split(f"{MARK} LOG", 1)[0]
        log = blk.split(f"{MARK} LOG\n", 1)[-1].split(f"{MARK} END", 1)[0]
        if f"{MARK} GENFAIL" in blk:
            print(f"{rel}: t27c gen FAILED\n  " + blk.split(f"{MARK} GENFAIL", 1)[1].split(f"{MARK} ZIG")[0].strip())
            bad += 1
            verdicts.append(f"{rel}: t27c gen FAILED")
            continue
        found = classify(log, zig.splitlines(), f"o{k}.zig")
        found += [("FALSE", f"test {t} (runtime)", "assertion failed at runtime", dump)
                  for t, dump in runtime_false(log)]
        if raw:
            print(log)
        tally = {k: sum(1 for f in found if f[0] == k) for k in ("FALSE", "UNDECLARED", "CODEGEN", "IMPORT")}
        passed = re.search(r"All (\d+) tests passed", log)
        verdict = (f"compiles; {passed.group(0)}" if passed and not found
                   else ", ".join(f"{v} {k}" for k, v in tally.items() if v) or "compiles")
        print(f"{rel}: {verdict}")
        verdicts.append(f"{rel}: {verdict}")
        groups = {}  # (kind, msg) -> owners, first source line; FALSE first
        for kind, owner, msg, src in found:
            g = groups.setdefault((kind, msg), [[], src])
            if owner and owner not in g[0]:
                g[0].append(owner)
        order = ("FALSE", "UNDECLARED", "CODEGEN", "IMPORT")
        for (kind, msg), (who, src) in sorted(groups.items(), key=lambda kv: order.index(kv[0][0])):
            n = sum(1 for f in found if f[0] == kind and f[2] == msg)
            print(f"  {kind:10} x{n:<3} {msg}")
            shown = who if verbose or kind == "FALSE" else who[:4]
            for o in shown:
                print(f"             in {o}")
            if len(who) > len(shown):
                print(f"             ... and {len(who) - len(shown)} more (-v)")
            if src and (verbose or kind == "FALSE"):
                print(f"             zig: {src[:160]}")
        bad += tally["FALSE"] + tally["UNDECLARED"]
    print(f"tri lab-exec: {bad} spec-side problem(s) across {len(rels)} spec(s)")
    if mode:
        return ratchet(counts_from_log("\n".join(verdicts)), mode == "bless")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
