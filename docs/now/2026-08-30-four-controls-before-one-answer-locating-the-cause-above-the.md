# NOW -- Four controls before one answer: locating the cause above the reported line (2026-08-30)

## Four controls before one answer: locating the cause above the reported line (Refs #2864)

- tri unparsed locate: feed growing prefixes of the module body back to the compiler, binary-search the first prefix that fails. 45 specs located and causally confirmed, 41 candidates refuted, 11 nothing claimed.
- Prefix bisection is UNSOUND alone: a truncated prefix can fail for a reason the whole file does not have. The refuted 41 are that.
- The fidelity control -- reconstruction must reproduce the original failure on the same line -- passed 46 of 46 while the answer was wrong for 16. Concatenating head+body+tail is the whole file no matter where the boundaries fall, so it cannot see a bad split.
- The causality control took three attempts. 'Error moved past the item' passed 45 of 45 by arithmetic. 'Error changed' credited commenting out a block-comment opener, which breaks the file further. What holds: the file parses, or the new error is LATER than the ORIGINAL -- compared against a fixed point, never against the item.
- Three bugs in the depth model, each found by reading an answer and none by a control: line comments, the module closer located by text instead of by depth, and a block comment wrapping a JSON schema whose brace moved every boundary in a 500-line file.
