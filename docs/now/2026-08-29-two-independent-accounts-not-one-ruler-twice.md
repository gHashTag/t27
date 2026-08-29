# NOW -- Two independent accounts, not one ruler twice (2026-08-29)

## Two independent accounts, not one ruler twice (Refs #2807)

- A scanner looking for the `adamw.t27` defect elsewhere reported 91 files with
  repeated top-level definitions, worst being `specs/numeric/gf16.t27` at 561
  extras. Tightening the pattern to module indentation gave 43 files and 539 --
  close enough to feel confirmed.
- Both were the same ruler. `gf16.t27` has 110 Zig test blocks, each opening
  `const a = gf16_encode_f32(...)` at four spaces. The scanner counted local
  variables. The true count for that file is zero, and the corpus-wide number
  is withdrawn.
- What settled it was a signal that does not pass through the name scanner:
  duplicated section banners. `adamw.t27` carries `// 1. Constants` /
  `// 2. Types` / `// 3. Core Functions` twice; `gf16.t27` has none. Exactly
  one file in `specs/` fires both signals, and it is the one already read by
  hand.
- Recorded as ci-gates 208. Third scanner artifact this session -- all three
  produced a confident number, and none was caught by re-reading the scanner.
