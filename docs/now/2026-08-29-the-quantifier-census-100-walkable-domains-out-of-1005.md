# NOW -- The quantifier census: 100 walkable domains out of 1005 (2026-08-29)

## The quantifier census: 100 walkable domains out of 1005 (Refs #2774)

- three independently written proposals for #2774 disagreed on the lowering and agreed exactly on the first step: report before you lower
- 1005 quantified clauses in four notations; at a 65536 ceiling 100 are walkable, 222 finite but over it, 544 unbounded, 139 have no binder a reader can resolve
- all 135 suffix forms (for all Trit, for any a b in {1,-1}) are prose with no binder -- the small domains are exactly the ones written without one
- no guard is read: x.len() == 4 narrows nothing here, because that is the part that needs a semantics and the report must not decide one quietly
