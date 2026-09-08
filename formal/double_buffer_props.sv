// ============================================================================
// double_buffer_ctrl properties.
//
// Wave 625. This module had NO properties of its own until now, which Prop. 76
// surfaced: it is 33 lines, it implements the ping-pong, and the ping-pong
// produced the campaign's longest-running defect -- three changes across eight
// waves (Props. 33, 46b, 47). Every one of those was diagnosed and fixed at the
// ENGINE level, because that is where the symptom was observable, and nobody
// went back to constrain the thing that produced it.
//
// The properties below are the invariants those three fixes imply, stated at
// the level the logic actually lives at.
//
// REQUIRES `-set-assumes` (Prop. 11) and `-flatten` (Prop. 7).
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module db_props (
    input wire        clk,
    input wire        rst_n,
    input wire        layer_done,
    input wire [5:0]  current_layer,
    input wire [11:0] neuron_id
);

    wire        use_buffer_a;
    wire [11:0] read_addr, write_addr;

    double_buffer_ctrl dut (
        .clk(clk), .rst_n(rst_n),
        .layer_done(layer_done), .current_layer(current_layer),
        .use_buffer_a(use_buffer_a),
        .read_addr(read_addr), .write_addr(write_addr),
        .neuron_id(neuron_id)
    );

    // The invariant Prop. 33 was about: the buffers alternate, and they
    // alternate on the layer boundary rather than on anything else. The
    // mutation harness carries "double buffer stops alternating" as a
    // hand-written mutant precisely because this failing is what a stuck
    // ping-pong looks like -- until now it was only caught at the engine.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(layer_done))
        a_toggles_on_layer_done: assert (use_buffer_a == ~$past(use_buffer_a));

    // The other half, and the one a "fix" for the first can break: nothing
    // else moves it. A controller that toggled every cycle would satisfy the
    // property above on every layer_done and still be wrong.
    always @(posedge clk) if (rst_n && $past(rst_n) && !$past(layer_done))
        a_stable_without_layer_done: assert (use_buffer_a == $past(use_buffer_a));

    // Reset state. Layer 0 reads A and writes B; the engine's read/write
    // selects are written assuming exactly this polarity, and Prop. 46b was a
    // configuration latch that went wrong because the phase was assumed rather
    // than established.
    //
    // `fv_started` is not decoration. Under `-set-init-zero` every register
    // begins at 0, so at time zero `$past(rst_n)` reads 0 whether or not a
    // reset ever happened, and the guard `rst_n && !$past(rst_n)` fires on a
    // state that is an artifact of the initialisation convention rather than a
    // reset. Without this the property refutes on the REAL design -- and worse,
    // it made the whole suite refute, so every mutant appeared to be detected.
    reg fv_started;
    always @(posedge clk) fv_started <= 1'b1;

    always @(posedge clk) if (fv_started && rst_n && !$past(rst_n))
        a_reset_reads_a: assert (use_buffer_a == 1'b1);

    // Read and write index the same slot -- in different buffers, which is the
    // point of the ping-pong. Prop. 31's DMA defect was exactly this pair
    // diverging in another module, so it is stated here rather than assumed.
    always @(posedge clk) if (rst_n)
        a_addresses_agree: assert (read_addr == write_addr);

endmodule

`default_nettype wire
