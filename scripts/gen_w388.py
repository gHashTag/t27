#!/usr/bin/env python3
"""Batch append W388 test/invariant blocks to all 27 IGLA specs.

W388 builds on the W387 block. If a previous run left a duplicate W387 block
at the end of a spec (because this script originally failed to bump the wave
number), that duplicate is removed first and then a single proper W388 block
is appended.
"""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)

WAVE_HEADER_RE = re.compile(r"// Wave Loop (\d+)")


def wave_segments(text: str) -> list[tuple[int, int, int]]:
    """Return list of (wave_number, start, end) for each Wave Loop block."""
    matches = list(WAVE_HEADER_RE.finditer(text))
    segments = []
    for i, m in enumerate(matches):
        wave = int(m.group(1))
        start = m.start()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        segments.append((wave, start, end))
    return segments


def append_w388(path: Path) -> None:
    text = path.read_text()
    segments = wave_segments(text)
    if not segments:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    # Remove a duplicate trailing W387 block left by the buggy first run.
    if len(segments) >= 2 and segments[-1][0] == 387 and segments[-2][0] == 387:
        dup_start = segments[-1][1]
        text = text[:dup_start].rstrip() + "\n"
        segments = segments[:-1]

    last_wave, last_start, last_end = segments[-1]
    if last_wave != 387:
        print(f"SKIP {path}: last wave is {last_wave}, expected 387")
        return

    block = text[last_start:last_end]
    new_block = block
    new_block = new_block.replace("Wave Loop 387", "Wave Loop 388")
    new_block = new_block.replace("w387_", "w388_")
    new_block = new_block.replace("after W386", "after W387")
    new_block = new_block.replace("W386 seal regeneration", "W387 seal regeneration")
    new_block = new_block.replace("WAVE_LOOP_386_COOPERATION.md", "WAVE_LOOP_387_COOPERATION.md")

    new_text = text[:last_end] + new_block
    path.write_text(new_text)
    print(f"APPENDED W388 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w388(spec)
