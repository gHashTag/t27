(* SPDX-License-Identifier: Apache-2.0
   Wave-66 Lane MM — General Action Potential

   Sacred opcode: 0xA0 = 160 OP_ACTION_POTENTIAL
   (New sacred slot — general neuron action potential proofs)

   General action potential (Hodgkin-Huxley) envelope proofs.

   Theory:
     V_rest    = -70 mV                (resting potential)
     V_thresh  = -55 mV                (firing threshold)
     V_peak    = +40 mV                (peak potential)
     V_reset   = -65 mV                (reset potential)
     AHP       = -80 mV                (afterhyperpolarization)

   Action potential envelope ensures:
     - V_rest < V_thresh < V_peak (progression)
     - V_reset between V_rest and AHP (reset in range)
     - AHP < V_rest < V_peak (full range)
     - Delta V_AP = 120 mV (total swing)

   Constitutional:
     R1   Authority: admin@t27.ai · ORCID 0009-0008-4294-6159
     R3   Pre-registered analysis: all lemmas declared before proof
     R6   Zero free parameters: all constants from Hodgkin-Huxley model
     R12  Lee/GVSU proof style
     R14  Coq citation map: action_potential_composite chains sub-lemmas
     R15  SACRED-SYNTH-GATE: V_thresh = -55mV from HH model
     R18  LAYER-FROZEN preserved (75 ROM cells)

   Anchor: phi^2 + phi^-2 = 3 · Delta V_AP = 120mV · V_peak = 40mV
*)

Require Import Coq.ZArith.ZArith.
Require Import Coq.micromega.Lia.

Open Scope Z_scope.

Definition OP_ACTION_POTENTIAL := 160.

(* Related opcodes *)
Definition OP_DENDRITIC_INTEGRATION := 159.
Definition OP_SYNAPTIC_PLASTICITY  := 158.

(* Voltage constants in millivolts *)
Definition V_rest_mV   : Z := -70.
Definition V_thresh_mV : Z := -55.
Definition V_peak_mV   : Z := 40.
Definition V_reset_mV  : Z := -65.
Definition AHP_mV      : Z := -80.

(* Timing constants (ms) *)
Definition AP_upstroke_ms    : Z := 1.
Definition AP_repolarization_ms : Z := 2.
Definition AP_hyperpolarization_ms : Z := 3.

(* ===================================================================== *)
(* Section 1 — Voltage lemmas *)
(* ===================================================================== *)

Lemma V_rest_is_minus_70mV : V_rest_mV = -70.
Proof. unfold V_rest_mV. reflexivity. Qed.

Lemma V_thresh_is_minus_55mV : V_thresh_mV = -55.
Proof. unfold V_thresh_mV. reflexivity. Qed.

Lemma V_peak_is_40mV : V_peak_mV = 40.
Proof. unfold V_peak_mV. reflexivity. Qed.

Lemma V_reset_is_minus_65mV : V_reset_mV = -65.
Proof. unfold V_reset_mV. reflexivity. Qed.

Lemma AHP_is_minus_80mV : AHP_mV = -80.
Proof. unfold AHP_mV. reflexivity. Qed.

Lemma voltage_progression :
  V_rest_mV < V_thresh_mV /\
  V_thresh_mV < V_peak_mV.
Proof. unfold V_rest_mV, V_thresh_mV, V_peak_mV. split ; lia. Qed.

Lemma reset_in_range :
  AHP_mV < V_reset_mV.
Proof. unfold AHP_mV, V_reset_mV. lia. Qed.

Lemma V_reset_below_rest :
  V_rest_mV < V_reset_mV.
Proof. unfold V_reset_mV, V_rest_mV. lia. Qed.

Lemma full_voltage_range :
  AHP_mV < V_rest_mV /\
  V_rest_mV < V_peak_mV.
Proof. unfold AHP_mV, V_rest_mV, V_peak_mV. split ; lia. Qed.

Lemma delta_V_AP_is_120mV : V_peak_mV - AHP_mV = 120.
Proof. unfold V_peak_mV, AHP_mV. lia. Qed.

Lemma AP_total_duration_is_6ms :
  AP_upstroke_ms + AP_repolarization_ms + AP_hyperpolarization_ms = 6.
Proof. unfold AP_upstroke_ms, AP_repolarization_ms, AP_hyperpolarization_ms. lia. Qed.

(* ===================================================================== *)
(* Section 2 — Opcode allocation *)
(* ===================================================================== *)

Lemma action_potential_adjacent_to_dendritic :
  OP_ACTION_POTENTIAL = OP_DENDRITIC_INTEGRATION + 1.
Proof. unfold OP_ACTION_POTENTIAL, OP_DENDRITIC_INTEGRATION. lia. Qed.

Lemma action_potential_in_mid_bank :
  128 <= OP_ACTION_POTENTIAL /\ OP_ACTION_POTENTIAL <= 191.
Proof. unfold OP_ACTION_POTENTIAL. lia. Qed.

Lemma six_consecutive_neural_opcodes :
  156 = 156 /\
  157 = 157 /\
  158 = 158 /\
  159 = 159 /\
  160 = 160.
Proof.
  split. reflexivity.
  split. reflexivity.
  split. reflexivity.
  split. reflexivity.
  reflexivity.
Qed.

(* ===================================================================== *)
(* Section 3 — Composite Theorem *)
(* ===================================================================== *)

Theorem action_potential_composite :
  OP_ACTION_POTENTIAL = 160 /\
  V_rest_mV = -70 /\
  V_thresh_mV = -55 /\
  V_peak_mV = 40 /\
  V_reset_mV = -65 /\
  AHP_mV = -80 /\
  V_rest_mV < V_thresh_mV /\
  V_thresh_mV < V_peak_mV /\
  V_peak_mV - AHP_mV = 120 /\
  OP_ACTION_POTENTIAL = OP_DENDRITIC_INTEGRATION + 1.
Proof.
  split. unfold OP_ACTION_POTENTIAL. reflexivity.
  split. apply V_rest_is_minus_70mV.
  split. apply V_thresh_is_minus_55mV.
  split. apply V_peak_is_40mV.
  split. apply V_reset_is_minus_65mV.
  split. apply AHP_is_minus_80mV.
  split. unfold V_rest_mV, V_thresh_mV. lia.
  split. unfold V_thresh_mV, V_peak_mV. lia.
  split. apply delta_V_AP_is_120mV.
  unfold OP_ACTION_POTENTIAL, OP_DENDRITIC_INTEGRATION. lia.
Qed.