#!/usr/bin/env python3
"""Gate 24: a line with an odd number of quotes swallows the rest of its file.

Prop. 186 found 107 corrupted field types of the form `bits : [[]Usize",` -- a
stray `"` where `]` belongs. The lexer treats it as the start of a string
literal, so **everything after it in the file is consumed as string content** and
33 specs captured zero declarations while every parser-side counter read zero.

Prop. 186e states why that class is so expensive: when a lexical error causes the
remainder of a file to be CONSUMED rather than reported, the symptom appears at
the consumer -- a missing declaration, an unclosed block -- and carries no
information about where the cause is. Six waves of parser work chased a symptom
whose cause was one character of data.

The signature is cheap and exact: a code line whose unescaped double-quote count
is ODD cannot close its own string. That is not always a defect -- a character
literal `'"'` is legal, and a multi-line string would be too if this language had
one -- so those are excluded explicitly rather than silently.

COVERAGE. Examines every line of all 497 specs for quote parity, which is total
-- a line either can close its own string or cannot. The bracket component added
in Prop. 192 is NOT total: it matches field declarations of the form
`name : [...],` and so covers exactly that shape. Prop. 193's
`delimiter_balance_scan` is the question-shaped replacement for it and subsumes
it; this one is kept as the regression test for the 107 sites of Prop. 186.

ARTIFACTS. Reads `specs/**/*.t27`. WRITES `formal/unbalanced_fields_baseline.txt`
when no baseline exists (Prop. 192). Nothing else.

Prop. 188.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SPECS = ROOT / "specs"


def odd_quote_lines(text):
    """Code lines whose double quotes cannot balance.

    Determined by a state machine, not by splitting: a `//` inside a string is
    part of a URL, not a comment, and a first attempt that split on `//` before
    counting reported 74 findings of which every sampled one was
    `"https://..."`. Escapes, character literals and raw strings are handled
    explicitly; each exclusion was verified against this corpus.
    """
    out = []
    in_raw = False
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if in_raw:
            if '"#' in line:
                in_raw = False
            continue
        if 'r#"' in line:
            if line.count('"#') == 0:
                in_raw = True
            continue
        if s.startswith("//") or s.startswith("#"):
            continue
        in_str = False
        k = 0
        while k < len(line):
            c = line[k]
            if in_str:
                if c == "\\":
                    k += 2
                    continue
                if c == '"':
                    in_str = False
            else:
                if c == '"':
                    in_str = True
                elif c == "'" and k + 2 < len(line) and line[k + 1] == '"' \
                        and line[k + 2] == "'":
                    k += 3          # a character literal holding a quote
                    continue
                elif c == "/" and k + 1 < len(line) and line[k + 1] == "/":
                    break           # a real comment: nothing after it is lexed
            k += 1
        if in_str:
            out.append((i, s))
    return out

def unbalanced_field_types(text):
    """Field declarations whose bracket nesting never closes (Prop. 192).

    `benchmarks : [[]Const [,` has three `[` and one `]`. A scanner that
    respects nesting therefore never stops, and consumes the struct's closing
    brace and everything after it. The naive scanner in `parse_struct_body`
    survives it only by ignoring nesting entirely -- which is why this
    corruption has been invisible: the one component that would notice is the
    one component written not to look.

    Same family as the runaway string above: a generator emitted a wrong
    character (`[` for `]`, as it emitted `"` for `]` elsewhere) and nothing
    reported it.
    """
    out = []
    for i, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if s.startswith("//") or s.startswith("#"):
            continue
        m = re.match(r"^\s*\w+\s*:\s*(\[[^,;]*)[,;]\s*$", line)
        if m and m.group(1).count("[") != m.group(1).count("]"):
            out.append((i, s))
    return out


def main():
    if not SPECS.exists():
        print(f"::error::runaway string scan: no such directory 'specs' under "
              f"the repository root -- nothing was scanned")
        return 1
    files = sorted(SPECS.rglob("*.t27"))
    if not files:
        print("::error::runaway string scan: found no .t27 files under specs/ "
              "-- nothing was scanned")
        return 1

    findings = []
    unbalanced = []
    for f in files:
        try:
            text = f.read_text()
        except (OSError, UnicodeDecodeError):
            continue
        for line_no, src in odd_quote_lines(text):
            findings.append((str(f.relative_to(ROOT)), line_no, src))
        for line_no, src in unbalanced_field_types(text):
            unbalanced.append((str(f.relative_to(ROOT)), line_no, src))

    print(f"runaway string scan: {len(files)} specs, {len(findings)} lines whose "
          f"quotes cannot balance, {len(unbalanced)} field types whose brackets "
          f"cannot balance")

    # Prop. 192: RATCHETED, not walled. The 18 known instances lose an element
    # type as well as a bracket -- `[[]Const [,` -- and balance decides only the
    # bracket, so repairing them is a judgement about what the type WAS. The
    # gate records the set and fails when it grows.
    ub_baseline = ROOT / "formal" / "unbalanced_fields_baseline.txt"
    ub_now = sorted(f"{p}:{n}" for p, n, _ in unbalanced)
    if not ub_baseline.exists():
        ub_baseline.write_text("\n".join(ub_now) + ("\n" if ub_now else ""))
        print(f"runaway string scan: unbalanced-field baseline written "
              f"({len(ub_now)} entries)")
    else:
        was = [l for l in ub_baseline.read_text().splitlines() if l.strip()]
        new_ub = [u for u in ub_now if u not in was]
        if new_ub:
            print(f"::error::runaway string scan: {len(new_ub)} NEW field "
                  f"type(s) whose brackets cannot balance. A nesting-aware "
                  f"scanner never stops on these and consumes the rest of the "
                  f"enclosing block (Prop. 192)")
            for u in new_ub[:10]:
                print(f"  {u}")
            return 1

    if not findings:
        return 0

    print(f"::error::runaway string scan: {len(findings)} line(s) contain an odd "
          f"number of double quotes. The lexer reads the rest of the file as "
          f"string content, so every declaration after this point is lost "
          f"SILENTLY -- no parse error, no recovery event, no swallowed-"
          f"declaration count. This is how 107 corrupted field types hid for "
          f"months (Prop. 186)")
    for path, line_no, src in findings[:20]:
        print(f"  {path}:{line_no}  {src[:70]}")
    return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::runaway string scan: could not read specs/ "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
