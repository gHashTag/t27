-- Trinity S³AI Lean 4 Library
-- Root module: imports EVERY module under Trinity/.
-- Repaired 2026-08-12: the previous root imported 9 of 23, so `lake build` returned
-- green while never compiling the 12-file IcarusLowerable/ subdirectory,
-- GoldenFloatRoundTrip, Lemmas or T1Lucas. A build target that does not reach the
-- source proves nothing about it.

import Trinity.CorePhi
import Trinity.ExactIdentities
import Trinity.GoldenFloatRoundTrip
import Trinity.H4Derivations
import Trinity.H4Lagrangian
import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.AstInduction
import Trinity.IcarusLowerable.Completeness
import Trinity.IcarusLowerable.Emitter
import Trinity.IcarusLowerable.Equivalence
import Trinity.IcarusLowerable.Lemmas
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Semantics
import Trinity.IcarusLowerable.SemanticsTotal
import Trinity.IcarusLowerable.Soundness
import Trinity.IcarusLowerable.Verilog
import Trinity.Lemmas
import Trinity.NeutrinoMasses
import Trinity.T1Lucas
import Trinity.TernaryFPGABoot
import Trinity.TernaryGemm
import Trinity.TernaryInference
import Trinity.TernaryMac
