#!/usr/bin/env python3
"""The one reader of `docs/BEE_TOOLBELT.md`.

Three things need the toolbelt and each would otherwise parse it again:
`tools/queen/feed_empty_bodies.py` and `tools/queen/feed_defects.py` embed the
`## Start here` block in every issue they open, and `scripts/tri_loop/toolbelt.py`
runs every command in the document against a real spec. Three parsers of one
file is the shape this repository keeps finding in its own specs - 576 of 4021
function bodies are byte-identical copies - and it is not a good look in the
tool that tells bees not to do it.

The document is the source of truth, not this module. What lives here is only
how to read it: `## Heading` starts a section, a list item whose first backtick
span is a command line is a command.

Used as:

    sys.path.insert(0, os.path.join(REPO_ROOT, "tools"))
    from toolbelt import brief, commands, section
"""
from __future__ import annotations

import os

DOC_RELPATH = os.path.join("docs", "BEE_TOOLBELT.md")
START_HERE = "## Start here"

# A command line is the first backtick span of a list item. The rest of the item
# is prose ABOUT the command and must not be run: "- `t27c lint <spec>` - the
# warnings a reviewer will quote at you" is one command and one sentence.
import re  # noqa: E402  (after the constants, so the why reads first)

_ITEM = re.compile(r"^\s*[-*]\s+`([^`]+)`")


def document(root: str) -> str:
    """The path to the toolbelt, given a checkout root."""
    return os.path.join(root, DOC_RELPATH)


def read(root: str) -> str:
    """The document's text, or "" when it is not there.

    Empty rather than an exception on purpose: a feeder that cannot read the
    toolbelt should open its issues without one, not stop opening issues. The
    caller that needs it to exist says so itself.
    """
    try:
        with open(document(root), encoding="utf-8") as handle:
            return handle.read()
    except OSError:
        return ""


def section(text: str, heading: str) -> str:
    """The lines under one `## heading`, up to the next `## `. Empty when absent."""
    out, inside = [], False
    for line in text.split("\n"):
        if line.startswith("## "):
            inside = line.strip() == heading
            continue
        if inside:
            out.append(line)
    return "\n".join(out).strip("\n")


def commands(text: str) -> list[str]:
    """Every command line in the given text, in the order it is written."""
    found = []
    for line in text.split("\n"):
        match = _ITEM.match(line)
        if match:
            found.append(match.group(1).strip())
    return found


def brief(root: str) -> str:
    """The `## Start here` block, verbatim, or "" when there is no document."""
    return section(read(root), START_HERE)
