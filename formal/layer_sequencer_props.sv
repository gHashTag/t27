// ============================================================================
// Formal properties for `layer_sequencer` (BitNet HLS, W36b / R-BN-2)
//
// Immediate assertions, not concurrent SVA (Yosys frontend limits, Props 2/6).
// REQUIRES `-set-assumes` (Prop 11) and `-flatten` (Prop 7).
//
// a_neuron_in_range is a regression witness for a real defect fixed
// 2026-08-09: `neuron_id == num_neurons - 1` compares against 16'hFFFF when
// num_neurons is zero, so the terminator never matched and the sequencer
// emitted valid work for neuron indices 0,1,2,... indefinitely. The chunk twin
// of this property proved on the same RTL -- the zero case had been handled
// for chunks and missed for neurons. See FORMAL_FOUNDATIONS Prop. 13.
//
// Prove with:
//   yosys -p "read_verilog -sv -formal layer_sequencer.sv \
//             formal/layer_sequencer_props.sv; \
//             prep -top ls_props -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 12 -set-init-zero -set-assumes"
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module ls_props (
    input wire        clk,
    input wire        rst_n,
    input wire        start,
    input wire [15:0] num_neurons,
    input wire [7:0]  num_chunks
);
    wire [15:0] neuron_id;
    wire [7:0]  chunk_id;
    wire        first_chunk, last_chunk, valid, done;

    layer_sequencer dut (
        .clk(clk), .rst_n(rst_n), .start(start),
        .num_neurons(num_neurons), .num_chunks(num_chunks),
        .neuron_id(neuron_id), .chunk_id(chunk_id),
        .first_chunk(first_chunk), .last_chunk(last_chunk),
        .valid(valid), .done(done)
    );

    // The descriptor is held stable for the duration, as the CSR block drives it.
    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_neurons == $past(num_neurons));
    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_chunks  == $past(num_chunks));

    // Harness sanity: a tautology. If this refutes, the run is not evaluating
    // what it appears to (Prop. 7's -flatten trap surfaces exactly this way).
    always @(posedge clk) if (rst_n)
        a_sanity: assert (chunk_id == chunk_id);

    // REGRESSION WITNESS: never emit work for a neuron at or beyond the count.
    always @(posedge clk) if (rst_n && valid)
        a_neuron_in_range: assert (neuron_id < num_neurons);

    // The twin that already held -- kept to show what the defect was *not*.
    always @(posedge clk) if (rst_n && valid)
        a_chunk_in_range: assert (chunk_id < num_chunks);

    // first/last flags must agree with the chunk index they describe.
    always @(posedge clk) if (rst_n && valid && $past(rst_n))
        a_last_chunk_consistent: assert (!last_chunk || $past(chunk_id) == $past(num_chunks) - 8'd1);

endmodule

`default_nettype wire
