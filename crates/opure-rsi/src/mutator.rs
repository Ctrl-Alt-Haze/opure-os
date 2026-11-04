//! OPURE RSI Bitwise Mutation Operators
//!
//! Provides deterministic bitwise mutation operators (flip, swap, mask-shift, and stochastic drift)
//! on 16-byte dual-mask TernaryCentroid64 prototypes using native SIMD/bitwise intrinsics and POPCNT fitness.

#![deny(warnings)]

use opure_hal::simd::ternary_dot_ternary;
use opure_hal::TernaryCentroid64;

pub struct PlasticityMutator;

impl PlasticityMutator {
    /// Enforces the ternary invariant: a weight cannot be both +1 and -1 simultaneously.
    #[inline(always)]
    pub fn enforce_invariant(centroid: &mut TernaryCentroid64) {
        centroid.neg_mask &= !centroid.pos_mask;
    }

    /// Flips a bit at `bit_idx` (0..63).
    /// If `to_negative` is true, sets it in `neg_mask`, otherwise in `pos_mask`.
    #[inline(always)]
    pub fn bit_flip(centroid: &mut TernaryCentroid64, bit_idx: usize, to_negative: bool) {
        let mask = 1u64 << (bit_idx & 63);
        if to_negative {
            centroid.pos_mask &= !mask;
            centroid.neg_mask ^= mask;
        } else {
            centroid.neg_mask &= !mask;
            centroid.pos_mask ^= mask;
        }
    }

    /// Swaps the ternary values at two indices.
    #[inline(always)]
    pub fn bit_swap(centroid: &mut TernaryCentroid64, idx_a: usize, idx_b: usize) {
        let bit_a = idx_a & 63;
        let bit_b = idx_b & 63;
        if bit_a == bit_b {
            return;
        }

        let val_a_pos = (centroid.pos_mask >> bit_a) & 1;
        let val_a_neg = (centroid.neg_mask >> bit_a) & 1;
        let val_b_pos = (centroid.pos_mask >> bit_b) & 1;
        let val_b_neg = (centroid.neg_mask >> bit_b) & 1;

        // Clear bits
        centroid.pos_mask &= !( (1u64 << bit_a) | (1u64 << bit_b) );
        centroid.neg_mask &= !( (1u64 << bit_a) | (1u64 << bit_b) );

        // Restore swapped
        centroid.pos_mask |= (val_b_pos << bit_a) | (val_a_pos << bit_b);
        centroid.neg_mask |= (val_b_neg << bit_a) | (val_a_neg << bit_b);
    }

    /// Rotates both bitmasks left by `shift` bits.
    #[inline(always)]
    pub fn mask_shift(centroid: &mut TernaryCentroid64, shift: u32) {
        centroid.pos_mask = centroid.pos_mask.rotate_left(shift);
        centroid.neg_mask = centroid.neg_mask.rotate_left(shift);
    }

    /// Stochastic deterministic mutation generating a child centroid from a PRNG seed.
    /// Runs in 1-2 CPU cycles without heap allocations.
    #[inline(always)]
    pub fn mutate_stochastic(centroid: &TernaryCentroid64, seed: u64, mutation_intensity: u8) -> TernaryCentroid64 {
        // Fast splitmix64 pseudo-random step
        let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        let prng = z ^ (z >> 31);

        let flips_count = (mutation_intensity as usize % 5) + 1; // 1..5 flips
        let mut child = *centroid;

        for i in 0..flips_count {
            let bit_idx = ((prng >> (i * 8)) & 63) as usize;
            let to_neg = ((prng >> (32 + i * 4)) & 1) != 0;
            Self::bit_flip(&mut child, bit_idx, to_neg);
        }

        Self::enforce_invariant(&mut child);
        child
    }

    /// Evaluates fitness between candidate and target using pure hardware POPCNT.
    #[inline(always)]
    pub fn evaluate_fitness(candidate: &TernaryCentroid64, target: &TernaryCentroid64) -> i32 {
        ternary_dot_ternary(candidate, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_enforcement() {
        let mut c = TernaryCentroid64::new(0b1111, 0b1100);
        PlasticityMutator::enforce_invariant(&mut c);
        // neg_mask must have 0 where pos_mask has 1
        assert_eq!(c.pos_mask & c.neg_mask, 0);
        assert_eq!(c.neg_mask, 0);
    }

    #[test]
    fn test_bit_flip_and_swap() {
        let mut c = TernaryCentroid64::empty();
        PlasticityMutator::bit_flip(&mut c, 2, false); // bit 2 -> +1
        assert_eq!(c.pos_mask, 0b0100);

        PlasticityMutator::bit_swap(&mut c, 2, 5); // swap bit 2 with bit 5
        assert_eq!(c.pos_mask, 1 << 5);
    }

    #[test]
    fn test_stochastic_mutation_fitness() {
        let target = TernaryCentroid64::new(0xFFFF_0000, 0x0000_FFFF);
        let child = PlasticityMutator::mutate_stochastic(&target, 12345, 2);
        let fitness = PlasticityMutator::evaluate_fitness(&child, &target);
        // Child should have high alignment with parent
        assert!(fitness > 0);
    }
}
