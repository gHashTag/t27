`default_nettype none

module gf16_heartbeat_top (
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

    localparam PHI_CYCLE       = 27'd80_901_699;
    localparam GF16_ONE        = 16'h3E00;
    localparam GF16_PHI_FRAC   = 16'h3D9E;
    localparam GF16_HALF       = 16'h3C00;
    localparam GF16_QUARTER    = 16'h3A00;

    reg [26:0] phi_counter = 0;
    reg [1:0]  temporal_layer = 0;
    reg [24:0] blink_counter = 0;

    always @(posedge cfgmclk) begin
        blink_counter <= blink_counter + 1'b1;
        phi_counter   <= phi_counter + 1'b1;
        if (phi_counter >= PHI_CYCLE) begin
            phi_counter   <= 0;
            temporal_layer <= temporal_layer + 1'b1;
        end
    end

    reg [15:0] vec_a0, vec_a1, vec_a2, vec_a3;
    reg [15:0] vec_b0, vec_b1, vec_b2, vec_b3;

    always @(posedge cfgmclk) begin
        vec_a0 <= GF16_ONE;
        vec_a1 <= GF16_PHI_FRAC;
        vec_a2 <= GF16_HALF;
        vec_a3 <= GF16_QUARTER;
        case (temporal_layer)
            2'd0: begin
                vec_b0 <= GF16_ONE;
                vec_b1 <= GF16_ONE;
                vec_b2 <= GF16_ONE;
                vec_b3 <= GF16_ONE;
            end
            2'd1: begin
                vec_b0 <= GF16_PHI_FRAC;
                vec_b1 <= GF16_PHI_FRAC;
                vec_b2 <= GF16_PHI_FRAC;
                vec_b3 <= GF16_PHI_FRAC;
            end
            2'd2: begin
                vec_b0 <= GF16_HALF;
                vec_b1 <= GF16_HALF;
                vec_b2 <= GF16_HALF;
                vec_b3 <= GF16_HALF;
            end
            default: begin
                vec_b0 <= GF16_QUARTER;
                vec_b1 <= GF16_QUARTER;
                vec_b2 <= GF16_QUARTER;
                vec_b3 <= GF16_QUARTER;
            end
        endcase
    end

    wire [15:0] dot4_result;

    gf16_dot4 dot4 (
        .a0(vec_a0), .a1(vec_a1), .a2(vec_a2), .a3(vec_a3),
        .b0(vec_b0), .b1(vec_b1), .b2(vec_b2), .b3(vec_b3),
        .result(dot4_result)
    );

    wire dot4_is_positive = ~dot4_result[15];
    wire [5:0] dot4_exp = dot4_result[14:9];
    wire dot4_is_nonzero = (dot4_exp != 6'd0);

    reg d5_out, d6_out, j26_out;
    always @(*) begin
        d5_out  = 1'b0;
        d6_out  = 1'b0;
        j26_out = 1'b0;
        case (temporal_layer)
            2'd0: begin
                d5_out  = blink_counter[24];
                d6_out  = blink_counter[24];
                j26_out = dot4_is_nonzero ? blink_counter[20] : 1'b0;
            end
            2'd1: begin
                d5_out  = 1'b1;
                d6_out  = 1'b1;
                j26_out = dot4_is_positive ? blink_counter[19] : 1'b0;
            end
            2'd2: begin
                d5_out  = blink_counter[22];
                d6_out  = blink_counter[22];
                j26_out = dot4_is_positive ? blink_counter[18] : 1'b0;
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
