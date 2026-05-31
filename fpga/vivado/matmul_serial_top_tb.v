module mst_tb;
  reg clk=0,rst_n; wire led_data,led_sync;
  matmul_serial_top uut(.clk(clk),.rst_n(rst_n),.led_data(led_data),.led_sync(led_sync));
  always #5 clk=~clk;
  reg [255:0] cap; integer i,k,fails; reg [255:0] expd;
  // expected = A x I = A, packed as {c00..c33}. A given in top. A*I=A so c=A row-major.
  initial begin
    fails=0; rst_n=0; repeat(2)@(posedge clk); rst_n=1;
    // wait for a sync pulse (frame start), then capture 256 bits
    @(posedge clk); while(led_sync!==1'b1) @(posedge clk);
    // led_data this cycle = sh[255] (bit0 of frame). Capture 256.
    cap=0; for(i=0;i<256;i=i+1) begin cap={cap[254:0],led_data}; @(posedge clk); end
    // expected: A matrix (since A*I=A). build from the a-constants:
    expd={16'h3E00,16'h4000,16'h4100,16'h4200, 16'h4300,16'h4380,16'h4400,16'h4440,
          16'h4480,16'h44C0,16'h4500,16'h4520, 16'h4540,16'h4560,16'h4580,16'h45A0};
    if(cap!==expd) begin fails=fails+1; $display("MISMATCH\n cap =%h\n exp =%h",cap,expd); end
    $display("MATMUL_SERIAL: %0d fail -> %s", fails, (fails==0)?"CLEAN (256-bit serial readback == A*I result)":"BUGS");
    $finish;
  end
endmodule
