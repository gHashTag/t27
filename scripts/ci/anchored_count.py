"""A count a document states about the tree: one generated block, checked at its anchor.

Two gates had the same flaw -- a figure checked as "today's number appears
somewhere in docs/theory/IGLA-FORMAL-RESULTS.md", a 27,000-line file. That
passes on any unrelated occurrence of the number and cannot see the stated
figure move:

  test_retaken_propositions_still_match.py   corpus-count    #5799 (#5801)
  test_catalog_table_matches_the_gate.py      catalog-count   #5881

Both now keep the figure in exactly one marked field,

    <!-- NAME anchor=<40-hex commit> -->N<!-- /NAME -->

and check the relation the field states: N is what the gate's own counter
returns on the tree of the anchor commit. Nothing outside the marker is read.
The block must also sit in a `RE-TAKEN AT` re-take naming the same commit, and
that re-take's measured row -- typed by hand from a command's output, where the
block is generated -- must say the same number.

Why anchored and not live is written in test_retaken_propositions_still_match.py:
a figure every pull request regenerates merges cleanly into a wrong master.

This module is the shared half. Each gate brings its marker name, its counter
(anchor -> (n, "") or (None, why)) and the row its re-take measures.
"""

import re
import subprocess

HEADING = re.compile(r"RE-TAKEN AT `([0-9a-f]{7,40})`")


def block_re(name):
    n = re.escape(name)
    return re.compile(
        rf"<!-- {n} anchor=(?P<anchor>[0-9a-f]+) -->"
        r"(?P<count>[^<]*)"
        rf"<!-- /{n} -->")


def render(name, anchor, n):
    return f"<!-- {name} anchor={anchor} -->{n}<!-- /{name} -->"


def git(*args, cwd=None):
    return subprocess.run(["git", *args], capture_output=True, text=True,
                          encoding="utf-8", errors="replace", cwd=cwd)


def count_at(anchor, counter, cwd=None):
    """`counter(anchor)` once `anchor` is here, or (None, why).

    CI checks out one commit (`fetch-depth: 1`), so the anchor is usually not
    there; fetch exactly it. A figure that could not be re-counted is reported as
    such -- unmeasured is not passing.
    """
    if git("cat-file", "-e", f"{anchor}^{{commit}}", cwd=cwd).returncode != 0:
        got = git("fetch", "--quiet", "--no-tags", "--depth=1", "origin", anchor, cwd=cwd)
        if got.returncode != 0:
            return None, f"cannot fetch {anchor} from origin: {got.stderr.strip()}"
    return counter(anchor)


def retake_end(doc, start):
    """End of the quoted re-take opened at `start`: the first line after it not starting `>`."""
    i = doc.find("\n", start)
    while i != -1:
        nxt = i + 1
        if not doc.startswith(">", nxt):
            return nxt
        i = doc.find("\n", nxt)
    return len(doc)


def write(path, shown, name, counter, new_anchor=None, cwd=None):
    """`--write`: regenerate every `name` block from its anchor's tree.

    With `new_anchor`, move the single block to that commit first.
    """
    with open(path, errors="ignore") as fh:
        doc = fh.read()
    blocks = list(block_re(name).finditer(doc))
    if not blocks:
        print(f"no {name} block in {shown}; nothing to write")
        return 1
    if new_anchor:
        if len(blocks) != 1:
            print(f"--anchor needs exactly one block, found {len(blocks)}")
            return 1
        rev = git("rev-parse", "--verify", f"{new_anchor}^{{commit}}", cwd=cwd)
        if rev.returncode != 0:
            print(f"--anchor {new_anchor}: not a commit here ({rev.stderr.strip()})")
            return 1
        new_anchor = rev.stdout.strip()
    out, last = [], 0
    for b in blocks:
        anchor = new_anchor or b["anchor"]
        n, why = count_at(anchor, counter, cwd)
        if n is None:
            print(f"cannot regenerate: {why}")
            return 1
        out.append(doc[last:b.start()])
        out.append(render(name, anchor, n))
        last = b.end()
        print(f"  {b['anchor'][:10]} {b['count']!r} -> {anchor[:10]} {n}")
    out.append(doc[last:])
    with open(path, "w") as fh:
        fh.write("".join(out))
    print(f"wrote {shown}")
    return 0


def verify(doc, name, counter, check, *, what, figure, noun, row, row_what,
           row_said, write_cmd, today=None, cwd=None):
    """Check every `name` block in `doc`, reporting through `check(name, ok, detail)`.

    Returns [(block match, (start, end) of its re-take or None)].
    """
    marks = list(block_re(name).finditer(doc))
    check(f"{what} is a marked {name} block", len(marks) > 0,
          f"no <!-- {name} anchor=... --> block -- a figure that is not marked "
          "is a substring anywhere in the file, which is what passed by coincidence")
    found = []
    for m in marks:
        anchor, stated = m["anchor"], m["count"]
        short = anchor[:10]
        found.append((m, None))
        check(f"block {short}: the anchor is a full 40-hex commit", len(anchor) == 40,
              f"`{anchor}` -- a short id cannot be fetched by CI and can turn ambiguous")
        if len(anchor) != 40:
            continue
        n, why = count_at(anchor, counter, cwd)
        check(f"block {short}: the tree at the anchor can be re-counted", n is not None, why)
        if n is None:
            continue
        since = "" if today is None else f"  (today: {today}, {today - n:+d} since)"
        print(f"      {noun} at {short}: {n}{since}")
        check(f"block {short}: states the {figure} of its anchor",
              stated.strip() == str(n),
              f"the block says {stated!r}, the tree at {anchor} has {n} {noun}. "
              f"Regenerate it -- do not type it: {write_cmd}")

        # The block belongs to the re-take it sits in: the nearest RE-TAKEN AT
        # above it names the same commit, and that re-take's own measured row
        # is the figure its prose says it equals.
        heads = list(HEADING.finditer(doc, 0, m.start()))
        check(f"block {short}: sits under a RE-TAKEN AT heading", bool(heads),
              f"a {name} block outside any re-take has no measurement to anchor")
        if not heads:
            continue
        head = heads[-1]
        check(f"block {short}: matches its heading `{head[1]}`",
              anchor.startswith(head[1]),
              f"the heading says `{head[1]}`, the block counts {anchor}: one of them "
              f"was moved without the other")
        span = (head.start(), max(m.end(), retake_end(doc, head.start())))
        found[-1] = (m, span)
        rows = row.findall(doc, *span)
        check(f"block {short}: the re-take's {row_what} agrees",
              bool(rows) and all(t == str(n) for t in rows),
              f"{row_said} {rows}, the anchor has {n}: "
              f"re-take the table at the anchor, it is a measurement" if rows else
              f"the re-take at `{head[1]}` has no {row_what} -- the block agrees "
              f"with nothing measured")
    return found
