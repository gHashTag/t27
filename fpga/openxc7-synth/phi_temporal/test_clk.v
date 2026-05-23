`default_nettype none
module test_clk (
    output wire led_d5,
    output wire led_d6,
    output wire led_j26
);
    (* KEEP="TRUE", DONT_TOUCH="TRUE" *)
    wire [15:0] ring;
    assign ring[0]  = ~ring[15];
    assign ring[1]  = ~ring[0];
    assign ring[2]  = ~ring[1];
    assign ring[3]  = ~ring[2];
    assign ring[4]  = ~ring[3];
    assign ring[5]  = ~ring[4];
    assign ring[6]  = ~ring[5];
    assign ring[7]  = ~ring[6];
    assign ring[8]  = ~ring[7];
    assign ring[9]  = ~ring[8];
    assign ring[10] = ~ring[9];
    assign ring[11] = ~ring[10];
    assign ring[12] = ~ring[11];
    assign ring[13] = ~ring[12];
    assign ring[14] = ~ring[13];
    assign ring[15] = ~ring[14];
    wire clk = ring[0];

    reg [24:0] counter = 0;
    always @(posedge clk) counter <= counter + 1;

    assign led_d5  = ~counter[23];
    assign led_d6  = ~counter[22];
    assign led_j26 = ~counter[20];
endmodule
