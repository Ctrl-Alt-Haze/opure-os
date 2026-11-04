//! OPURE Distributed Geometric Centroid Consensus
//!
//! Reconciles candidate TernaryCentroid64 prototypes across cluster nodes via
//! bitwise majority voting without Paxos/Raft consensus serialization lag.

#![deny(warnings)]

use opure_hal::TernaryCentroid64;

pub struct GeometricConsensus;

impl GeometricConsensus {
    /// Compute consensus centroid across N peer candidate prototypes using majority bitwise voting.
    /// Invariant: pos_mask and neg_mask are strictly mutually exclusive.
    pub fn compute_consensus<const N: usize>(
        candidates: &[TernaryCentroid64; N],
    ) -> TernaryCentroid64 {
        if N == 0 {
            return TernaryCentroid64::empty();
        }

        let threshold = (N / 2) + 1;
        let mut consensus_pos: u64 = 0;
        let mut consensus_neg: u64 = 0;

        for bit in 0..64 {
            let mask = 1u64 << bit;
            let mut pos_votes = 0;
            let mut neg_votes = 0;

            for i in 0..N {
                if (candidates[i].pos_mask & mask) != 0 {
                    pos_votes += 1;
                }
                if (candidates[i].neg_mask & mask) != 0 {
                    neg_votes += 1;
                }
            }

            if pos_votes >= threshold && pos_votes > neg_votes {
                consensus_pos |= mask;
            } else if neg_votes >= threshold && neg_votes > pos_votes {
                consensus_neg |= mask;
            }
        }

        let mut res = TernaryCentroid64::new(consensus_pos, consensus_neg);
        // Enforce ternary mutual exclusivity invariant
        res.neg_mask &= !res.pos_mask;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometric_consensus_majority_voting() {
        let mut candidates = [TernaryCentroid64::empty(); 3];
        // 2 nodes vote positive for lower 32 bits, 1 votes negative
        candidates[0] = TernaryCentroid64::new(0x0000_0000_FFFF_FFFF, 0x0000_0000_0000_0000);
        candidates[1] = TernaryCentroid64::new(0x0000_0000_FFFF_FFFF, 0x0000_0000_0000_0000);
        candidates[2] = TernaryCentroid64::new(0x0000_0000_0000_0000, 0x0000_0000_FFFF_FFFF);

        let consensus = GeometricConsensus::compute_consensus(&candidates);
        assert_eq!(consensus.pos_mask, 0x0000_0000_FFFF_FFFF);
        assert_eq!(consensus.neg_mask, 0);
    }
}
