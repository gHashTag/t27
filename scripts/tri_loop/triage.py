#!/usr/bin/env python3
"""tri triage -- classify every open issue into one of five ordered classes.

Why this exists.

The plan in #1697 measured "~47 of 102 open issues are autonomous wave-loop
journal noise". The ratio has moved since: of 237 open issues, roughly 80%
record work that was already done rather than work that is waiting.

A record with no completion condition is not a task. Counting it as backlog is
the same composition error #2133 identified in the ratchet, applied to the
tracker instead of the corpus. This tool reports the split so that any statement
about "N open issues" can be read correctly.

What changed, and why three classes were not enough.

The earlier version had three classes -- journal, plan, actionable -- and
everything that did not look like a journal entry fell into `actionable` by
default. That default is the flaw. An issue waiting on hardware, an issue that
asks a research question with no defined answer, and an issue that duplicates
another are all things a loop cannot pick up and finish, yet all three were
counted as available work. The count of "~26 actionable" was therefore an upper
bound being read as an estimate.

Five ordered classes, first match wins:

  actionable  a defect or change with a checkable completion condition
  research    an open question; done is a matter of judgement, not a check
  tracking    a journal entry, plan or epic; records or aggregates, never closes
  blocked     cannot proceed without something outside the tool: hardware, an
              upstream release, a human decision, an expired credential
  duplicate   superseded, obsolete, or the same subject as an earlier issue

Ordering matters and is deliberate: `blocked` is tested before `actionable`, so
a blocked defect is never advertised as available. `duplicate` is tested last,
because a duplicate that is also blocked is more usefully seen as blocked.

Autoclose is forbidden. This tool prints and exits; it never mutates the
tracker. The classification reads titles and labels, and reads bodies only for
the blocked and duplicate signals, where the evidence is not in the title. It
remains a composition estimate, never a verdict on an individual issue.

Usage:
    tri triage [owner/repo] [--json] [--class NAME] [--bodies]
"""
import json
import re
import subprocess
import sys
from collections import Counter

CLASSES = ("actionable", "research", "tracking", "blocked", "duplicate")

TRACKING = re.compile(
    r"^(wave\s|wave\s*loop|feat\(igla\):\s*wave|formal:.*\(prop\.|tick\s"
    r"|\[plan\]|epic\b|census\b|report:)", re.I
)
RESEARCH = re.compile(
    r"(\?$|^(research|investigate|explore|study|survey|consider|should we|why\s)"
    r"|\bopen question\b|\bhypothes)", re.I
)
BLOCKED = re.compile(
    r"\b(blocked\s+(on|by)|waiting\s+(on|for)|needs\s+hardware|requires\s+the\s+board"
    r"|routing[- ]pending|deferred\b|upstream\s+bug|awaiting\s+review"
    r"|needs\s+a\s+human|cannot\s+proceed\s+until)\b", re.I
)
DUPLICATE = re.compile(
    r"\b(duplicate\s+of|superseded\s+by|obsolete\b|closed\s+in\s+favour|see\s+instead)\b",
    re.I
)
LABEL_MAP = {
    "blocked": "blocked", "wontfix": "duplicate", "duplicate": "duplicate",
    "question": "research", "research": "research", "journal": "tracking",
    "epic": "tracking", "plan": "tracking",
}


def klass(row, use_bodies=True):
    """Return (class, reason). First match in CLASSES order wins."""
    title = row.get("title") or ""
    body = (row.get("body") or "") if use_bodies else ""
    labels = {(l.get("name") or "").lower() for l in row.get("labels") or []}

    for name, cls in LABEL_MAP.items():
        if name in labels:
            return cls, f"label:{name}"

    m = BLOCKED.search(title) or (BLOCKED.search(body[:2000]) if body else None)
    if m:
        return "blocked", f"phrase:{m.group(0).strip().lower()[:40]}"

    m = TRACKING.search(title)
    if m:
        return "tracking", f"title-form:{m.group(0).strip().lower()[:40]}"

    m = RESEARCH.search(title)
    if m:
        return "research", f"title-form:{m.group(0).strip().lower()[:40]}"

    m = DUPLICATE.search(title) or (DUPLICATE.search(body[:2000]) if body else None)
    if m:
        return "duplicate", f"phrase:{m.group(0).strip().lower()[:40]}"

    return "actionable", "default: no tracking, research, blocked or duplicate signal"


def main(argv):
    repo = "gHashTag/t27"
    as_json = "--json" in argv
    use_bodies = "--bodies" in argv or True
    want = None
    if "--class" in argv:
        i = argv.index("--class")
        if i + 1 < len(argv):
            want = argv[i + 1].lower()
            if want not in CLASSES:
                print(f"unknown class {want!r}; expected one of {', '.join(CLASSES)}",
                      file=sys.stderr)
                return 2
    positional = [a for a in argv if not a.startswith("--")]
    positional = [a for a in positional if a != want]
    if positional:
        repo = positional[0]

    fields = "number,title,labels,createdAt,comments"
    if use_bodies:
        fields += ",body"
    # A full page is a LOWER BOUND and only a short one is a total. `gh` returns
    # at most --limit rows and says nothing about what it left behind, so a read
    # that FILLS its limit is a floor, not a census. Measured 2026-09-04: 506
    # open issues here against the 1000 asked for, and the backlog grew 140 ->
    # 506 between 2026-08-01 and today (7-11 a day), which reaches 1000 in
    # roughly 45-68 days. Latent, dated, and one line to say out loud.
    LIMIT = "1000"
    out = subprocess.run(
        ["gh", "issue", "list", "--repo", repo, "--state", "open", "--limit", LIMIT,
         "--json", fields],
        capture_output=True, text=True)
    if out.returncode != 0:
        print(out.stderr.strip(), file=sys.stderr)
        return 2
    rows = json.loads(out.stdout)
    if len(rows) >= int(LIMIT):
        print(f"  issues read from gh  {len(rows)}   *** EQUALS the --limit of "
              f"{LIMIT}: a LOWER BOUND, not a total. Raise --limit and read "
              f"again. ***", file=sys.stderr)

    for r in rows:
        r["class"], r["reason"] = klass(r, use_bodies)
        r.pop("body", None)
    counts = Counter(r["class"] for r in rows)

    if as_json:
        payload = {"repo": repo, "open": len(rows),
                   "counts": {c: counts.get(c, 0) for c in CLASSES},
                   "issues": sorted(rows, key=lambda r: r["number"])}
        if want:
            payload["issues"] = [r for r in payload["issues"] if r["class"] == want]
        print(json.dumps(payload, indent=2))
        return 0

    total = len(rows) or 1
    print(f"{repo}: {len(rows)} open")
    for c in CLASSES:
        v = counts.get(c, 0)
        print(f"  {v:5d}  {c:<11s} ({100 * v / total:.0f}%)")

    for c in ([want] if want else ["actionable", "blocked"]):
        sel = sorted((r for r in rows if r["class"] == c), key=lambda r: r["number"])
        print(f"\n{c} ({len(sel)}):")
        for r in sel:
            print(f'  #{r["number"]:<6d} {r["createdAt"][:10]}  {r["title"][:88]}')
            print(f'          {r["reason"]}')

    print("\nNOTE: a composition estimate from titles, labels and body prefixes -- not a")
    print("      verdict on any single issue. This tool never closes anything, and")
    print("      nothing here licenses a bulk action. `blocked` is tested before")
    print("      `actionable` on purpose, so no blocked item is advertised as available.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
