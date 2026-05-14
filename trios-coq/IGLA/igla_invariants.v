(* IGLA Invariants — Master Import File *)
(* Compile order: see below *)
(* Trinity Identity: phi^2 + phi^-2 = 3 *)
(* Zenodo DOI: 10.5281/zenodo.19227877 *)

(* Compile order (dependency chain):
   1. lucas_closure_gf16.v     — INV-5: phi^2n + phi^-2n in Z
   2. gf16_precision.v         — INV-3: GF16 safe domain
   3. nca_entropy_stability.v  — INV-4: NCA entropy band
   4. nca_entropy_band.v       — INV-4: entropy band width
   5. lr_phi_optimality.v      — INV-1/INV-8: LR phi band
   6. lr_convergence.v         — INV-1: descent lemma
   7. bpb_decreases.v          — INV-1: BPB monotone decrease
   8. asha_champion_survives.v — INV-2: ASHA champion survival
   9. gf16_safe_domain.v       — INV-3: GF16 safe domain (alternate)
  10. igla_asha_bound.v        — INV-10: ASHA rungs Trinity
   11. igla_found_criterion.v   — INV-7: Victory gate (L7)
   12. ema_decay_valid.v        — INV-6: EMA cos schedule [0.996,1.0]
*)
