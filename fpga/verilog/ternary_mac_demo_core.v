`default_nettype none

// ternary_mac_demo_core.v -- clock-source-independent core of the IGLA RACE
// on-board demo.
//
// Split out of ternary_mac_demo_top_v2 so that the sequencer can be driven by
// an ordinary clock port.  That matters for two reasons:
//
//   * Formal verification.  yosys cannot reason through a STARTUPE2 blackbox,
//     so a design whose only clock comes from that primitive cannot be
//     model-checked.  With `clk` as a port, `fpga/formal/prove_demo_core.ys`
//     bounded-model-checks the accumulator invariant directly.
//   * Simulation.  The testbench drives `clk` itself instead of relying on a
//     hand-written STARTUPE2 stub.
//
// The board-level wrapper (ternary_mac_demo_top_v2) adds STARTUPE2 and nothing
// else, so what is verified here is what is synthesized there.
//
// Behaviour: one datapath step every 2^PRESCALE_BITS clocks.  Each step applies
// the next weight of the repeating sequence {+1, 0, -1, 0} to a = +1 and feeds
// acc_out back into acc_in, so the accumulator genuinely accumulates and all
// three decode branches of ternary_mac_top stay live.
//
// Invariant (proved as theorem T3): once reset is released, acc_out is always
// 0 or +1, so it is never negative and led_t23 never lights.

module ternary_mac_demo_core #(
    parameter integer PRESCALE_BITS = 24
) (
    input  wire clk,
    output wire led_r23,   // active-low: lit when the accumulator is non-zero
    output wire led_t23    // active-low: lit when the accumulator is negative
);
    // ------------------------------------------------------------------
    // Prescaler: one datapath step per 2^PRESCALE_BITS clocks.
    // ------------------------------------------------------------------
    reg [PRESCALE_BITS-1:0] prescale = {PRESCALE_BITS{1'b0}};
    wire step = (prescale == {PRESCALE_BITS{1'b1}});

    always @(posedge clk) begin
        prescale <= prescale + 1'b1;
    end

    // ------------------------------------------------------------------
    // Power-on reset: hold the MAC in reset for the first three steps so the
    // accumulator starts from a known zero without an external button.
    // ------------------------------------------------------------------
    reg [1:0] por = 2'b00;
    always @(posedge clk) begin
        if (step && por != 2'b11)
            por <= por + 1'b1;
    end
    wire rst_n = (por == 2'b11);

    // ------------------------------------------------------------------
    // Weight sequence {+1, 0, -1, 0} -- covers both zero encodings.
    // ------------------------------------------------------------------
    reg [1:0] phase = 2'b00;
    always @(posedge clk) begin
        if (!rst_n)
            phase <= 2'b00;
        else if (step)
            phase <= phase + 1'b1;
    end

    reg [1:0] w_code;
    always @(*) begin
        case (phase)
            2'd0:    w_code = 2'b01;  // +1
            2'd1:    w_code = 2'b00;  // zero, encoding A
            2'd2:    w_code = 2'b10;  // -1
            default: w_code = 2'b11;  // zero, encoding B
        endcase
    end

    // ------------------------------------------------------------------
    // The MAC, with its output fed back as the accumulator input.
    // ------------------------------------------------------------------
    wire signed [7:0]  a = 8'sd1;
    wire signed [31:0] acc_out;

    ternary_mac_top mac (
        .clk(clk),
        .rst_n(rst_n),
        .en(step),
        .a(a),
        .w_code(w_code),
        .acc_in(acc_out),
        .acc_out(acc_out)
    );

    // Whole-accumulator predicates: a stuck or constant-folded datapath is
    // immediately visible, unlike v1's raw low-order bits.
    assign led_r23 = ~(acc_out != 32'sd0);
    assign led_t23 = ~(acc_out[31]);

`ifdef FORMAL
    // ------------------------------------------------------------------
    // Theorem T3 -- checked by fpga/formal/prove_demo_core.ys.
    //
    // Once reset is released the accumulator is confined to {0, +1}; it is
    // therefore never negative, so the sign LED never lights.  This is what
    // makes the on-board pass criterion falsifiable: the launch plan predicts
    // "led_t23 stays dark", and T3 is why that is a prediction rather than a
    // hope.  A board that lights led_t23 contradicts the model.
    // ------------------------------------------------------------------
    always @(posedge clk) begin
        if (rst_n) begin
            // T3a -- the accumulator never leaves {0, +1}.
            assert (acc_out == 32'sd0 || acc_out == 32'sd1);

            // T3b -- it is therefore never negative, so the sign LED is dark.
            assert (led_t23 == 1'b1);

            // T3c -- the activity LED is exactly the "accumulator non-zero"
            // predicate, so a stuck LED implies a stuck accumulator.
            assert (led_r23 == ~(acc_out != 32'sd0));
        end
    end
`endif
endmodule
