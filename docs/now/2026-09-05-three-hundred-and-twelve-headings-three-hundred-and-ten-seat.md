# NOW -- three hundred and twelve headings, three hundred and ten seats (2026-09-05)

## three hundred and twelve headings, three hundred and ten seats (Refs #3195)

- bodies() keys by title, which is right for the history walk and wrong for every other question: a repeated heading text is one key and the later insert overwrites, so only the last copy body is ever examined
- measured on docs/NOW.md: 312 heading lines, 310 distinct texts. The command printed present-on-master 310 for a 312-heading file and asked the hollow question over 310 seats, while four comments in that same source said 312. The 312-vs-310 measurement was written into the prose two passes before the tool was built on the map that collapses them
- the verdict was right by luck: all four colliding occurrences have bodies, so hollow is 0 either way. It flips the first time a repeated subject earlier copy is bare
- fixed by separating the questions rather than picking one: hollow and misattribution run over occurrences which collapses nothing, the history walk keeps the title map, and both counts print with their unit
