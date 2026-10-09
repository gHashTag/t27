// ============================================================================
// trit_stdlib primitive properties -- EXHAUSTIVE, not bounded.
//
// Wave 628. These five are the last INDIRECT modules from Prop. 76, and they
// are purely combinational. That changes what a proof means here: with no
// state, `sat -verify -prove-asserts -seq 1` quantifies over EVERY input
// combination, so a PROVED verdict carries no depth caveat and no induction
// argument. These are the first unbounded module results in this campaign
// outside the k-induction suite, and the only ones that need no bound audit
// (Prop. 68).
//
// Encoding, from the RTL: 2'b00 = -1, 2'b01 = 0, 2'b10 = +1, 2'b11 reserved.
//
// The reserved encoding is ASSUMED away, and that assumption is not free: the
// primitives disagree about it. `adder_tree_27` maps 2'b11 to 0 via an else
// branch, while `trit27_parallel_multiply` tests `ai == bi` and so treats
// 2'b11 * 2'b11 as (+1). Both are defensible readings of "reserved" and they
// are inconsistent with each other -- which is worth knowing and is recorded
// here rather than hidden by the assumption that makes the proofs go through.
//
// Prop. 79a deliberately left the dot product's own correctness unstated: the
// MAC properties checked the accumulator around it. This states it.
//
// REQUIRES `-set-assumes` (Prop. 11) and `-flatten` (Prop. 7).
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

// Shared decode. A function, not a wire, so every wrapper reads one definition.
`define TV(t) (((t) == 2'b00) ? -7'sd1 : ((t) == 2'b10) ? 7'sd1 : 7'sd0)
`define VALID(t) ((t) != 2'b11)

module tha_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] sum, carry;
    trit_half_adder dut (.a(a), .b(b), .sum(sum), .carry(carry));

    always @(*) assume (`VALID(a) && `VALID(b));

    // The balanced-ternary adder axiom: a digit and a carry weighted by three.
    always @(*)
        a_half_adder_axiom: assert (`TV(sum) + 7'sd3 * `TV(carry)
                                    == `TV(a) + `TV(b));
endmodule

module tfa_props (input wire [1:0] a, input wire [1:0] b, input wire [1:0] cin);
    wire [1:0] sum, cout;
    trit_full_adder dut (.a(a), .b(b), .cin(cin), .sum(sum), .cout(cout));

    always @(*) assume (`VALID(a) && `VALID(b) && `VALID(cin));

    always @(*)
        a_full_adder_axiom: assert (`TV(sum) + 7'sd3 * `TV(cout)
                                    == `TV(a) + `TV(b) + `TV(cin));
endmodule

module tpm_props (input wire [53:0] a, input wire [53:0] b);
    wire [53:0] result;
    trit27_parallel_multiply dut (.a(a), .b(b), .result(result));

    // Procedural loops, not `generate`: yosys rejects an assertion placed
    // inside a generate-for with "Cannot add procedural assertion".
    integer k;
    reg        all_valid;
    reg [53:0] expect_result;
    always @(*) begin
        all_valid     = 1'b1;
        expect_result = 54'd0;
        for (k = 0; k < 27; k = k + 1) begin
            all_valid = all_valid && `VALID(a[k*2 +: 2]) && `VALID(b[k*2 +: 2]);
            expect_result[k*2 +: 2] =
                (`TV(a[k*2 +: 2]) * `TV(b[k*2 +: 2]) == 7'sd1)  ? 2'b10 :
                (`TV(a[k*2 +: 2]) * `TV(b[k*2 +: 2]) == -7'sd1) ? 2'b00 : 2'b01;
        end
    end
    always @(*) assume (all_valid);

    // All 27 lanes at once, over every input combination.
    always @(*)
        a_lanes_are_products: assert (result == expect_result);
endmodule

module at27_props (input wire [53:0] trits);
    wire signed [5:0] sum;
    adder_tree_27 dut (.trits(trits), .sum(sum));

    integer k;
    reg signed [7:0] expect_sum;
    always @(*) begin
        expect_sum = 8'sd0;
        for (k = 0; k < 27; k = k + 1)
            expect_sum = expect_sum + `TV(trits[k*2 +: 2]);
    end

    reg all_valid;
    always @(*) begin
        all_valid = 1'b1;
        for (k = 0; k < 27; k = k + 1)
            all_valid = all_valid && `VALID(trits[k*2 +: 2]);
    end
    always @(*) assume (all_valid);

    always @(*)
        a_tree_sums_all_27: assert ($signed({{2{sum[5]}}, sum}) == expect_sum);
endmodule

module dot_props (input wire [53:0] a, input wire [53:0] b);
    wire signed [5:0] result;
    trit27_dot_product dut (.input_vec(a), .weight_vec(b), .result(result));

    integer k;
    reg signed [7:0] expect_dot;
    always @(*) begin
        expect_dot = 8'sd0;
        for (k = 0; k < 27; k = k + 1)
            expect_dot = expect_dot + `TV(a[k*2 +: 2]) * `TV(b[k*2 +: 2]);
    end

    reg all_valid;
    always @(*) begin
        all_valid = 1'b1;
        for (k = 0; k < 27; k = k + 1)
            all_valid = all_valid && `VALID(a[k*2 +: 2]) && `VALID(b[k*2 +: 2]);
    end
    always @(*) assume (all_valid);

    // What Prop. 79a left unstated: the dot product is the dot product.
    always @(*)
        a_dot_product_correct: assert ($signed({{2{result[5]}}, result}) == expect_dot);
endmodule

// The dot product's RANGE, asserted with NO validity assumption.
//
// Wave 631. `a_dot_product_correct` above needs `all_valid` to state the exact
// value, because the reserved code 2'b11 has no defined trit. The BOUND needs
// nothing: the decoder maps every code that is neither TRIT_N nor TRIT_P --
// 2'b11 included -- to zero, so no input whatsoever can push the sum of 27
// products past 27 in magnitude.
//
// This is stated separately because it is the fact the accumulator argument in
// `pipeline_stage2_props` rests on, and a fact a proof depends on should be
// proved rather than left implicit in another property's cone. It is also the
// unconditional half: were this only available under `all_valid`, the
// accumulator bound would silently inherit an assumption about BRAM contents
// that nothing enforces.
module dot_range_props (input wire [53:0] a, input wire [53:0] b);
    wire signed [5:0] result;
    trit27_dot_product dut (.input_vec(a), .weight_vec(b), .result(result));

    always @(*)
        a_dot_within_27: assert (result >= -6'sd27 && result <= 6'sd27);
endmodule

// Non-vacuity. Every wrapper above assumes inputs are valid trits, and an
// assumption that admitted nothing would make all five proofs vacuous. This
// asserts no valid vector exists, so it must REFUTE. Prop. 12a's oracle, in the
// only form that makes sense for stateless logic.
module at27_alive (input wire [53:0] trits);
    integer k;
    reg all_valid;
    always @(*) begin
        all_valid = 1'b1;
        for (k = 0; k < 27; k = k + 1)
            all_valid = all_valid && `VALID(trits[k*2 +: 2]);
    end
    always @(*) a_valid_inputs_exist: assert (!all_valid);
endmodule

`default_nettype wire
