// ============================================================================
// OPURE Hardware Reference Design: AXI4-Lite Slave MMIO Interface
// Connects the OPURE NCM hardware accelerator to host SoC processor (Zynq / RISC-V)
// ============================================================================

`timescale 1ns / 1ps

module opure_axi_lite_slave (
    input  wire        s_axi_aclk,
    input  wire        s_axi_aresetn,
    // Write Address Channel
    input  wire [5:0]  s_axi_awaddr,
    input  wire        s_axi_awvalid,
    output reg         s_axi_awready,
    // Write Data Channel
    input  wire [31:0] s_axi_wdata,
    input  wire [3:0]  s_axi_wstrb,
    input  wire        s_axi_wvalid,
    output reg         s_axi_wready,
    // Write Response Channel
    output reg  [1:0]  s_axi_bresp,
    output reg         s_axi_bvalid,
    input  wire        s_axi_bready,
    // Read Address Channel
    input  wire [5:0]  s_axi_araddr,
    input  wire        s_axi_arvalid,
    output reg         s_axi_arready,
    // Read Data Channel
    output reg  [31:0] s_axi_rdata,
    output reg  [1:0]  s_axi_rresp,
    output reg         s_axi_rvalid,
    input  wire        s_axi_rready
);

    // Register Offsets
    localparam ADDR_CONTROL       = 6'h00;
    localparam ADDR_STATUS        = 6'h04;
    localparam ADDR_VEC_POS_LO    = 6'h08;
    localparam ADDR_VEC_POS_HI    = 6'h0C;
    localparam ADDR_VEC_NEG_LO    = 6'h10;
    localparam ADDR_VEC_NEG_HI    = 6'h14;
    localparam ADDR_RESULT_SCORE  = 6'h18;
    localparam ADDR_RESULT_INTENT = 6'h1C;

    // Internal Registers
    reg [31:0] reg_control;
    reg [31:0] reg_status;
    reg [31:0] reg_pos_lo;
    reg [31:0] reg_pos_hi;
    reg [31:0] reg_neg_lo;
    reg [31:0] reg_neg_hi;
    reg [31:0] reg_result_score;
    reg [31:0] reg_result_intent;

    // AXI4-Lite Handshake Logic
    always @(posedge s_axi_aclk or negedge s_axi_aresetn) begin
        if (!s_axi_aresetn) begin
            s_axi_awready <= 1'b0;
            s_axi_wready  <= 1'b0;
            s_axi_bvalid  <= 1'b0;
            s_axi_bresp   <= 2'b00;
            reg_control   <= 32'd0;
            reg_status    <= 32'd0;
            reg_pos_lo    <= 32'd0;
            reg_pos_hi    <= 32'd0;
            reg_neg_lo    <= 32'd0;
            reg_neg_hi    <= 32'd0;
        end else begin
            // Write Address & Data
            if (s_axi_awvalid && s_axi_wvalid && !s_axi_awready) begin
                s_axi_awready <= 1'b1;
                s_axi_wready  <= 1'b1;
                s_axi_bvalid  <= 1'b1;
                s_axi_bresp   <= 2'b00; // OKAY

                case (s_axi_awaddr)
                    ADDR_CONTROL:    reg_control <= s_axi_wdata;
                    ADDR_VEC_POS_LO: reg_pos_lo  <= s_axi_wdata;
                    ADDR_VEC_POS_HI: reg_pos_hi  <= s_axi_wdata;
                    ADDR_VEC_NEG_LO: reg_neg_lo  <= s_axi_wdata;
                    ADDR_VEC_NEG_HI: reg_neg_hi  <= s_axi_wdata;
                    default: ;
                endcase
            end else begin
                s_axi_awready <= 1'b0;
                s_axi_wready  <= 1'b0;
                if (s_axi_bvalid && s_axi_bready) begin
                    s_axi_bvalid <= 1'b0;
                end
            end
        end
    end

    // Read Handshake Logic
    always @(posedge s_axi_aclk or negedge s_axi_aresetn) begin
        if (!s_axi_aresetn) begin
            s_axi_arready <= 1'b0;
            s_axi_rvalid  <= 1'b0;
            s_axi_rresp   <= 2'b00;
            s_axi_rdata   <= 32'd0;
        end else begin
            if (s_axi_arvalid && !s_axi_arready) begin
                s_axi_arready <= 1'b1;
                s_axi_rvalid  <= 1'b1;
                s_axi_rresp   <= 2'b00; // OKAY
                case (s_axi_araddr)
                    ADDR_CONTROL:       s_axi_rdata <= reg_control;
                    ADDR_STATUS:        s_axi_rdata <= reg_status;
                    ADDR_VEC_POS_LO:    s_axi_rdata <= reg_pos_lo;
                    ADDR_VEC_POS_HI:    s_axi_rdata <= reg_pos_hi;
                    ADDR_VEC_NEG_LO:    s_axi_rdata <= reg_neg_lo;
                    ADDR_VEC_NEG_HI:    s_axi_rdata <= reg_neg_hi;
                    ADDR_RESULT_SCORE:  s_axi_rdata <= reg_result_score;
                    ADDR_RESULT_INTENT: s_axi_rdata <= reg_result_intent;
                    default:            s_axi_rdata <= 32'hDEADBEEF;
                endcase
            end else begin
                s_axi_arready <= 1'b0;
                if (s_axi_rvalid && s_axi_rready) begin
                    s_axi_rvalid <= 1'b0;
                end
            end
        end
    end

endmodule
