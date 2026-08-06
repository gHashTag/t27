import Lake
open Lake DSL

package «Trinity» where
  -- add package configuration options here

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git"

@[default_target]
lean_lib «Trinity» where
  -- add library configuration options here
