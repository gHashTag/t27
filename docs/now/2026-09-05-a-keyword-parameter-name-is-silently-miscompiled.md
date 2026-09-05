# NOW -- A parameter named with a keyword is silently miscompiled (2026-09-05)

## A parameter named with a keyword is silently miscompiled (Refs #3299)

- Found by censusing the FIRST rustc error of all 246 failing specs rather than reading
  them one at a time. The largest group, `expected type, found \`,\``, held 20 specs.
- My first reading of that group was wrong: I called it one class of 20. It is at least
  four. Thirteen are struct fields whose type is empty **in the spec source** -- the
  compiler prints exactly what is written, so those are not its defect. Two are this one.
- The parameter parser skips any token that is not `Ident`, as error recovery. `module`,
  `use` and `fn` lex as keywords, so the name is dropped, the colon is dropped on the
  next turn, the TYPE becomes the next parameter's name, and that parameter is left with
  an empty type.
- It is not an emitter defect. C and Rust are broken identically, which is what said the
  cause was upstream of both: `bool f(const char* a,  str, bool b)`.
- At exit 0. A green exit that is not a result.
- Repaired where the recovery path is: a keyword FOLLOWED BY A COLON is an identifier.
  Not followed by one, the old path runs unchanged, one token consumed.
- Measured on the emission, not only on acceptance: of the 12 specs using a keyword in a
  name position, 2 emitted an empty type before and 0 after. The Rust column moved
  335 to 336, because the second spec now fails on something else -- and the error text
  changed from `expected type, found \`,\`` to `cannot find module or crate parser`,
  which is how that was confirmed rather than assumed.
- Corpus-side bound, measured while here: at most 14 of the 246 failures carry an empty
  type in their source. 232 are downstream of the source, so the compiler-side class is
  nowhere near exhausted.
