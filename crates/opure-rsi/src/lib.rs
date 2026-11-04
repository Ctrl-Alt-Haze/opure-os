//! OPURE RSI (Recursive Self-Improvement) Ternary Centroid Plasticity Engine
//!
//! Enables OPURE's cognitive routing and security baselines to self-optimize online
//! without floating-point backpropagation, utilizing gradient-free, bitwise evolution
//! and hardware POPCNT fitness scoring.

#![no_std]
#![deny(warnings)]

pub mod mutator;
pub mod feedback;
pub mod engine;

pub use mutator::PlasticityMutator;
pub use feedback::OnlineFeedbackLoop;
pub use engine::{TernaryPlasticityEngine, MAX_MUTABLE_CENTROIDS};

use opure_hal::TernaryCentroid64;

/// C99 FFI Export for external kernel-mode or hypervisor-level centroid adaptation
#[no_mangle]
pub unsafe extern "C" fn opure_rsi_mutate_centroid_c(
    centroid_ptr: *mut TernaryCentroid64,
    mutation_mask: u32,
    mode: u32,
) -> i32 {
    if centroid_ptr.is_null() {
        return -1;
    }

    let centroid = &mut *centroid_ptr;
    match mode {
        0 => {
            // Deterministic bit-flip
            PlasticityMutator::bit_flip(centroid, (mutation_mask & 63) as usize, false);
        }
        1 => {
            // Mask shift
            PlasticityMutator::mask_shift(centroid, mutation_mask & 31);
        }
        2 => {
            // Stochastic mutation pass
            *centroid = PlasticityMutator::mutate_stochastic(centroid, mutation_mask as u64, 3);
        }
        _ => return -2,
    }

    PlasticityMutator::enforce_invariant(centroid);
    0
}
