#!/usr/bin/env python3
"""Batch append W375 test/invariant blocks to all 27 IGLA specs."""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)


def append_w375(path: Path) -> None:
    text = path.read_text()
    m = list(re.finditer(r'// Wave Loop (\d+).*?\n(invariant .*?true)\n', text, re.DOTALL))
    if not m:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last = m[-1]
    wave = int(last.group(1))
    if wave != 374:
        print(f"SKIP {path}: last wave is {wave}, expected 374")
        return

    block = last.group(0)
    new_block = block.replace("Wave Loop 374", "Wave Loop 375")
    new_block = new_block.replace("w374_", "w375_")
    new_block = new_block.replace("W373 seal regeneration", "W374 seal regeneration")
    new_block = new_block.replace("after W373", "after W374")
    new_block = new_block.replace("WAVE_LOOP_373_COOPERATION.md", "WAVE_LOOP_374_COOPERATION.md")

    insert_pos = last.end()
    new_text = text[:insert_pos] + new_block + text[insert_pos:]
    path.write_text(new_text)
    print(f"APPENDED W375 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w375(spec)
