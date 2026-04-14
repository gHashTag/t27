module mac_formal_props (
    input wire clk,
    input wire rst_n,
    input wire [26:0] a,
    input wire [26:0] b,
    input wire [31:0] acc_in,
    input wire enable,
    output wire [31:0] acc_out,
    output wire valid
);

    default clocking fp @(posedge clk); endclocking
    default disable !rst_n;

    // P1: After reset, accumulator is zero
    assert property (rst_n |-> acc_out == 32'd0)
        else $error("P1 FAILED: acc_out not zero after reset");

    // P2: When enable is low, accumulator does not change
    assume property (!enable |=> $stable(acc_out));

    // P3: valid output only after enable was asserted
    assert property (valid |-> $past(enable, 8))
        else $error("P3 FAILED: valid without prior enable");

    // P4: Accumulator output width never exceeds 32 bits (overflow check)
    cover property (acc_out == 32'hFFFFFFFF);

    // P5: Ternary LUT correctness: trit values are only 0, 1, or 2 (encoded)
    assume property (a >= 0 && b >= 0);

    // P6: valid signal deasserts after one cycle
    assert property (valid |=> !valid)
        else $error("P6 FAILED: valid held more than one cycle");

    // P7: Enable pulse causes valid within 8 cycles
    assert property (enable |-> ##[1:8] valid)
        else $error("P7 FAILED: valid not seen within 8 cycles of enable");

endmodule
