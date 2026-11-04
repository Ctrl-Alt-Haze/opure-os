//! OPURE RSI Ternary Plasticity Engine
//!
//! Manages unmanaged cacheline-aligned pools of mutable TernaryCentroid64 prototypes
//! executing gradient-free, bitwise online evolution.

#![deny(warnings)]

use opure_geometric::SystemIntent;
use opure_hal::{StaticSlab, TernaryCentroid64};
use crate::feedback::OnlineFeedbackLoop;
use crate::mutator::PlasticityMutator;

pub const MAX_MUTABLE_CENTROIDS: usize = 64;

/// Zero-Heap Ternary Plasticity Engine
pub struct TernaryPlasticityEngine {
    pub centroids: StaticSlab<TernaryCentroid64, MAX_MUTABLE_CENTROIDS>,
    pub scratch: StaticSlab<TernaryCentroid64, 16>,
    pub mutation_cycles: u64,
    pub successful_adaptations: u64,
}

impl TernaryPlasticityEngine {
    pub const fn new() -> Self {
        Self {
            centroids: StaticSlab::new(),
            scratch: StaticSlab::new(),
            mutation_cycles: 0,
            successful_adaptations: 0,
        }
    }

    /// Register an initial centroid prototype.
    pub fn register_centroid(&mut self, centroid: TernaryCentroid64) -> Result<usize, ()> {
        self.centroids.push(centroid)
    }

    /// Retrieve an immutable reference to a registered centroid.
    pub fn get_centroid(&self, class_idx: usize) -> Option<&TernaryCentroid64> {
        self.centroids.get(class_idx)
    }

    /// Step online evolution based on incoming runtime intent and telemetry.
    pub fn step_evolution(
        &mut self,
        class_idx: usize,
        observed: &TernaryCentroid64,
        intent: SystemIntent,
    ) -> bool {
        self.mutation_cycles += 1;
        let current = match self.centroids.get_mut(class_idx) {
            Some(c) => c,
            None => return false,
        };

        match intent {
            SystemIntent::NominalSteadyState => {
                OnlineFeedbackLoop::reinforce_nominal(current, observed, 0xFFFF_0000_FFFF_0000);
                self.successful_adaptations += 1;
                true
            }
            SystemIntent::ChaoticRegimeDivergence
            | SystemIntent::CoredumpAutoPatch
            | SystemIntent::EnclaveQuarantineLock => {
                let (best_candidate, fitness) = OnlineFeedbackLoop::adapt_on_divergence(
                    current,
                    observed,
                    intent,
                    self.mutation_cycles,
                    &mut self.scratch,
                );

                let current_fitness = PlasticityMutator::evaluate_fitness(current, observed);
                if fitness >= current_fitness {
                    *current = best_candidate;
                    self.successful_adaptations += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> Option<&TernaryCentroid64> {
        self.centroids.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_evolution_loop() {
        let mut engine = TernaryPlasticityEngine::new();
        let initial = TernaryCentroid64::new(0b0011, 0b1100);
        let idx = engine.register_centroid(initial).unwrap();

        let target = TernaryCentroid64::new(0b1111, 0b0000);
        let adapted = engine.step_evolution(idx, &target, SystemIntent::ChaoticRegimeDivergence);

        assert!(adapted);
        assert!(engine.mutation_cycles > 0);
    }
}
