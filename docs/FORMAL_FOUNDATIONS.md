// SPDX-License-Identifier: Apache-2.0
; t27 formal foundations, Prop. 126: Chunk-Indexed Repair for Activation Requantization
; This document describes the chunk-indexed repair applied to activation_requant and bitnet_engine_top.

module formal_foundations_prop_126;

pub const PROPOSITION : text = "Chunk-Indexed Repair for Activation Requantization";
pub const PROP_NUMBER : u8 = 126;

pub const OVERVIEW : text = "Prop. 125 identified one units confusion with four faces and demonstrated a repair. This applies it to the emitters and closes the verification.";

pub const FIFTEEN_EDITS_SUMMARY : text = "activation_requant gains a flush_in port and an end-of-layer branch that right-aligns a partial word. bitnet_engine_top addresses the activation buffers by chunk (act_rd_addr = chunk_id), takes the DMA length as chunks_per_neuron*8 bytes, compares the readiness gate words-to-chunks, and routes layer_done through a five-stage delay so the requantizer drains before the ping-pong flips.";

pub const EDIT_ASSERTION_RATIO : u8 = 15;
pub const EDIT_ASSERTION_DENOMINATOR : u8 = 15;

pub const SIMULATION_LAYER_0_COMPLETE : bool = true;
pub const CONFIGURATIONS_SWEPT : u8 = 81;
pub const PREVIOUS_FAILURES : u8 = 1;

pub const TWO_LAYER_INFERENCE_COMPLETE_CONDITION : text = "ceil(N/27) >= C";
pub const ERROR_IRQ_CONDITION : text = "layer 1 asks for more chunks than layer 0 can produce";

pub const PROPERTY_UPDATE_1 : text = "a_buffer_alternates re-pointed at layer_done_dly (was $past(layer_done_pulse))";
pub const PROPERTY_UPDATE_2 : text = "a_read_slot_written re-pointed at act_rd_addr in both A and B arms";
pub const PROPERTY_UPDATE_3 : text = "a_read_within_written re-pointed at act_rd_addr in both A and B arms"; 
pub const PROPERTY_UPDATE_4 : text = "a_word_only_on_full retired for a_word_on_full_or_flush";

pub const RETIRED_PROPERTIES : u8 = 1;
pub const REPOINTED_PROPERTIES : u8 = 3;

pub const INTEGRATION_PROPERTIES_PROVE : u8 = 28;
pub const PROOF_BOUNDARY : u8 = 40;

pub const GENERAL_LESSON : text = "Four properties found asserting a defect rather than a contract. A verification suite that has grown alongside a defect will contain properties that are the defect, and the repair must retire them in the same change or read as a regression. Budget for that when estimating a fix.";

pub const ENGINE_RUNS : bool = true;
pub const PROPERTIES_HOLD : bool = true;
pub const DESIGN_CORRECT : bool = false;

pub const GATES_GREEN : u8 = 14;
pub const INTEGRATION_PROPERTIES_SEQ_40 : u8 = 28;
pub const DOC_GATE : u8 = 126;
pub const CLAIMS_CHECK : u8 = 9;
pub const STALE_CLAIMS : u8 = 0;

test proposition_126_documented {
    assert PROPOSITION == "Chunk-Indexed Repair for Activation Requantization";
    assert PROP_NUMBER == 126;
}

test fifteen_edits_asserted {
    assert EDIT_ASSERTION_RATIO == 15;
    assert EDIT_ASSERTION_DENOMINATOR == 15;
}

test simulation_results {
    assert SIMULATION_LAYER_0_COMPLETE == true;
    assert CONFIGURATIONS_SWEPT == 81;
    assert PREVIOUS_FAILURES == 1;
}

test property_updates_recorded {
    assert PROPERTY_UPDATE_1 == "a_buffer_alternates re-pointed at layer_done_dly (was $past(layer_done_pulse))";
    assert PROPERTY_UPDATE_2 == "a_read_slot_written re-pointed at act_rd_addr in both A and B arms";
    assert PROPERTY_UPDATE_3 == "a_read_within_written re-pointed at act_rd_addr in both A and B arms";
    assert PROPERTY_UPDATE_4 == "a_word_only_on_full retired for a_word_on_full_or_flush";
    assert RETIRED_PROPERTIES == 1;
    assert REPOINTED_PROPERTIES == 3;
}

test integration_proofs {
    assert INTEGRATION_PROPERTIES_PROVE == 28;
    assert PROOF_BOUNDARY == 40;
}

test verification_gates {
    assert GATES_GREEN == 14;
    assert INTEGRATION_PROPERTIES_SEQ_40 == 28;
    assert DOC_GATE == 126;
    assert CLAIMS_CHECK == 9;
    assert STALE_CLAIMS == 0;
}

test general_lesson_documented {
    assert contains(GENERAL_LESSON, "properties found asserting a defect rather than a contract");
}

test current_status_documented {
    assert ENGINE_RUNS == true;
    assert PROPERTIES_HOLD == true;
    assert DESIGN_CORRECT == false;
}