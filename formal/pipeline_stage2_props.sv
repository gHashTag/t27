// ============================================================================
// pipeline_stage2_compute properties.
//
// Wave 627. Third of the INDIRECT modules from Prop. 76, and the last
// non-trivial one -- the remaining six are combinational primitives inside
// trit_stdlib.sv. This is the MAC datapath, and Wave 615's undetected
// activation/requant mutation lived next door to it.
//
// The claims here are about the ACCUMULATION, not about the dot product. A
// second instance of `trit27_dot_product` is driven with the same inputs to
// give the expected per-chunk contribution: that is a shadow of the primitive,
// not an assumption that the primitive is correct, and it lets the properties
// say exactly what the surrounding logic must do with whatever the primitive
// returns.
//
// `accumulator` and `result` are assigned the same expression on every update
// and both reset to zero, so `$past(result)` is also the previous accumulator
// value -- which is why these properties can be stated from the ports alone.
//
// REQUIRES `-set-assumes` (Prop. 11) and `-flatten` (Prop. 7).
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module ps2_props (
    input wire        clk,
    input wire        rst_n,
    input wire        valid_in,
    input wire [53:0] input_chunk,
    input wire [53:0] weight_chunk,
    input wire        first_chunk,
    input wire        last_chunk
);

    wire               valid_out, result_final;
    wire signed [15:0] result;

    pipeline_stage2_compute dut (
        .clk(clk), .rst_n(rst_n),
        .valid_in(valid_in),
        .input_chunk(input_chunk), .weight_chunk(weight_chunk),
        .first_chunk(first_chunk), .last_chunk(last_chunk),
        .valid_out(valid_out), .result(result), .result_final(result_final)
    );

    // Shadow of the per-chunk contribution. Same primitive, same inputs -- this
    // checks the accumulator around it, and deliberately assumes nothing about
    // whether the dot product itself is right.
    wire signed [5:0] fv_dot;
    trit27_dot_product fv_simd (
        .input_vec(input_chunk), .weight_vec(weight_chunk), .result(fv_dot)
    );

    // A first chunk RESTARTS the sum. This is the property that separates "a
    // neuron's dot product" from "everything since reset": drop the first_chunk
    // test and the accumulator runs on across neuron boundaries.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(valid_in) && $past(first_chunk))
        a_first_chunk_restarts: assert (result == $past(fv_dot));

    // Every later chunk ADDS exactly its own contribution -- no more, no less.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(valid_in) && !$past(first_chunk))
        a_accumulates_one_chunk: assert (result == $past(result) + $past(fv_dot));

    // The result is held while no chunk is accepted. Without this, a datapath
    // that recomputed on idle cycles would satisfy both properties above and
    // still corrupt a neuron between chunks.
    always @(posedge clk) if (rst_n && $past(rst_n) && !$past(valid_in))
        a_result_held_when_idle: assert (result == $past(result));

    // valid_out is exactly "a last chunk was accepted last cycle" -- not "a
    // chunk was accepted", and not sticky.
    always @(posedge clk) if (rst_n && $past(rst_n))
        a_valid_out_follows_last: assert (valid_out == ($past(valid_in) && $past(last_chunk)));

endmodule

`default_nettype wire
