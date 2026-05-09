`default_nettype none

module temporal_heartbeat_top (
    output wire led_d5,
    output wire led_d6,
    output wire led_j26
);

    wire cfgmclk;
    STARTUPE2 #(
        .PROG_USR("FALSE"),
        .SIM_CCLK_FREQ(10.0)
    ) startup (
        .CFGCLK(),
        .CFGMCLK(cfgmclk),
        .EOS(),
        .PREQ(),
        .CLK(1'b0),
        .GSR(1'b0),
        .GTS(1'b0),
        .KEYCLEARB(1'b0),
        .PACK(1'b0),
        .USRCCLKO(1'b0),
        .USRCCLKTS(1'b0),
        .USRDONEO(1'b1),
        .USRDONETS(1'b1)
    );

    localparam PHI_CYCLE = 27'd80_901_699;

    reg [26:0] phi_counter = 0;
    reg [1:0]  temporal_layer = 0;
    reg [24:0] blink_counter = 0;

    always @(posedge cfgmclk) begin
        blink_counter <= blink_counter + 1'b1;
        phi_counter <= phi_counter + 1'b1;
        if (phi_counter >= PHI_CYCLE) begin
            phi_counter <= 0;
            temporal_layer <= temporal_layer + 1'b1;
        end
    end

    reg d5_out, d6_out, j26_out;
    always @(*) begin
        d5_out  = 1'b0;
        d6_out  = 1'b0;
        j26_out = 1'b0;
        case (temporal_layer)
            2'd0: begin
                d5_out  = blink_counter[24];
                d6_out  = blink_counter[24];
                j26_out = blink_counter[22];
            end
            2'd1: begin
                d5_out  = 1'b1;
                d6_out  = 1'b1;
                j26_out = blink_counter[20];
            end
            2'd2: begin
                d5_out  = blink_counter[22];
                d6_out  = blink_counter[22];
                j26_out = blink_counter[20];
            end
            default: begin
                d5_out  = 1'b0;
                d6_out  = 1'b0;
                j26_out = 1'b0;
            end
        endcase
    end

    assign led_d5  = d5_out;
    assign led_d6  = d6_out;
    assign led_j26 = j26_out;

endmodule
