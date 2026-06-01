// matmul_serial_io.v — load A then B over serial `sin` (512 bits, MSB-first:
// A[0..15] then B[0..15], each 16x16-bit row-major), then compute A*B (combinational
// gf16_matmul4x4) and serialise the 256-bit result on `dout` (sync pulse at frame
// start). `busy` high while loading. Sim-only for now (pin map TBD for synth).
`default_nettype none
module matmul_serial_io (input wire clk, input wire sin, output reg dout, output reg sync, output reg busy);
    reg [511:0] inbuf = 512'b0;
    reg [9:0]   lcnt  = 10'd0;     // 0..512 load counter
    reg         loaded = 1'b0;
    reg [255:0] outsh = 256'b0;
    reg [7:0]   ocnt  = 8'd0;
    wire [15:0] c00,c01,c02,c03,c10,c11,c12,c13,c20,c21,c22,c23,c30,c31,c32,c33;
    gf16_matmul4x4 u_mm (.a00(inbuf[511 -:16]),.a01(inbuf[495 -:16]),.a02(inbuf[479 -:16]),.a03(inbuf[463 -:16]),.a10(inbuf[447 -:16]),.a11(inbuf[431 -:16]),.a12(inbuf[415 -:16]),.a13(inbuf[399 -:16]),.a20(inbuf[383 -:16]),.a21(inbuf[367 -:16]),.a22(inbuf[351 -:16]),.a23(inbuf[335 -:16]),.a30(inbuf[319 -:16]),.a31(inbuf[303 -:16]),.a32(inbuf[287 -:16]),.a33(inbuf[271 -:16]),.b00(inbuf[255 -:16]),.b01(inbuf[239 -:16]),.b02(inbuf[223 -:16]),.b03(inbuf[207 -:16]),.b10(inbuf[191 -:16]),.b11(inbuf[175 -:16]),.b12(inbuf[159 -:16]),.b13(inbuf[143 -:16]),.b20(inbuf[127 -:16]),.b21(inbuf[111 -:16]),.b22(inbuf[95 -:16]),.b23(inbuf[79 -:16]),.b30(inbuf[63 -:16]),.b31(inbuf[47 -:16]),.b32(inbuf[31 -:16]),.b33(inbuf[15 -:16]),.c00(c00),.c01(c01),.c02(c02),.c03(c03),.c10(c10),.c11(c11),.c12(c12),.c13(c13),.c20(c20),.c21(c21),.c22(c22),.c23(c23),.c30(c30),.c31(c31),.c32(c32),.c33(c33));
    wire [255:0] result = {c00,c01,c02,c03,c10,c11,c12,c13,c20,c21,c22,c23,c30,c31,c32,c33};
    always @(posedge clk) begin
        if (!loaded) begin
            busy <= 1'b1; sync <= 1'b0; dout <= 1'b0;
            inbuf <= {inbuf[510:0], sin};
            if (lcnt == 10'd511) begin loaded <= 1'b1; lcnt <= 10'd0; end
            else lcnt <= lcnt + 10'd1;
        end else begin
            busy <= 1'b0;
            sync <= (ocnt == 8'd0);
            dout <= outsh[255];                            // emit MSB-first
            if (ocnt == 8'd255) outsh <= result;           // reload at frame end
            else outsh <= {outsh[254:0], 1'b0};
            ocnt <= (ocnt == 8'd255) ? 8'd0 : ocnt + 8'd1;
        end
    end
endmodule

