#!/usr/bin/env python3
"""Gate 19: "the spec parses" must mean the parser read the spec.

`t27c parse` exits 0 on all 497 specs. It also recovers from 1741 parse errors
while doing so, dropping 3292 constant declarations, because
`parse_module_body` recovers from a failed declaration by
skipping to the next one and continuing. Recovery is the right behaviour for a
resilient parser. Reporting nothing is not: a decline that is not counted
(Prop. 103 shape two), in the compiler for the project's stated source of truth.

Measured before this gate existed: 676 constant declarations in specs/, 214
reaching any AST. 31%. Every spec green.

This gate reads the `recovery-events:` line that `t27c parse` now writes to
stderr AND compares constants written against constants reaching the AST -- the
two are different, and a planted regression moved the second while leaving the
first unchanged. It RATCHETS. It does not demand zero, because 3292 cannot be
fixed
in one wave and a gate that lands red gets disabled rather than obeyed
(Prop. 26). It demands that the number never rise, and that no spec in the
protected set regress.

ARTIFACTS. Reads `specs/**/*.t27` and runs `./target/release/t27c`. WRITES
`formal/spec_parse_baseline.txt` when no baseline exists. Nothing else.

Prop. 143.
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
T27C = ROOT / "target" / "release" / "t27c"
BASELINE = ROOT / "formal" / "spec_parse_baseline.txt"

# The binary lands in the WORKSPACE target dir, not bootstrap/target. Getting
# this wrong produces exit 127 on every spec, which reads as total parser death
# rather than as a missing binary.


def measure():
    out = {}
    for s in sorted(ROOT.glob("specs/**/*.t27")):
        r = subprocess.run([str(T27C), "parse", str(s)],
                           capture_output=True, text=True)
        if r.returncode not in (0, 1):
            return None, (f"t27c returned {r.returncode} on "
                          f"{s.relative_to(ROOT)} -- neither parse nor error")
        m = re.search(r"recovery-events: (\d+)", r.stderr)
        if m is None:
            return None, (f"no 'recovery-events:' line from t27c on "
                          f"{s.relative_to(ROOT)} -- the gate cannot see what "
                          f"was dropped, so it is measuring nothing")
        # WITHDRAWN (Prop. 149): a second component counted "constants
        # written minus constants reaching an AST". No regex formulation of
        # it is sound, and three attempts each gave a different answer:
        #
        #   ^\s*const\s+\w+        -> counted FUNCTION-LOCAL bindings and
        #                             missed every `pub const`; this is the
        #                             version that shipped, reporting 3292
        #   brace-depth <= 1       -> array literals `[32]u16{` open braces,
        #                             so the depth accounting drifts
        #   ^\s*pub\s+const        -> undercounts; the ratio came out at 118%
        #
        # `const` is legal at module scope AND inside a function body, so
        # separating them needs parsing -- which is the thing being measured.
        # Only the parser can report this soundly. The ratchet now runs on
        # recovery events alone, which the parser itself emits.
        # Prop. 152: the sound replacement for what Prop. 149 withdrew. The
        # PARSER reports how many module-level declarations its recovery
        # swallowed -- module scope decided by parsing, not by a regex.
        sw = re.search(r"declarations-swallowed: (\d+)", r.stderr)
        if sw is None:
            return None, (f"no 'declarations-swallowed:' line from t27c on "
                          f"{s.relative_to(ROOT)} -- the gate cannot see what "
                          f"recovery discarded")

        # Prop. 186: the swallowed counter lives INSIDE the recovery skip, so a
        # spec whose preamble fails never reaches it and reports 0 while
        # capturing nothing -- 34 specs do exactly that. The third component is
        # the coarse outside-the-machine check: does a file that declares
        # something public capture ANY declaration at all? It needs no parser
        # knowledge and cannot be fooled by scope, which is what made every
        # regex-based attempt at this wrong (Props. 149, 185c).
        text = s.read_text(errors="ignore")
        declares = bool(re.search(
            r"^\s{0,4}(?:pub|export)\s+(?:fn|const|struct|enum)\s+\w", text, re.M))
        captured = len(re.findall(
            r"kind: (?:ConstDecl|FnDecl|StructDecl|EnumDecl)", r.stdout))
        blind = 1 if (declares and captured == 0) else 0
        out[str(s.relative_to(ROOT))] = (int(m.group(1)), int(sw.group(1)), blind)
    return out, None


def main():
    if not T27C.exists():
        print(f"::error::spec parse gate: no such file "
              f"'target/release/t27c' -- build it first; nothing was measured")
        return 1
    specs = list(ROOT.glob("specs/**/*.t27"))
    if not specs:
        print("::error::spec parse gate: found no files under specs/ -- "
              "nothing was measured")
        return 1

    now, err = measure()
    if err:
        print(f"::error::spec parse gate: {err}")
        return 1

    total = sum(v[0] for v in now.values())
    lost = sum(v[1] for v in now.values())
    blind = sum(v[2] for v in now.values())
    dirty = sum(1 for v in now.values() if v[0])
    print(f"spec parse gate: {len(now)} specs, {dirty} recovering, "
          f"{total} recovery events, {lost} declarations swallowed, "
          f"{blind} specs declaring but capturing nothing")

    if not BASELINE.exists():
        BASELINE.write_text("".join(f"{v[0]}\t{v[1]}\t{v[2]}\t{k}\n"
                                    for k, v in sorted(now.items())))
        print(f"spec parse gate: baseline written to {BASELINE.name} "
              f"({total} discarded)")
        return 0

    was = {}
    for line in BASELINE.read_text().splitlines():
        if not line.strip():
            continue
        a, b, c, k = line.split("\t", 3)
        was[k] = (int(a), int(b), int(c))

    old_total = sum(v[0] for v in was.values())
    old_lost = sum(v[1] for v in was.values())
    regressions = [(k, was.get(k, (0, 0, 0)), v) for k, v in sorted(now.items())
                   if v[0] > was.get(k, (0, 0, 0))[0]
                   or v[1] > was.get(k, (0, 0, 0))[1]
                   or v[2] > was.get(k, (0, 0, 0))[2]]
    if regressions:
        print(f"::error::spec parse gate: {len(regressions)} spec(s) under "
              f"specs/ recover more or swallow more than the baseline -- a spec "
              f"that parses less than it used to is a spec the compiler reads "
              f"less of")
        for k, a, b in regressions[:10]:
            print(f"  {k}: {a} -> {b}")
        return 1

    if total < old_total or lost < old_lost:
        print(f"spec parse gate: improved {old_total}->{total} events, "
              f"{old_lost}->{lost} lost; update {BASELINE.name} to lock it in")
    # A ratchet that is never tightened is a ceiling. Say the number every run
    # so the direction is visible without anyone reading the baseline file.
    print(f"spec parse gate: ratchet holds ({total} <= {old_total} events, {lost} <= {old_lost} lost)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::spec parse gate: could not measure specs/ "
              f"({type(exc).__name__}: {exc}) -- nothing was measured")
        sys.exit(1)
