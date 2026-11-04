//! Cycle-Accurate Verilog HDL Simulation Model
//!
//! Replicates the gate-level adder tree logic of `opure_popcnt64.v` and `opure_ternary_dot.v`
//! bit-for-bit to allow hardware-software co-verification.

use opure_hal::TernaryCentroid64;

/// Simulates the 6-level Wallace adder tree in `opure_popcnt64.v`.
pub fn verilog_popcnt64_sim(val: u64) -> u32 {
    // Level 1: 32 pairs of 2-bit sums
    let mut l1 = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        let b0 = ((val >> (2 * i)) & 1) as u8;
        let b1 = ((val >> (2 * i + 1)) & 1) as u8;
        l1[i] = b0 + b1;
        i += 1;
    }

    // Level 2: 16 groups of 3-bit sums
    let mut l2 = [0u8; 16];
    let mut i = 0;
    while i < 16 {
        l2[i] = l1[2 * i] + l1[2 * i + 1];
        i += 1;
    }

    // Level 3: 8 groups of 4-bit sums
    let mut l3 = [0u8; 8];
    let mut i = 0;
    while i < 8 {
        l3[i] = l2[2 * i] + l2[2 * i + 1];
        i += 1;
    }

    // Level 4: 4 groups of 5-bit sums
    let mut l4 = [0u8; 4];
    let mut i = 0;
    while i < 4 {
        l4[i] = l3[2 * i] + l3[2 * i + 1];
        i += 1;
    }

    // Level 5: 2 groups of 6-bit sums
    let l5_0 = l4[0] + l4[1];
    let l5_1 = l4[2] + l4[3];

    // Level 6: Final sum
    (l5_0 + l5_1) as u32
}

/// Simulates the combinational arithmetic logic of `opure_ternary_dot.v`.
pub fn verilog_ternary_dot_sim(
    vec: &TernaryCentroid64,
    cent: &TernaryCentroid64,
) -> i32 {
    let pos_matches = (vec.pos_mask & cent.pos_mask) | (vec.neg_mask & cent.neg_mask);
    let neg_matches = (vec.pos_mask & cent.neg_mask) | (vec.neg_mask & cent.pos_mask);

    let pos_count = verilog_popcnt64_sim(pos_matches) as i32;
    let neg_count = verilog_popcnt64_sim(neg_matches) as i32;

    pos_count - neg_count
}
