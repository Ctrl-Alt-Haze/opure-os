// ============================================================================
// OPURE Hardware Reference Design: 64D Multiplier-Less Ternary Dot Product
// Computes ternary inner product directly using bitwise logic & popcnt tree
// Latency: 1 Clock Cycle | Zero Floating-Point Multipliers
// ============================================================================

`timescale 1ns / 1ps

module opure_ternary_dot (
    input  wire [63:0] vec_pos,
    input  wire [63:0] vec_neg,
    input  wire [63:0] cent_pos,
    input  wire [63:0] cent_neg,
    output wire signed [8:0] dot_score
);

    // Bitwise product components:
    // +1 contribution: (+1 * +1) OR (-1 * -1)
    wire [63:0] pos_matches = (vec_pos & cent_pos) | (vec_neg & cent_neg);
    // -1 contribution: (+1 * -1) OR (-1 * +1)
    wire [63:0] neg_matches = (vec_pos & cent_neg) | (vec_neg & cent_pos);

    wire [6:0] pos_count;
    wire [6:0] neg_count;

    // Dual parallel popcount trees
    opure_popcnt64 u_pos_popcnt (
        .in_data(pos_matches),
        .out_count(pos_count)
    );

    opure_popcnt64 u_neg_popcnt (
        .in_data(neg_matches),
        .out_count(neg_count)
    );

    // Score = pos_count - neg_count (-64 to +64)
    assign dot_score = $signed({2'b00, pos_count}) - $signed({2'b00, neg_count});

endmodule
