module gf16_uart_sim_top (
    input  wire clk,
    input  wire rst_n,
    input  wire uart_rx,
    output wire uart_tx,
    output wire led_r23,
    output wire led_t23
);

    localparam CLK_HZ = 323_310_000;
    localparam UART_BAUD = 115200;
    localparam CLK_DIV = CLK_HZ / (UART_BAUD * 16);

    wire [7:0] rx_data;
    wire rx_valid;
    wire tx_done;

    reg [7:0] tx_data;
    reg tx_start;

    reg [15:0] a00, a01, a02, a03;
    reg [15:0] a10, a11, a12, a13;
    reg [15:0] a20, a21, a22, a23;
    reg [15:0] a30, a31, a32, a33;
    reg [15:0] b00, b01, b02, b03;
    reg [15:0] b10, b11, b12, b13;
    reg [15:0] b20, b21, b22, b23;
    reg [15:0] b30, b31, b32, b33;

    wire [15:0] c00, c01, c02, c03;
    wire [15:0] c10, c11, c12, c13;
    wire [15:0] c20, c21, c22, c23;
    wire [15:0] c30, c31, c32, c33;

    gf16_matmul4x4 u_matmul (
        .a00(a00), .a01(a01), .a02(a02), .a03(a03),
        .a10(a10), .a11(a11), .a12(a12), .a13(a13),
        .a20(a20), .a21(a21), .a22(a22), .a23(a23),
        .a30(a30), .a31(a31), .a32(a32), .a33(a33),
        .b00(b00), .b01(b01), .b02(b02), .b03(b03),
        .b10(b10), .b11(b11), .b12(b12), .b13(b13),
        .b20(b20), .b21(b21), .b22(b22), .b23(b23),
        .b30(b30), .b31(b31), .b32(b32), .b33(b33),
        .c00(c00), .c01(c01), .c02(c02), .c03(c03),
        .c10(c10), .c11(c11), .c12(c12), .c13(c13),
        .c20(c20), .c21(c21), .c22(c22), .c23(c23),
        .c30(c30), .c31(c31), .c32(c32), .c33(c33)
    );

    uart_rx #(.CLK_DIV(CLK_DIV)) u_rx (
        .clk(clk),
        .rx(uart_rx),
        .valid(rx_valid),
        .data(rx_data)
    );

    uart_tx #(.CLK_DIV(CLK_DIV)) u_tx (
        .clk(clk),
        .data(tx_data),
        .start(tx_start),
        .tx(uart_tx),
        .done(tx_done)
    );

    reg [6:0] byte_cnt;
    reg [2:0] tx_state;
    reg [3:0] tx_word_cnt;

    always @(posedge clk) begin
        if (!rst_n) begin
            byte_cnt <= 0;
            tx_start <= 0;
            tx_state <= 0;
            tx_word_cnt <= 0;
        end else begin
            tx_start <= 0;

            if (rx_valid) begin
                case (byte_cnt)
                    0: a00[7:0] <= rx_data;
                    1: a00[15:8] <= rx_data;
                    2: a01[7:0] <= rx_data;
                    3: a01[15:8] <= rx_data;
                    4: a02[7:0] <= rx_data;
                    5: a02[15:8] <= rx_data;
                    6: a03[7:0] <= rx_data;
                    7: a03[15:8] <= rx_data;
                    8: a10[7:0] <= rx_data;
                    9: a10[15:8] <= rx_data;
                    10: a11[7:0] <= rx_data;
                    11: a11[15:8] <= rx_data;
                    12: a12[7:0] <= rx_data;
                    13: a12[15:8] <= rx_data;
                    14: a13[7:0] <= rx_data;
                    15: a13[15:8] <= rx_data;
                    16: a20[7:0] <= rx_data;
                    17: a20[15:8] <= rx_data;
                    18: a21[7:0] <= rx_data;
                    19: a21[15:8] <= rx_data;
                    20: a22[7:0] <= rx_data;
                    21: a22[15:8] <= rx_data;
                    22: a23[7:0] <= rx_data;
                    23: a23[15:8] <= rx_data;
                    24: a30[7:0] <= rx_data;
                    25: a30[15:8] <= rx_data;
                    26: a31[7:0] <= rx_data;
                    27: a31[15:8] <= rx_data;
                    28: a32[7:0] <= rx_data;
                    29: a32[15:8] <= rx_data;
                    30: a33[7:0] <= rx_data;
                    31: a33[15:8] <= rx_data;
                    32: b00[7:0] <= rx_data;
                    33: b00[15:8] <= rx_data;
                    34: b01[7:0] <= rx_data;
                    35: b01[15:8] <= rx_data;
                    36: b02[7:0] <= rx_data;
                    37: b02[15:8] <= rx_data;
                    38: b03[7:0] <= rx_data;
                    39: b03[15:8] <= rx_data;
                    40: b10[7:0] <= rx_data;
                    41: b10[15:8] <= rx_data;
                    42: b11[7:0] <= rx_data;
                    43: b11[15:8] <= rx_data;
                    44: b12[7:0] <= rx_data;
                    45: b12[15:8] <= rx_data;
                    46: b13[7:0] <= rx_data;
                    47: b13[15:8] <= rx_data;
                    48: b20[7:0] <= rx_data;
                    49: b20[15:8] <= rx_data;
                    50: b21[7:0] <= rx_data;
                    51: b21[15:8] <= rx_data;
                    52: b22[7:0] <= rx_data;
                    53: b22[15:8] <= rx_data;
                    54: b23[7:0] <= rx_data;
                    55: b23[15:8] <= rx_data;
                    56: b30[7:0] <= rx_data;
                    57: b30[15:8] <= rx_data;
                    58: b31[7:0] <= rx_data;
                    59: b31[15:8] <= rx_data;
                    60: b32[7:0] <= rx_data;
                    61: b32[15:8] <= rx_data;
                    62: b33[7:0] <= rx_data;
                    63: b33[15:8] <= rx_data;
                    default: ;
                endcase
                byte_cnt <= byte_cnt + 1;
                if (byte_cnt == 63) begin
                    tx_state <= 1;
                    tx_word_cnt <= 0;
                    byte_cnt <= 0;
                end
            end

            case (tx_state)
                1: begin
                    if (tx_done || tx_word_cnt == 0) begin
                        case (tx_word_cnt)
                            0: tx_data <= c00[7:0];
                            1: tx_data <= c00[15:8];
                            2: tx_data <= c01[7:0];
                            3: tx_data <= c01[15:8];
                            4: tx_data <= c02[7:0];
                            5: tx_data <= c02[15:8];
                            6: tx_data <= c03[7:0];
                            7: tx_data <= c03[15:8];
                            8: tx_data <= c10[7:0];
                            9: tx_data <= c10[15:8];
                            10: tx_data <= c11[7:0];
                            11: tx_data <= c11[15:8];
                            12: tx_data <= c12[7:0];
                            13: tx_data <= c12[15:8];
                            14: tx_data <= c13[7:0];
                            15: tx_data <= c13[15:8];
                            16: tx_data <= c20[7:0];
                            17: tx_data <= c20[15:8];
                            18: tx_data <= c21[7:0];
                            19: tx_data <= c21[15:8];
                            20: tx_data <= c22[7:0];
                            21: tx_data <= c22[15:8];
                            22: tx_data <= c23[7:0];
                            23: tx_data <= c23[15:8];
                            24: tx_data <= c30[7:0];
                            25: tx_data <= c30[15:8];
                            26: tx_data <= c31[7:0];
                            27: tx_data <= c31[15:8];
                            28: tx_data <= c32[7:0];
                            29: tx_data <= c32[15:8];
                            30: tx_data <= c33[7:0];
                            31: tx_data <= c33[15:8];
                            default: ;
                        endcase
                        tx_start <= 1;
                        tx_state <= 2;
                    end
                end
                2: begin
                    if (tx_done) begin
                        tx_word_cnt <= tx_word_cnt + 1;
                        if (tx_word_cnt == 31) begin
                            tx_state <= 0;
                            tx_word_cnt <= 0;
                        end else begin
                            tx_state <= 1;
                        end
                    end
                end
                default: tx_state <= 0;
            endcase
        end
    end

    assign led_r23 = ~rst_n;
    assign led_t23 = tx_state != 0;

endmodule
