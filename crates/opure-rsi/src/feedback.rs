//! OPURE RSI Online Reward-Driven Feedback Loop
//!
//! Hooks into opure-bus event streams and opure-oztae audit frames to drive
//! continuous centroid plasticity without floating-point backpropagation.

#![deny(warnings)]

use opure_geometric::SystemIntent;
use opure_hal::{StaticSlab, TernaryCentroid64};
use crate::mutator::PlasticityMutator;

pub struct OnlineFeedbackLoop;

impl OnlineFeedbackLoop {
    /// Reinforce centroid bitmasks when execution achieves NominalSteadyState.
    #[inline(always)]
    pub fn reinforce_nominal(
        centroid: &mut TernaryCentroid64,
        observed_sample: &TernaryCentroid64,
        reinforce_rate_mask: u64,
    ) {
        // Incorporate observed active bits under the reinforcement filter mask
        let active_pos = observed_sample.pos_mask & reinforce_rate_mask;
        let active_neg = observed_sample.neg_mask & reinforce_rate_mask;

        centroid.pos_mask |= active_pos;
        centroid.neg_mask |= active_neg;
        PlasticityMutator::enforce_invariant(centroid);
    }

    /// Adapt centroid upon fault / chaotic divergence:
    /// Spawns mutated candidate centroids in an unmanaged scratch slab,
    /// evaluates fitness against historical target via hardware POPCNT,
    /// and selects the superior candidate.
    pub fn adapt_on_divergence(
        current_centroid: &TernaryCentroid64,
        ideal_target: &TernaryCentroid64,
        intent: SystemIntent,
        seed: u64,
        scratch_slab: &mut StaticSlab<TernaryCentroid64, 16>,
    ) -> (TernaryCentroid64, i32) {
        scratch_slab.clear();

        let intensity = match intent {
            SystemIntent::ChaoticRegimeDivergence => 4, // High exploration
            SystemIntent::CoredumpAutoPatch => 3,
            SystemIntent::EnclaveQuarantineLock => 5,
            _ => 1,
        };

        let mut best_candidate = *current_centroid;
        let mut best_fitness = PlasticityMutator::evaluate_fitness(&best_candidate, ideal_target);

        // Generate and evaluate 8 local mutated candidates in unmanaged scratch slab
        for i in 0..8u64 {
            let child = PlasticityMutator::mutate_stochastic(
                current_centroid,
                seed.wrapping_add(i.wrapping_mul(0x517CC1B727220A95)),
                intensity,
            );
            let fitness = PlasticityMutator::evaluate_fitness(&child, ideal_target);
            if fitness > best_fitness {
                best_fitness = fitness;
                best_candidate = child;
            }
            let _ = scratch_slab.push(child);
        }

        (best_candidate, best_fitness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reinforcement_nominal() {
        let mut centroid = TernaryCentroid64::new(0b0001, 0b0010);
        let sample = TernaryCentroid64::new(0b0100, 0b1000);

        OnlineFeedbackLoop::reinforce_nominal(&mut centroid, &sample, 0xFFFF_FFFF);
        assert_eq!(centroid.pos_mask, 0b0101);
        assert_eq!(centroid.neg_mask, 0b1010);
    }

    #[test]
    fn test_adapt_on_divergence_selection() {
        let current = TernaryCentroid64::new(0b0000_1111, 0b1111_0000);
        let target = TernaryCentroid64::new(0b1111_1111, 0b0000_0000);
        let mut slab = StaticSlab::<TernaryCentroid64, 16>::new();

        let (best, fitness) = OnlineFeedbackLoop::adapt_on_divergence(
            &current,
            &target,
            SystemIntent::ChaoticRegimeDivergence,
            9999,
            &mut slab,
        );

        assert!(fitness >= PlasticityMutator::evaluate_fitness(&current, &target));
        assert_eq!(slab.len(), 8);
        assert_ne!(best, TernaryCentroid64::empty());
    }
}
