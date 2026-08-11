// End-to-end DATA check for bitnet_engine_top.
//
// Every property in the campaign constrains CONTROL. Prop. 81b named that
// boundary and Prop. 121 demonstrated it at scale: 28 integration properties
// proved while the machine computed the wrong answer and deadlocked. Nothing has
// ever compared an engine OUTPUT against a reference.
//
// This drives the engine through its CSR aperture with KNOWN weights and a KNOWN
// input vector, computes the expected layer-0 activation word in the testbench,
// and compares. The two memory ports are separate, which is what makes it
// possible: the DMA reads input activations over m_axi_*, the prefetcher reads
// weights over mem_rd_*.
//
// Encoding: 2'b00 = -1, 2'b01 = 0, 2'b10 = +1.
`timescale 1ns/1ps

module tb_data;
    localparam integer C = 1;          // chunks per neuron
    localparam integer N = 1;          // neurons
    localparam integer THRESH = 3;     // requant threshold

    reg clk = 0, rst_n = 0;
    always #5 clk = ~clk;

    // ---- the known input vector: 27x(+1) in one 54-bit word ---------------
    // dot with all-(+1) weights = 27*(+1) = +27, requant -> TRIT_P
    function [53:0] input_word;
        input dummy;
        integer i;
        begin
            input_word = 54'd0;
            for (i = 0; i < 27; i = i + 1)
                // Wave 662: all (+1). The previous vector was chosen so the
                // reference accumulator would be 0 -- a value wrong under most
                // indexing errors, and ALSO the value an uninitialised counter
                // reads. It could not tell a working engine from a silent
                // harness. 27 and TRIT_P can be produced by neither.
                input_word[i*2 +: 2] = 2'b10;
        end
    endfunction

    // ---- the known weights: all (+1) ---------------------------------------
    function [53:0] weight_word;
        input dummy;
        integer i;
        begin
            weight_word = 54'd0;
            for (i = 0; i < 27; i = i + 1) weight_word[i*2 +: 2] = 2'b10;
        end
    endfunction

    // ---- the reference: what layer 0 must produce --------------------------
    // acc = sum over all C*27 lanes of input_trit * weight_trit
    function signed [15:0] ref_acc;
        input dummy;
        integer i, c;
        reg signed [15:0] a;
        reg [53:0] iw, ww;
        reg signed [1:0] it, wt;
        begin
            a = 0;
            iw = input_word(0);
            ww = weight_word(0);
            for (c = 0; c < C; c = c + 1)
                for (i = 0; i < 27; i = i + 1) begin
                    it = (iw[i*2 +: 2] == 2'b00) ? -1 :
                         (iw[i*2 +: 2] == 2'b10) ?  1 : 0;
                    wt = (ww[i*2 +: 2] == 2'b00) ? -1 :
                         (ww[i*2 +: 2] == 2'b10) ?  1 : 0;
                    a = a + it * wt;
                end
            ref_acc = a;
        end
    endfunction

    // requant: +1 above threshold, -1 below -threshold, else 0
    function [1:0] ref_trit;
        input signed [15:0] a;
        begin
            ref_trit = (a >  THRESH) ? 2'b10 :
                       (a < -THRESH) ? 2'b00 : 2'b01;
        end
    endfunction

    // ---- DUT wiring --------------------------------------------------------
    reg  [7:0]  aw; reg [31:0] wd; reg awv, wv; wire awr, wr_rdy;
    wire [1:0]  bresp; wire bvalid; reg br = 1'b1;
    reg  [7:0]  ar; reg arv; wire arr;
    wire [31:0] rdata; wire [1:0] rresp; wire rvalid; reg rr;
    wire [63:0] m_araddr; wire [7:0] m_arlen; wire m_arvalid; reg m_arready;
    reg  [63:0] m_rdata; reg m_rvalid, m_rlast; wire m_rready;
    wire [31:0] mem_addr; wire mem_rd_en; reg [63:0] mem_rd_data; reg mem_rd_valid;
    wire busy, done_o; wire [31:0] eng_cyc;
    wire [15:0] neuron_out; wire neuron_out_valid;
    wire [53:0] actw; wire actw_v; wire irq;

    bitnet_engine_top dut (
        .clk(clk), .rst_n(rst_n),
        .s_axi_awaddr(aw), .s_axi_awvalid(awv), .s_axi_awready(awr),
        .s_axi_wdata(wd), .s_axi_wstrb(4'hF), .s_axi_wvalid(wv),
        .s_axi_wready(wr_rdy),
        .s_axi_bresp(bresp), .s_axi_bvalid(bvalid), .s_axi_bready(br),
        .s_axi_araddr(ar), .s_axi_arvalid(arv), .s_axi_arready(arr),
        .s_axi_rdata(rdata), .s_axi_rresp(rresp), .s_axi_rvalid(rvalid),
        .s_axi_rready(rr),
        .m_axi_araddr(m_araddr), .m_axi_arlen(m_arlen),
        .m_axi_arvalid(m_arvalid), .m_axi_arready(m_arready),
        .m_axi_rdata(m_rdata), .m_axi_rlast(m_rlast),
        .m_axi_rvalid(m_rvalid), .m_axi_rready(m_rready),
        .mem_addr(mem_addr), .mem_rd_en(mem_rd_en),
        .mem_rd_data(mem_rd_data), .mem_rd_valid(mem_rd_valid),
        .busy(busy), .done(done_o), .cycle_count(eng_cyc),
        .neuron_out(neuron_out), .neuron_out_valid(neuron_out_valid),
        .act_word_out(actw), .act_word_out_valid(actw_v),
        .irq(irq)
    );

    // ---- AXI4 read slave: serves the INPUT ACTIVATION vector ---------------
    reg [8:0] beats_left; reg burst_active;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            burst_active <= 0; beats_left <= 0;
            m_rvalid <= 0; m_rlast <= 0; m_rdata <= 64'd0; m_arready <= 1;
        end else if (!burst_active) begin
            m_rvalid <= 0; m_rlast <= 0; m_arready <= 1;
            if (m_arvalid && m_arready) begin
                burst_active <= 1; m_arready <= 0;
                beats_left <= {1'b0, m_arlen} + 9'd1;
                m_rvalid <= 1; m_rlast <= (m_arlen == 8'd0);
                m_rdata  <= {10'd0, input_word(0)};
            end
        end else if (m_rready) begin
            beats_left <= beats_left - 9'd1;
            if (beats_left == 9'd1) begin
                burst_active <= 0; m_rvalid <= 0; m_rlast <= 0; m_arready <= 1;
            end else begin
                m_rvalid <= 1; m_rlast <= (beats_left == 9'd2);
                m_rdata  <= {10'd0, input_word(0)};
            end
        end
    end

    // ---- weight memory: serves the WEIGHTS ---------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin mem_rd_valid <= 0; mem_rd_data <= 64'd0; end
        else begin
            mem_rd_valid <= mem_rd_en;
            mem_rd_data  <= {10'd0, weight_word(0)};
        end
    end

    task csr_write(input [7:0] a_, input [31:0] d_);
        begin
            @(posedge clk); aw <= a_; wd <= d_; awv <= 1; wv <= 1;
            @(posedge clk); while (!(awr && wr_rdy)) @(posedge clk);
            awv <= 0; wv <= 0; @(posedge clk);
        end
    endtask

    integer cyc; integer seen; integer pf_wait; reg [53:0] got;
    reg signed [15:0] acc_seen;
    // Wave 662: a companion flag. Comparing an assigned-under-a-condition
    // variable against a reference is not a measurement unless the
    // condition fired, and the previous version reported its initial
    // value as an agreement.
    reg saw_mac;

    // capture the first emitted activation word and the MAC result
    always @(posedge clk) if (rst_n) begin
        if (actw_v && seen == 0) begin
            got  = actw;
            seen = 1;
        end
        if (dut.mac_valid_q) begin acc_seen = dut.mac_result; saw_mac = 1'b1; end
    end


    // ---- weight-path probe --------------------------------------------------
    integer wr_beats = 0; integer mac_beats = 0;
    integer sp_n = 0; integer rden_n = 0; integer rdv_n = 0; integer pfd_n = 0;
    always @(posedge clk) if (rst_n) begin
        if (dut.prefetch.bram_we) wr_beats = wr_beats + 1;
        if (dut.start_prefetch) sp_n = sp_n + 1;
        if (dut.mem_rd_en)      rden_n = rden_n + 1;
        if (dut.mem_rd_valid)   rdv_n = rdv_n + 1;
        if (dut.prefetch_done)  pfd_n = pfd_n + 1;
        if (dut.mac_valid_q)      mac_beats = mac_beats + 1;
    end

    initial begin
        aw=0; wd=0; awv=0; wv=0; ar=0; arv=0; rr=1;
        seen=0; got=54'd0; acc_seen=0; saw_mac=0; cyc=0;
        repeat (8) @(posedge clk); rst_n = 1; repeat (4) @(posedge clk);

        csr_write(8'h10, 32'd1);          // num_layers
        csr_write(8'h14, N);              // neurons
        // Wave 663: weight_words is NEURONS x CHUNKS, not a constant. The
        // sweep harness of Prop. 125 computes W = N*C and reaches the
        // prefetcher; this one hardcoded 64 and never saw a single BRAM
        // write. Also enable the IRQs the sweep enables.
        csr_write(8'h18, ((N*C) << 16) | C);        // weight_words, chunks
        csr_write(8'h08, 32'd7);                    // irq enable
        csr_write(8'h1C, THRESH);         // threshold
        csr_write(8'h00, 32'h2);          // start DMA
        @(posedge clk); csr_write(8'h00, 32'h0);
        // Wait for the weight PREFETCH to finish, not a fixed delay.
        // Wave 661: 200 cycles was not enough, so the MAC read an
        // unwritten weight BRAM and the accumulator arrived at the
        // requantizer as X.
        // Wave 661: the weight prefetch is triggered BY the inference start,
        // so waiting for prefetch_done before starting is a deadlock of my own
        // making. Start inference, then give the prefetch room to run.
        repeat (400) @(posedge clk);
        csr_write(8'h00, 32'h1);          // start inference
        @(posedge clk); csr_write(8'h00, 32'h0);

        repeat (2000) @(posedge clk);
        while (cyc < 20000 && seen == 0) begin @(posedge clk); cyc = cyc + 1; end

        $display("REFERENCE  acc=%0d  trit=%b", ref_acc(0), ref_trit(ref_acc(0)));
        $display("ENGINE     acc=%0d  trit=%b  (word=%h, emitted=%0d)",
                 acc_seen, got[1:0], got, seen);
        if (!saw_mac)
            $display("RESULT: THE MAC NEVER PRODUCED A RESULT -- nothing was measured");
        else if (seen == 0)
            $display("RESULT: NO ACTIVATION WORD EMITTED");
        else if (acc_seen !== ref_acc(0))
            $display("RESULT: MAC MISMATCH  engine=%0d reference=%0d",
                     acc_seen, ref_acc(0));
        else if (got[1:0] !== ref_trit(ref_acc(0)))
            $display("RESULT: TRIT MISMATCH engine=%b reference=%b",
                     got[1:0], ref_trit(ref_acc(0)));
        else
            $display("RESULT: MATCH");
        $display("PROBE bram_we=%0d start_prefetch=%0d mem_rd_en=%0d mem_rd_valid=%0d prefetch_done=%0d mac=%0d",
                 wr_beats, sp_n, rden_n, rdv_n, pfd_n, mac_beats);
        $display("PROBE bram_we=%0d start_prefetch=%0d mem_rd_en=%0d mem_rd_valid=%0d prefetch_done=%0d mac=%0d",
                 wr_beats, sp_n, rden_n, rdv_n, pfd_n, mac_beats);
        $finish;
    end
endmodule
