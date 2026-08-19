`default_nettype none

// mvp_ternary_classifier_check.v -- the self-check, on an ordinary clock port.
//
// Split out of mvp_ternary_classifier_top.v for the same reason
// ternary_mac_demo_core.v is split out of its top: a module containing
// STARTUPE2 cannot be simulated without a vendor stub and cannot be reached by
// yosys model checking.  Everything that decides PASS or FAIL lives here, so
// the on-silicon verdict is checked in simulation first.  A self-checking
// harness that is itself unchecked is not evidence.
//
// See mvp_ternary_classifier_top.v for the pass criterion and the LED map.
// Refs #1959

module mvp_ternary_classifier_check #(
    parameter integer PRESCALE_BITS = 24
) (
    input  wire clk,
    output wire led_r23,
    output wire led_t23
);
    // ---- Sequencer: sweeps the FULL 8-bit input space, 0..255 ----
    //
    // WHY ALL 256 AND NOT THE TEN REFERENCE VECTORS.  A first version drove
    // only the ten vectors from the spec header.  It passed, and it was weak:
    // with ten constants on the input, synthesis is free to constant-fold the
    // classifier down to a ten-entry lookup and the LED would then attest to a
    // circuit that is not the one being claimed.  The measurement showed it --
    // network plus checker came to the same 83 LUT the network alone costs
    // with a free input, which is only possible if the network shrank.
    //
    // Sweeping every input keeps the general circuit instantiated, and buys a
    // second property for free: the invariant below is checked 256 times per
    // cycle instead of ten.
    // The sweep runs at the full clock rate: all 256 inputs in 256 cycles,
    // about 3.9 us at CFGMCLK, so the entire input space is re-verified
    // roughly 250,000 times per second.  The blink rate is derived
    // separately below -- tying the two together would force a choice between
    // a visible LED and timely coverage, and there is no reason to choose.
    reg [7:0] vec_x = 8'd0;
    always @(posedge clk) vec_x <= vec_x + 1'b1;
    wire step = 1'b1;

    // ---- The network under test: ZERO DSP ----
    wire [7:0] result;
    wire       ready;
    IglaMvpTernaryClassifier dut (
        .clk(clk),
        .rst_n(1'b1),
        .en(1'b1),
        .x(vec_x),
        .ready(ready),
        .result(result)
    );

    // ---- Expectation 1: the ten reference values, transcribed from the spec
    // ---- header, where each was computed independently BEFORE the
    // ---- implementation ran.  `checked` marks the inputs with a reference.
    reg [7:0] expect_class;
    reg       checked;
    always @(*) begin
        checked = 1'b1;
        case (vec_x)
            8'd0:   expect_class = 8'd0;
            8'd7:   expect_class = 8'd0;
            8'd60:  expect_class = 8'd1;
            8'd224: expect_class = 8'd2;
            8'd255: expect_class = 8'd0;
            8'd3:   expect_class = 8'd0;
            8'd24:  expect_class = 8'd1;
            8'd192: expect_class = 8'd2;
            8'd15:  expect_class = 8'd0;
            8'd240: expect_class = 8'd2;
            default: begin expect_class = 8'd0; checked = 1'b0; end
        endcase
    end

    // ---- Expectation 2: an invariant that must hold for ALL 256 inputs.
    // ---- argmax over three classes cannot return anything but 0, 1 or 2.
    // ---- Weaker than a reference value, but it covers the 246 inputs no
    // ---- reference exists for, and it is the property a specialised or
    // ---- mis-routed circuit is most likely to break.
    wire in_range = (result <= 8'd2);

    // ---- Sticky verdict: falls on the first violation of either, never rises
    reg ok = 1'b1;
    always @(posedge clk)
        if (step && ((checked && (result != expect_class)) || !in_range))
            ok <= 1'b0;

    // ---- Heartbeat: a slow, human-visible divider, independent of the sweep
    reg [PRESCALE_BITS-1:0] prescale = {PRESCALE_BITS{1'b0}};
    always @(posedge clk) prescale <= prescale + 1'b1;
    wire beat = prescale[PRESCALE_BITS-1];

    assign led_r23 = beat & ok;   // blinks only while every vector matches
    assign led_t23 = ~ok;         // lit only after a wrong class
endmodule

`default_nettype wire
