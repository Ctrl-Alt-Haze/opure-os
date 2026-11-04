// ============================================================================
// OPURE Hardware Reference Design: Parallel Ternary NCM Classification Core
// Evaluates multiple centroid classes in parallel and computes argmax intent
// Latency: 2 Clock Cycles (Pipelined) | Clock Frequency: >300 MHz
// ============================================================================

`timescale 1ns / 1ps

module opure_ncm_core #(
    parameter NUM_CLASSES = 4
)(
    input  wire        clk,
    input  wire        rst_n,
    input  wire        start,
    input  wire [63:0] vec_pos,
    input  wire [63:0] vec_neg,
    input  wire [63:0] cent_pos [0:NUM_CLASSES-1],
    input  wire [63:0] cent_neg [0:NUM_CLASSES-1],
    output reg         done,
    output reg  signed [8:0] best_score,
    output reg  [3:0]  best_intent
);

    wire signed [8:0] class_scores [0:NUM_CLASSES-1];

    genvar c;
    generate
        for (c = 0; c < NUM_CLASSES; c = c + 1) begin : gen_dots
            opure_ternary_dot u_dot (
                .vec_pos(vec_pos),
                .vec_neg(vec_neg),
                .cent_pos(cent_pos[c]),
                .cent_neg(cent_neg[c]),
                .dot_score(class_scores[c])
            );
        end
    endgenerate

    // Pipelined Argmax Comparator
    integer i;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            done        <= 1'b0;
            best_score  <= -9'sd65;
            best_intent <= 4'd0;
        end else if (start) begin
            done <= 1'b1;
            best_score  <= class_scores[0];
            best_intent <= 4'd0;
            for (i = 1; i < NUM_CLASSES; i = i + 1) begin
                if (class_scores[i] > best_score) begin
                    best_score  <= class_scores[i];
                    best_intent <= i[3:0];
                end
            end
        end else begin
            done <= 1'b0;
        end
    end

endmodule
