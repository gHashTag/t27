#!/usr/bin/env python3
"""Batch append W391 test/invariant blocks to all 27 IGLA specs."""

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


def append_w391(path: Path) -> None:
    text = path.read_text()
    segments = wave_segments(text)
    if not segments:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last_wave, last_start, last_end = segments[-1]
    if last_wave != 390:
        print(f"SKIP {path}: last wave is {last_wave}, expected 390")
        return

    block = text[last_start:last_end]
    new_block = block
    new_block = new_block.replace("Wave Loop 390", "Wave Loop 391")
    new_block = new_block.replace("w390_", "w391_")
    new_block = new_block.replace("after W389", "after W390")
    new_block = new_block.replace("W389 seal regeneration", "W390 seal regeneration")
    new_block = new_block.replace("WAVE_LOOP_389_COOPERATION.md", "WAVE_LOOP_390_COOPERATION.md")

    new_text = text[:last_end] + new_block
    path.write_text(new_text)
    print(f"APPENDED W391 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w391(spec)
