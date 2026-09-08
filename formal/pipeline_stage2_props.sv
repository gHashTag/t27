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

// ============================================================================
// Wave 631: does the 16-bit accumulator actually fit?
//
// `a_accumulates_one_chunk` above is a 16-bit equation, so it holds *modulo
// 2^16* -- it is satisfied by an accumulator that wraps. Nothing in this file
// said whether the width is sufficient, and the module cannot answer that on
// its own: it has NO chunk counter and NO num_chunks input. It accumulates for
// as long as valid_in is held with first_chunk low, so in isolation it
// overflows after 1214 chunks of +27.
//
// The width is sufficient only because of a CALLER CONTRACT that appears
// nowhere in the module: `layer_sequencer` walks chunk_id over an 8-bit port,
// so at most 255 chunks separate two first_chunk strobes, and 255 * 27 = 6885
// sits well inside [-32768, +32767]. Widening num_chunks to 16 bits for larger
// layers -- an ordinary-looking change to an unrelated file -- silently
// reintroduces the wrap. That reasoning was nowhere in the tree before this
// wave. It is now written down, and checked.
//
// Separate wrapper, deliberately: these prove by INDUCTION rather than to a
// depth, which matters because the overflow they rule out is 1214 cycles away
// and no feasible bound reaches it -- a bounded run would report "proves" and
// mean nothing. Induction needs no shadow instance, and leaving it out keeps
// one 27-input adder tree in the cone instead of two.
// ============================================================================

module ps2_bound (
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

    // Accumulations since the last restart. Formal-only, and 16 bits wide
    // rather than 8 so that the contract below is an assumption we STATE, not
    // a wrap we silently inherit from the counter's own width.
    reg [15:0] fv_chunks;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) fv_chunks <= 16'd0;
        else if (valid_in) fv_chunks <= first_chunk ? 16'd1 : fv_chunks + 16'd1;

    // The contract itself, and it is load-bearing rather than convenient:
    // relax it and both properties below become FALSE. Stated as an assumption
    // because it is a fact about the sequencer, which this wrapper bypasses.
    always @(posedge clk) assume (fv_chunks <= 16'd255);

    wire signed [31:0] fv_acc32 = result;
    wire signed [31:0] fv_reach = $signed({16'd0, fv_chunks}) * 32'sd27;

    // The inductive invariant: after n accumulations the sum cannot have left
    // [-27n, +27n], because each chunk contributes a dot product of 27 trits.
    // That per-chunk bound is not assumed here -- it is proved unconditionally
    // and exhaustively by `dot_range_props` in trit_stdlib_props.sv.
    always @(posedge clk) if (rst_n)
        a_accumulator_within_chunk_bound:
            assert (fv_acc32 <= fv_reach && fv_acc32 >= -fv_reach);

    // The consequence worth having: the accumulator always retains headroom
    // for one more chunk, so the next 16-bit addition cannot wrap. This is the
    // no-overflow claim, stated without reference to the dot product's value.
    always @(posedge clk) if (rst_n)
        a_accumulator_has_headroom:
            assert (fv_acc32 <= 32'sd32740 && fv_acc32 >= -32'sd32740);

endmodule

// Non-vacuity for the wrapper above. `assume (fv_chunks <= 255)` is the only
// assumption there, and an assumption that emptied the state space would make
// both properties hold for the worst possible reason. This asserts the
// accumulator is always zero, which is FALSE of any design that accumulates
// anything -- so it must REFUTE, and a refutation is the evidence that the
// contract constrains the chunk count without freezing the datapath.
// Prop. 12a's oracle, applied to an assumption rather than to a suite.
module ps2_bound_alive (
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

    reg [15:0] fv_chunks;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) fv_chunks <= 16'd0;
        else if (valid_in) fv_chunks <= first_chunk ? 16'd1 : fv_chunks + 16'd1;

    always @(posedge clk) assume (fv_chunks <= 16'd255);

    always @(posedge clk) if (rst_n)
        a_accumulator_is_always_zero: assert (result == 16'sd0);

endmodule

`default_nettype wire
