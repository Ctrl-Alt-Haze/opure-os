// ============================================================================
// OPURE Hardware Reference Design: 64-Bit Combinational Population Counter
// Target Architecture: Xilinx/AMD Zynq-7000 / UltraScale+ / Lattice ECP5
// Synthesizable Verilog-2001 | Single-Cycle Latency | 0 DSP Blocks
// ============================================================================

`timescale 1ns / 1ps

module opure_popcnt64 (
    input  wire [63:0] in_data,
    output wire [6:0]  out_count
);

    // Level 1: 32 pairs of 2-bit sums
    wire [1:0] l1_sum [31:0];
    genvar i;
    generate
        for (i = 0; i < 32; i = i + 1) begin : gen_l1
            assign l1_sum[i] = in_data[2*i] + in_data[2*i + 1];
        end
    endgenerate

    // Level 2: 16 groups of 3-bit sums
    wire [2:0] l2_sum [15:0];
    generate
        for (i = 0; i < 16; i = i + 1) begin : gen_l2
            assign l2_sum[i] = l1_sum[2*i] + l1_sum[2*i + 1];
        end
    endgenerate

    // Level 3: 8 groups of 4-bit sums
    wire [3:0] l3_sum [7:0];
    generate
        for (i = 0; i < 8; i = i + 1) begin : gen_l3
            assign l3_sum[i] = l2_sum[2*i] + l2_sum[2*i + 1];
        end
    endgenerate

    // Level 4: 4 groups of 5-bit sums
    wire [4:0] l4_sum [3:0];
    generate
        for (i = 0; i < 4; i = i + 1) begin : gen_l4
            assign l4_sum[i] = l3_sum[2*i] + l3_sum[2*i + 1];
        end
    endgenerate

    // Level 5: 2 groups of 6-bit sums
    wire [5:0] l5_sum [1:0];
    assign l5_sum[0] = l4_sum[0] + l4_sum[1];
    assign l5_sum[1] = l4_sum[2] + l4_sum[3];

    // Level 6: Final 7-bit population count (0 to 64)
    assign out_count = l5_sum[0] + l5_sum[1];

endmodule
