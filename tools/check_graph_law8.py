#!/usr/bin/env python3
"""LAW 8 -- every edge flows forward, and no cycle -- measured instead of asserted.

docs/TECHNOLOGY-TREE.md states LAW 8 and, since #3092, admits that nothing
verifies it. Something claimed to: `.claude/skills/tri/scripts/graph-depcheck.sh`
opens with "Validate t27 canonical dependency graph" and is advertised in
skill.md as "Validate graph dependencies". It reads nothing. Its tier check is

    local violations=0
    ...
    if [[ $violations -eq 0 ]]; then echo "  OK No forward tier dependencies detected"

-- a green tick computed from a literal, and `GRAPH_FILE` is assigned and never
read. Run it from an empty directory and the output is byte-identical.

What the graph actually says, at the commit this was written against:

    nodes 55   edges 91
    forward 65   same-tier 21   tier-backward 5
    cycles 1     17 -> 19 -> 18 -> 17

So LAW 8 is violated today. The numbers are held by a down-only ledger rather
than failed on: repairing the graph is an architectural decision, and a checker
landing red on arrival is a checker that gets muted. A NEW violation fails; a
repaired one fails until the ledger is lowered in the same commit.

Exit 0 within the ledger, 1 when it moves, 2 when the graph cannot be read --
absence is not health, and that is the whole reason this file exists.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GRAPH = ROOT / "architecture" / "graph_v2.json"

# LAW 8 is stated over DEPENDENCIES, and this graph carries twelve edge kinds
# of which three are documentation relations:
#
#   documented-by  a spec -> the doc that documents it   (2 edges)
#   references     a doc  -> another doc                 (1)
#   standardizes   a doc  -> the specs it standardises   (3)
#
# `documented-by` is the INVERSE of a dependency, so counting it makes a spec
# "depend on" its own documentation. Measured by dropping one kind at a time:
#
#   all 91 edges                cycles 1   backward 5
#   drop documented-by (2)      cycles 0   backward 3
#   drop references    (1)      cycles 0   backward 5
#
# The single cycle 17 -> 19 -> 18 -> 17 is made of one `documented-by`, one
# `references` and one `import`; remove either documentation edge and it is
# gone. Two of the five backward edges are `documented-by`, backward by
# construction.
#
# So both readings are reported and both are held. The dependency reading is
# the one LAW 8 is about; the all-edges reading is kept beside it so nobody has
# to trust this file's choice of which kinds are documentation.
DOC_KINDS = ("documented-by", "references", "standardizes")

# Measured at 3cee86539. Down only: see the module docstring.
MAX_CYCLES = 1
MAX_TIER_BACKWARD = 5
MAX_DEP_CYCLES = 0
MAX_DEP_TIER_BACKWARD = 3


def refuse(msg: str) -> None:
    print(f"check_graph_law8: {msg}", file=sys.stderr)
    print("  Exit 2 = could not run. An absent graph is not a graph without cycles.",
          file=sys.stderr)
    sys.exit(2)


def load() -> tuple[list, list]:
    if not GRAPH.is_file():
        refuse(f"{GRAPH.relative_to(ROOT)} is missing.")
    try:
        doc = json.loads(GRAPH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        refuse(f"{GRAPH.relative_to(ROOT)} does not parse: {exc}")
    nodes = doc.get("nodes")
    edges = doc.get("edges")
    if not isinstance(nodes, list) or not isinstance(edges, list):
        refuse("the graph has no `nodes`/`edges` arrays; the shape changed.")
    if not nodes or not edges:
        refuse(f"the graph is empty ({len(nodes)} nodes, {len(edges)} edges).")
    return nodes, edges


def cycles_of(edges: list) -> list[list]:
    """Every cycle reachable by DFS, reported as the node ring that closes it."""
    out = defaultdict(list)
    for e in edges:
        out[e.get("from")].append(e.get("to"))
    WHITE, GREY, BLACK = 0, 1, 2
    colour: dict = defaultdict(int)
    found: list[list] = []

    def walk(u, stack):
        colour[u] = GREY
        stack.append(u)
        for v in out.get(u, []):
            if colour[v] == GREY:
                found.append(stack[stack.index(v):] + [v])
            elif colour[v] == WHITE:
                walk(v, stack)
        stack.pop()
        colour[u] = BLACK

    for n in list(out):
        if colour[n] == WHITE:
            walk(n, [])
    return found


def main() -> int:
    if "--self-check" in sys.argv:
        # A checker that cannot fail is a green light with no bulb behind it.
        ring = cycles_of([{"from": 1, "to": 2}, {"from": 2, "to": 1}])
        acyclic = cycles_of([{"from": 1, "to": 2}, {"from": 2, "to": 3}])
        print(f"  self-check  finds a two-node cycle:   {'ok' if len(ring) == 1 else 'BROKEN'}")
        print(f"  self-check  calls a chain acyclic:    {'ok' if not acyclic else 'BROKEN'}")
        return 0 if (len(ring) == 1 and not acyclic) else 2

    nodes, edges = load()
    tier = {n.get("id"): n.get("tier") for n in nodes}

    backward, same, forward, unrankable = [], 0, 0, 0
    for e in edges:
        a, b = tier.get(e.get("from")), tier.get(e.get("to"))
        if not isinstance(a, int) or not isinstance(b, int):
            unrankable += 1
        elif a > b:
            backward.append(e)
        elif a == b:
            same += 1
        else:
            forward += 1

    rings = cycles_of(edges)

    dep = [e for e in edges if e.get("kind") not in DOC_KINDS]
    dep_rings = cycles_of(dep)
    dep_backward = []
    for e in dep:
        a, b = tier.get(e.get("from")), tier.get(e.get("to"))
        if isinstance(a, int) and isinstance(b, int) and a > b:
            dep_backward.append(e)

    print(f"graph: {len(nodes)} nodes, {len(edges)} edges  ({GRAPH.relative_to(ROOT)})")
    print(f"  forward {forward}   same-tier {same}   tier-backward {len(backward)}"
          f"   unrankable {unrankable}")
    print(f"  cycles {len(rings)}")
    for r in rings:
        print("    " + " -> ".join(str(x) for x in r))
    for e in backward:
        print(f"    backward: {e.get('from')} (tier {tier.get(e.get('from'))})"
              f" -> {e.get('to')} (tier {tier.get(e.get('to'))})")

    print(f"\ndependency reading -- {len(dep)} edges, excluding "
          f"{', '.join(DOC_KINDS)}:")
    print(f"  cycles {len(dep_rings)}   tier-backward {len(dep_backward)}")
    for r in dep_rings:
        print("    " + " -> ".join(str(x) for x in r))
    for e in dep_backward:
        print(f"    backward: {e.get('from')} (tier {tier.get(e.get('from'))})"
              f" -> {e.get('to')} (tier {tier.get(e.get('to'))})  kind={e.get('kind')}")

    bad = False
    if len(dep_rings) != MAX_DEP_CYCLES:
        verb = "rose" if len(dep_rings) > MAX_DEP_CYCLES else "fell"
        print(f"\nFAIL: dependency cycles {verb} {MAX_DEP_CYCLES} -> {len(dep_rings)}.")
        bad = True
    if len(dep_backward) != MAX_DEP_TIER_BACKWARD:
        verb = "rose" if len(dep_backward) > MAX_DEP_TIER_BACKWARD else "fell"
        print(f"FAIL: dependency tier-backward {verb} {MAX_DEP_TIER_BACKWARD}"
              f" -> {len(dep_backward)}.")
        bad = True
    if len(rings) != MAX_CYCLES:
        verb = "rose" if len(rings) > MAX_CYCLES else "fell"
        print(f"\nFAIL: cycles {verb} {MAX_CYCLES} -> {len(rings)}.")
        bad = True
    if len(backward) != MAX_TIER_BACKWARD:
        verb = "rose" if len(backward) > MAX_TIER_BACKWARD else "fell"
        print(f"FAIL: tier-backward edges {verb} {MAX_TIER_BACKWARD} -> {len(backward)}.")
        bad = True
    if bad:
        print("  Up: LAW 8 is violated in a new place. Down: good -- lower the\n"
              "  ledger in the same commit, so the next one cannot hide under slack.")
        return 1

    print(f"\nok: over DEPENDENCIES, LAW 8 has {MAX_DEP_CYCLES} cycle(s) and"
          f" {MAX_DEP_TIER_BACKWARD} backward edge(s) -- the recorded ledger."
          f"\n    Over all {len(edges)} edges it reads {MAX_CYCLES} and"
          f" {MAX_TIER_BACKWARD}, and the difference is documentation.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
