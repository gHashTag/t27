# NOW -- One page instead of a pass (2026-09-04)

## The consolidation that cost a pass is now a command

- `tri whats-open` prints every gate instrument's reading on one page. Assembling it by hand cost
  the whole of pass 66, because `gates dead` takes over four minutes and `unmeasured` about fifty
  seconds while the other three finish in two -- so nobody ran them together.
- **The consequence was worse than the inconvenience.** Two consecutive passes hunted a defect
  class and came back empty: once because every gate was already honest, once because the question
  had been asked, tooled and **withdrawn** five days earlier. The instruments were not missing; the
  habit of reading them first was.
- It **quotes**, it does not measure: every figure is another command's output. A wrong number is
  wrong in the tool named beside it, and that is where the fix belongs.

## Four things it refuses to do

- **Silently drop its slow half.** `dead` and `unmeasured` are skipped without `--all`, and the
  skip is printed. A report that quietly omits what it could not afford is the shape this
  repository keeps finding.
- **Print a bare integer.** Two headlines capture only a count, and a column of naked numbers is
  how a figure loses the noun that made it mean something.
- **Hardcode a qualifier.** "5 of which print a page as a total" is read from the same output as
  the 25 -- writing it by hand would have been wrong the moment #3158 lands. A count hardcoded in a
  status tool is precisely the defect this loop keeps finding elsewhere.
- **Report a clean status it could not read.** If no instrument runs it exits **2**, not 0. Pass 64
  spent itself learning that failing closed, not owning a self-check, is what protects a tool from
  being believed.

## And it prints what is already settled

Three of four recent hunts re-opened something already closed, at a pass each. So the page also
carries the measured-and-clean list with its provenance: no gate reads a slice of its own subject;
no quiet gate guards a subject that is missing today; and PR-only-by-construction is not a gap --
a claim otherwise was withdrawn on 2026-08-30.

Refs #3157
