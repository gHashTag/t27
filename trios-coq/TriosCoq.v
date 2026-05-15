(** TriosCoq.v — master Require Export for trios-coq Coq SoT.
    L-DPC25 Lane X · lever-coq-spec-ext
    Issue: https://github.com/gHashTag/trinity-fpga/issues/104
    Author: admin@t27.ai
    Anchor: φ²+φ⁻²=3
    R5-HONEST: 74 _CoqProject paths post Lane Z; 75 after Lane X.
*)

(** Existing IGLA module (preserved) *)
Require Export IGLA.RMarker.

(** L-DPC24 Lane Z: 4-slot R-marker hyper-vector spec with R15 invariant *)
Require Export HoloRMarker4Slot.

(** L-DPC25 Lane X: holo_op alphabet + Q4 Qed invariant *)
Require Export HoloOpAlphabet.
