// ============================================================================
// OPURE Hardware Reference Design: Self-Checking Testbench for 64-Bit Popcount
// Verifies 6-level Wallace adder tree against software assertions
// ============================================================================

`timescale 1ns / 1ps

module opure_popcnt64_tb;

    reg  [63:0] in_data;
    wire [6:0]  out_count;
    integer     errors;

    // Unit Under Test (UUT)
    opure_popcnt64 uut (
        .in_data(in_data),
        .out_count(out_count)
    );

    initial begin
        // VCD Waveform Dump
        $dumpfile("popcnt64_waveform.vcd");
        $dumpvars(0, opure_popcnt64_tb);

        errors = 0;
        #10;

        // Test 1: All zeros
        in_data = 64'h0000_0000_0000_0000;
        #5;
        if (out_count !== 7'd0) begin
            $display("[FAIL] All zeros: expected 0, got %0d", out_count);
            errors = errors + 1;
        end else begin
            $display("[PASS] All zeros: count = %0d", out_count);
        end

        // Test 2: All ones
        in_data = 64'hFFFF_FFFF_FFFF_FFFF;
        #5;
        if (out_count !== 7'd64) begin
            $display("[FAIL] All ones: expected 64, got %0d", out_count);
            errors = errors + 1;
        end else begin
            $display("[PASS] All ones: count = %0d", out_count);
        end

        // Test 3: Alternating 0x5555... (32 ones)
        in_data = 64'h5555_5555_5555_5555;
        #5;
        if (out_count !== 7'd32) begin
            $display("[FAIL] Alternating 0x5555: expected 32, got %0d", out_count);
            errors = errors + 1;
        end else begin
            $display("[PASS] Alternating 0x5555: count = %0d", out_count);
        end

        // Test 4: Single bit sliding
        in_data = 64'h0000_0000_0000_0001;
        #5;
        if (out_count !== 7'd1) begin
            $display("[FAIL] Single bit: expected 1, got %0d", out_count);
            errors = errors + 1;
        end else begin
            $display("[PASS] Single bit: count = %0d", out_count);
        end

        if (errors == 0) begin
            $display(">>> ALL POPCNT64 HARDWARE TESTS PASSED <<<");
        end else begin
            $display(">>> %0d POPCNT64 TESTS FAILED <<<", errors);
        end

        $finish;
    end

endmodule
