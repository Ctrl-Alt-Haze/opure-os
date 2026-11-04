//! OPURE Silicon Coprocessor Dispatch & Batch Evaluator
//!
//! Provides a zero-allocation, driverless interface mimicking bare-metal GPU
//! Tensor Core execution for 1.58-bit ternary matrix operations.

#![deny(warnings)]

use opure_hal::TernaryCentroid64;
use opure_geometric::SystemIntent;

pub const MAX_BATCH_PROTOTYPES: usize = 64;

/// High-Throughput Hardware Coprocessor Pipeline
pub struct TernarySiliconCoprocessor<const NUM_CLASSES: usize> {
    pub protos_pos: [u64; NUM_CLASSES],
    pub protos_neg: [u64; NUM_CLASSES],
    pub class_intents: [SystemIntent; NUM_CLASSES],
}

impl<const NUM_CLASSES: usize> TernarySiliconCoprocessor<NUM_CLASSES> {
    pub fn new(
        centroids: &[TernaryCentroid64; NUM_CLASSES],
        class_intents: [SystemIntent; NUM_CLASSES],
    ) -> Self {
        let mut protos_pos = [0u64; NUM_CLASSES];
        let mut protos_neg = [0u64; NUM_CLASSES];

        for i in 0..NUM_CLASSES {
            protos_pos[i] = centroids[i].pos_mask;
            protos_neg[i] = centroids[i].neg_mask;
        }

        Self {
            protos_pos,
            protos_neg,
            class_intents,
        }
    }

    /// Silicon warp-parallel evaluation of a single 64D query against all prototypes.
    /// Bit-for-bit identical to the `k_ternary_ncm_gemv` PTX kernel.
    #[inline(always)]
    pub fn execute_warp_gemv(
        &self,
        query: &TernaryCentroid64,
        scores_out: &mut [i32; NUM_CLASSES],
    ) -> (SystemIntent, i32) {
        let in_pos = query.pos_mask;
        let in_neg = query.neg_mask;

        let mut best_score = i32::MIN;
        let mut best_idx = 0;

        for gid in 0..NUM_CLASSES {
            let p_pos = self.protos_pos[gid];
            let p_neg = self.protos_neg[gid];

            // Emulates PTX: and.b64 followed by popc.b64
            let match_pos = (in_pos & p_pos).count_ones();
            let match_neg = (in_neg & p_neg).count_ones();
            let mismatch_0 = (in_pos & p_neg).count_ones();
            let mismatch_1 = (in_neg & p_pos).count_ones();

            let score = ((match_pos + match_neg) as i32) - ((mismatch_0 + mismatch_1) as i32);
            scores_out[gid] = score;

            if score > best_score {
                best_score = score;
                best_idx = gid;
            }
        }

        (self.class_intents[best_idx], best_score)
    }

    /// Batch evaluate an array of queries across parallel SIMD lanes.
    #[inline(always)]
    pub fn execute_batch<const BATCH_SIZE: usize>(
        &self,
        queries: &[TernaryCentroid64; BATCH_SIZE],
        out_intents: &mut [SystemIntent; BATCH_SIZE],
        out_scores: &mut [i32; BATCH_SIZE],
    ) {
        let mut scratch_scores = [0i32; NUM_CLASSES];
        for i in 0..BATCH_SIZE {
            let (intent, score) = self.execute_warp_gemv(&queries[i], &mut scratch_scores);
            out_intents[i] = intent;
            out_scores[i] = score;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coprocessor_gemv_execution() {
        let mut centroids = [TernaryCentroid64::empty(); 4];
        centroids[0] = TernaryCentroid64::new(0xFFFF_FFFF_0000_0000, 0x0000_0000_FFFF_FFFF);
        centroids[1] = TernaryCentroid64::new(0x0000_0000_FFFF_FFFF, 0xFFFF_FFFF_0000_0000);
        centroids[2] = TernaryCentroid64::new(0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA);
        centroids[3] = TernaryCentroid64::new(0xAAAA_AAAA_AAAA_AAAA, 0x5555_5555_5555_5555);

        let intents = [
            SystemIntent::NominalSteadyState,
            SystemIntent::ThermalThrottleCore,
            SystemIntent::ProcessHangDeadlock,
            SystemIntent::ChaoticRegimeDivergence,
        ];

        let coprocessor = TernarySiliconCoprocessor::new(&centroids, intents);
        let mut scores = [0i32; 4];

        // Query matching centroid 0 exactly
        let query0 = centroids[0];
        let (intent0, score0) = coprocessor.execute_warp_gemv(&query0, &mut scores);
        assert_eq!(intent0, SystemIntent::NominalSteadyState);
        assert_eq!(score0, 64);

        // Query matching centroid 1 exactly
        let query1 = centroids[1];
        let (intent1, score1) = coprocessor.execute_warp_gemv(&query1, &mut scores);
        assert_eq!(intent1, SystemIntent::ThermalThrottleCore);
        assert_eq!(score1, 64);
    }
}
