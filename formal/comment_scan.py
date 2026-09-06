#!/usr/bin/env python3
"""Fail when a gate matches a regex against source it never stripped comments from.

Wave 652. Five separate fixes across four files have now been spent on one
defect: a pattern applied to raw source text, matching inside a `//` comment.

    Prop.  95   claims_check counted `a_x: assert` inside a comment, inventing
                a twenty-ninth integration property out of prose.
    Prop. 102c  orphan_scan had the identical defect, on the identical regex,
                in a sibling file -- and survived a wave because nobody grepped
                after fixing the first.
    Prop. 118   identity_scan spliced a comment INTO an assertion body, so a
                remark explaining why a property is not a self-comparison made
                it read as one; guard_scan reported a guard as present from a
                comment saying it had been removed; bound_scan's note parser
                rejected the repository's own backtick style.

Each was found separately, wave after wave. This closes the class instead of
meeting it again: any gate that reads Verilog and applies a regex must pass the
text through a comment stripper first, or declare why it does not.

DECLARING is legitimate and common here. Some gates read comments ON PURPOSE --
`bound_scan` parses `// BOUND:` notes, `width_scan` parses `range [-N, +M]`
annotations, `doc_gate` reads markdown where `//` means nothing. Those say so
with a marker, and the marker is the point: it forces the question to be
answered once, in writing, where a reader can check it.

Marker:  # comment-scan: <reason>

Usage:  python3 formal/comment_scan.py [--self-test]
"""

import ast
import pathlib
import re
import sys

# A gate is in scope if it reads Verilog. Markdown and YAML have no `//`.
READS_VERILOG = re.compile(r"\.sv\b|build/rtl|glob\(['\"]\*\.sv")
# Recognised strippers, BY NAME. mutate.py's is called `code_mask` -- it masks
# comments, nested `ifdef T27_FORMAL* regions and labelled assertion lines --
# and the first version of this list did not know that name, so a gate that
# does the right thing was reported as one that does not. Over-detection, in
# the gate written to close an over-detection class.
STRIPPERS = re.compile(r"nocomment|strip_comments|code_mask|sub\(r?['\"]//|"
                       r"re\.sub\([^)]*//[^)]*\)")
MARKER = re.compile(r"#\s*comment-scan:\s*(.+)")


def check_file(path):
    """(finding or None, in_scope) for one gate."""
    src = path.read_text()
    try:
        tree = ast.parse(src)
    except SyntaxError:
        return (f"{path.name}: does not parse, so it cannot be checked -- "
                "which is not the same as being safe"), False

    if not READS_VERILOG.search(src):
        return None, False

    # Does it apply a regex at all?
    uses_regex = bool(re.search(r"re\.(findall|search|finditer|match|sub)|"
                                r"\.finditer\(|\.findall\(|\.search\(", src))
    if not uses_regex:
        return None, False

    declared = MARKER.search(src)
    if declared:
        return None, True
    if STRIPPERS.search(src):
        return None, True
    return (f"{path.name}: reads Verilog and applies a regex, but never strips "
            "`//` comments and does not declare why. Five fixes across four "
            "files have gone to this one shape (Props. 95, 102c, 118). Add a "
            "stripper, or `# comment-scan: <reason>` stating what it reads "
            "comments for."), True


def check(root):
    gates = sorted((root / "formal").glob("*.py"))
    if not gates:
        print(f"::error::comment_scan found no gates under {root}/formal")
        return 1

    # Prop. 208: this gate PASSED when starved. Copied alone into an empty tree
    # it found one file -- itself -- and that file matches READS_VERILOG,
    # because the pattern `\.sv\b` occurs in this very source as a regex
    # literal. **A scanner that satisfies its own liveness floor by matching its
    # own source cannot detect an empty corpus.** The floor below (`scoped == 0`)
    # was written precisely to stop a silent clean sweep, and self-matching
    # walked around it.
    #
    # Two independent fixes, because either alone leaves a hole:
    #   1. a scanner is not a subject -- exclude this file from the population;
    #   2. require the scripts the workflows actually run to be present, the
    #      same repair coverage_gate needed for the same reason (Prop. 200).
    gates = [g for g in gates if g.name != pathlib.Path(__file__).name]
    wf_dir = root / ".github" / "workflows"
    cited = set()
    if wf_dir.exists():
        for y in wf_dir.glob("*.yml"):
            cited.update(re.findall(r"python3 formal/(\w+\.py)",
                                    y.read_text(errors="ignore")))
    if not cited:
        print(f"::error::comment_scan found no `python3 formal/*.py` step in "
              f"any workflow -- there is nothing to check the scanned corpus "
              f"against, so this gate can establish nothing")
        return 1
    absent = sorted(cited - {g.name for g in gates} - {pathlib.Path(__file__).name})
    if absent:
        print(f"::error::comment_scan: {len(absent)} script(s) the workflows "
              f"run are absent from formal/ -- the corpus scanned is not the "
              f"corpus CI executes (Prop. 208)")
        for a in absent[:8]:
            print(f"  formal/{a}")
        return 1
    bad, scoped = [], 0
    for g in gates:
        finding, in_scope = check_file(g)
        scoped += in_scope
        if finding:
            bad.append(finding)
    for b in bad:
        print(f"::error::{b}")
    # A scan that found nothing in scope reports clean. Prop. 82c's lesson.
    if scoped == 0:
        print(f"::error::comment_scan found no Verilog-reading gates among "
              f"{len(gates)} files -- it checked nothing, so its silence means "
              "nothing.")
        return 1
    print(f"comment scan: {len(gates)} gates, {scoped} read Verilog with a "
          f"regex, {len(bad)} neither strip comments nor say why")
    return 1 if bad else 0


def self_test():
    import tempfile
    bad = []
    CASES = [
        ("a gate that strips comments", '''"""Reads build/rtl/x.sv."""
import re
def go(t):
    return re.findall(r"a_x", re.sub(r"//[^\\n]*", "", t))
''', None),
        ("a gate that declares why it reads them", '''"""Reads build/rtl/x.sv."""
# comment-scan: parses `// BOUND:` notes, so comments ARE the subject.
import re
def go(t):
    return re.findall(r"// BOUND:", t)
''', None),
        ("a gate that does neither -- the Prop. 95 shape", '''"""Reads build/rtl/x.sv."""
import re
def go(t):
    return re.findall(r"a_[a-z]+: assert", t)
''', "never strips"),
        ("a gate that reads no Verilog at all", '''"""Reads README.md."""
import re
def go(t):
    return re.findall(r"x", t)
''', None),
    ]
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td)
        for name, text, want in CASES:
            f = d / "zz_gate.py"
            f.write_text(text)
            finding, _ = check_file(f)
            ok = (want is None and finding is None) or \
                 (want is not None and finding is not None and want in finding)
            print(f"  {'ok  ' if ok else 'FAIL'} {name}")
            if not ok:
                bad.append(name)

        (d / "formal").mkdir()
        (d / "formal" / "zz.py").write_text('"""Nothing."""\nx = 1\n')
        rc = check(d)
        print(f"  {'ok  ' if rc else 'FAIL'} a tree with nothing in scope fails "
              f"rather than passing silently: exit {rc}")
        if rc == 0:
            bad.append("the zero-scope guard let an empty tree pass")

    for b in bad:
        print(f"::error::comment_scan self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else check(r))
