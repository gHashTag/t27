// No external reset: relies on FPGA config-time register init (sh=0,bitpos=0).
// Frame 1 emits zeros; frame 2+ emit the real 256-bit result. clk = E3 12 MHz.
`default_nettype none
module matmul_serial_hw (input wire clk, output reg led_data, output reg led_sync);
    wire [15:0] c [0:15];
    gf16_matmul4x4 u_mm (
        .a00(16'h3E00),.a01(16'h4000),.a02(16'h4100),.a03(16'h4200),
        .a10(16'h4300),.a11(16'h4380),.a12(16'h4400),.a13(16'h4440),
        .a20(16'h4480),.a21(16'h44C0),.a22(16'h4500),.a23(16'h4520),
        .a30(16'h4540),.a31(16'h4560),.a32(16'h4580),.a33(16'h45A0),
        .b00(16'h3E00),.b01(16'h0000),.b02(16'h0000),.b03(16'h0000),
        .b10(16'h0000),.b11(16'h3E00),.b12(16'h0000),.b13(16'h0000),
        .b20(16'h0000),.b21(16'h0000),.b22(16'h3E00),.b23(16'h0000),
        .b30(16'h0000),.b31(16'h0000),.b32(16'h0000),.b33(16'h3E00),
        .c00(c[0]),.c01(c[1]),.c02(c[2]),.c03(c[3]),.c10(c[4]),.c11(c[5]),.c12(c[6]),.c13(c[7]),
        .c20(c[8]),.c21(c[9]),.c22(c[10]),.c23(c[11]),.c30(c[12]),.c31(c[13]),.c32(c[14]),.c33(c[15])
    );
    wire [255:0] result = {c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],c[10],c[11],c[12],c[13],c[14],c[15]};
    reg [255:0] sh = 256'b0;
    reg [7:0]   bitpos = 8'b0;
    always @(posedge clk) begin
        led_sync <= (bitpos==8'd0);
        led_data <= sh[255];
        if (bitpos==8'd255) begin sh<=result; bitpos<=8'd0; end
        else begin sh<={sh[254:0],1'b0}; bitpos<=bitpos+8'd1; end
    end
endmodule
