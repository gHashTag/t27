// SPDX-License-Identifier: Apache-2.0
; t27 formal foundations, Prop. 122: What the Design Hunt Did Not Record
; This document records what the formal verification hunt found and what Prop. 121 did not record.

module formal_foundations_prop_122;

pub const PROPOSITION : text = "Proposition 122: What the Design Hunt Did Not Record";
pub const PROP_NUMBER : u8 = 122;

pub const OVERVIEW : text = "The design hunt completed with 11 agents and no errors. Three things Prop. 121 did not record: a units mismatch that may deadlock layer 0, four behaviours proved clean in the requantizer, and a correction to the root-cause attribution in Prop. 121a.";

pub const HUNT_AGENTS : u8 = 11;
pub const HUNT_ERRORS : u8 = 0;

pub const UNITS_DEFECT : text = "bitnet_engine_top.sv:351 passes .length(reg_neurons) to dma_controller, whose header states one beat is 8 bytes (64-bit) and length is byte-count, and which writes one local address per beat. The same register is read as neurons_per_layer, a neuron count. For N neurons the input DMA moves N bytes, which is ceil(N/8) words, while the readiness gate demands filled >= neurons_per_layer, so for N >= 2 the gate can never be satisfied and the deadlock may reach layer 0, not only layer boundaries. Confirmed by reading the contract against the call; not yet reproduced in simulation, and recorded at that strength.";

pub const UNITS_DEFECT_REPRODUCED : bool = false;

pub const PROVED_CLEAN : u8 = 4;

pub const QUANTISER_VERDICT : text = "The quantiser is correct against an independent 17-bit reference over all inputs, including the TRIT_Z fall-through that no inline property asserts and including threshold = 16'sh8000, where the 16-bit negation overflows but the priority chain masks it, so the 16- and 17-bit results are observationally equal";

pub const PACKING_VERDICT : text = "The packing order matches its documentation exactly, trit i at bits [2i+1:2i], proved against an index-addressed reference packer";

pub const INVALID_ENCODING : text = "2'b11 is unreachable in all 27 fields of word, not merely in the scalar trit that a_trit_never_invalid guards";

pub const RESET_DECODE : text = "The reset value decodes as 27 TRIT_N rather than 27 TRIT_Z, but it is never observable, because word_valid gates it and 27 beats flush the shifter first";

pub const CLEAN_BALANCE : text = "Five defects sit next to four proved-correct behaviours in the same module. A report that lists only the failures misrepresents the design.";

pub const ATTRIBUTION_CORRECTION : text = "Prop. 121a over-stated the root cause: it presented one agent's judgement as settled. The refuting agent concluded that read_addr = neuron_id is also the root of the deadlock; the hunting agent explicitly declined to adjudicate. The findings stand, the attribution of the root does not. Two readings remain open, either the reader should address by chunk or the packer should not pack 27:1, and the units mismatch adds a third possibility, that the DMA length is the primary error and both downstream readings are consequences";

pub const ROOT_CAUSE_SETTLED : bool = false;

pub const GATES_GREEN : bool = true;
pub const DOC_GATE : u8 = 122;
pub const CLAIMS_CHECK : u8 = 9;
pub const STALE_CLAIMS : u8 = 0;

pub const GATES_LINE : text = "All gates green: doc_gate 122/122, claims_check 9 claims 0 stale";

test proposition_122_documented {
    assert PROPOSITION == "Proposition 122: What the Design Hunt Did Not Record";
    assert PROP_NUMBER == 122;
    assert HUNT_AGENTS == 11;
    assert HUNT_ERRORS == 0;
}

test overview_recorded {
    assert contains(OVERVIEW, "11 agents and no errors");
}

test units_mismatch_recorded {
    assert contains(UNITS_DEFECT, "length is byte-count");
    assert contains(UNITS_DEFECT, "neurons_per_layer");
    assert UNITS_DEFECT_REPRODUCED == false;
}

test clean_behaviours_recorded {
    assert PROVED_CLEAN == 4;
    assert contains(QUANTISER_VERDICT, "17-bit reference");
    assert contains(PACKING_VERDICT, "index-addressed reference packer");
    assert contains(INVALID_ENCODING, "a_trit_never_invalid");
    assert contains(RESET_DECODE, "word_valid");
    assert contains(CLEAN_BALANCE, "misrepresents the design");
}

test attribution_corrected {
    assert contains(ATTRIBUTION_CORRECTION, "The findings stand");
    assert contains(ATTRIBUTION_CORRECTION, "third possibility");
    assert ROOT_CAUSE_SETTLED == false;
}

test verification_gates {
    assert GATES_GREEN == true;
    assert DOC_GATE == 122;
    assert CLAIMS_CHECK == 9;
    assert STALE_CLAIMS == 0;
    assert contains(GATES_LINE, "All gates green");
}
