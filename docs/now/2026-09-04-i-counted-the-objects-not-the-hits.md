# NOW -- I counted the objects, not the hits (2026-09-04)

## A count is a claim about an instrument's output

`docs/now/2026-09-04-grep-printed-the-match-and-said-ok.md` said `grep -rn
"Kernel/Phi.v"` "finds three hits in the tree -- this line and two Coq
`Require`s". The same sentence was copied into `.github/workflows/coq-kernel.yml`
as the stated rationale for the exit-2 fix, worded more strongly ("in the whole
tree").

- The real count at the commit that shipped that sentence was **14 lines in 11
  files**; at the tip it is 22 in 15. The claim was low by 4.7x.
- **Zero** of those hits is a Coq `Require`. A Coq require names the module
  `T27.Kernel.Phi` -- dotted, no slash, no `.v` -- so the string being grepped
  is structurally incapable of matching one.
- Two such `Require`s do exist, in `coq/Kernel/PhiFloat.v` and
  `coq/Kernel/FlowerE8Embedding.v`. So "three" was 1 real grep hit plus 2 real
  objects that grep did not find and cannot find.

The three objects were all real. The instrument was asserted to have found
them and had not. That is the defect: a count is a claim about an INSTRUMENT's
output, not about the world, and the two were merged in one sentence.

The conclusion the number supported -- rename the file and this gate dies
green -- survives, and is worse than stated: the path is pinned in 15 files
including two machine-readable conformance JSONs, so a rename that updates only
`coq/_CoqProject` leaves more than prose behind.

Both sites now name the command instead of a count, so the claim cannot rot.

Refs #3063
