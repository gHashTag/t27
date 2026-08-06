module ternary_mac_demo_top (
    output wire led_r23,
    output wire led_t23
);
    // Ring-oscillator clock source (matches gf16/blinky pattern for OpenXC7 user-pin designs).
    (* KEEP = "TRUE" *) wire osc;
    (* KEEP = "TRUE" *) wire chain [19:0];
    reg [22:0] counter = 0;

    assign chain[0] = ~chain[19];
    genvar i;
    generate
        for (i = 1; i < 20; i = i + 1) begin : inv_chain
            (* KEEP = "TRUE" *) LUT1 #(.INIT(2'b01)) inv (
                .I0(chain[i-1]),
                .O(chain[i])
            );
        end
    endgenerate
    assign osc = chain[19];

    // Drive the ternary MAC with trivial stimulus.
    wire        rst_n  = 1'b1;
    wire        en     = 1'b1;
    wire signed [7:0]  a = counter[7:0];
    wire        [1:0]  w_code = 2'b01;  // .plus weight (+1)
    wire signed [31:0] acc_in = 32'sd0;
    wire signed [31:0] acc_out;

    ternary_mac_top mac (
        .clk(osc),
        .rst_n(rst_n),
        .en(en),
        .a(a),
        .w_code(w_code),
        .acc_in(acc_in),
        .acc_out(acc_out)
    );

    always @(posedge osc) begin
        counter <= counter + 1;
    end

    // Visual feedback on the two red LEDs of the QMTech Wukong V1.
    assign led_r23 = ~acc_out[0];
    assign led_t23 = ~acc_out[1];
endmodule
