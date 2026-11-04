//! NCM Cognitive Routing & Scottish Diagnostic Bridge
//!
//! Hashes incoming natural language prompts into 64D JointEmbeddingVectors,
//! executes sub-48ns TernaryNcmRouter classification, and formats responses.

#![deny(warnings)]

use opure_geometric::{DEFAULT_EMBEDDING_DIM, SystemIntent, TernaryNcmRouter};
use opure_shell::ScottishFeedbackLayer;

pub struct NcmEndpointBridge;

impl NcmEndpointBridge {
    /// Deterministic string to 64D feature vector projection.
    /// Maps character sequences into normalized i8 dimensions without heap allocations.
    pub fn embed_prompt(prompt: &str, out_vector: &mut [i8; DEFAULT_EMBEDDING_DIM]) {
        *out_vector = [0i8; DEFAULT_EMBEDDING_DIM];
        let bytes = prompt.as_bytes();

        let mut hash: u64 = 0xCBF29CE484222325; // FNV offset basis
        for (idx, &b) in bytes.iter().enumerate() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001B3);

            let dim = (idx + (hash as usize)) % DEFAULT_EMBEDDING_DIM;
            let val = ((hash >> 32) as i8) ^ (b as i8);
            out_vector[dim] = out_vector[dim].saturating_add(val);
        }
    }

    /// Embed prompt into a normalized 64D float array for `/v1/embeddings`.
    pub fn embed_prompt_f32(prompt: &str, out_embedding: &mut [f32; DEFAULT_EMBEDDING_DIM]) {
        let mut i8_vec = [0i8; DEFAULT_EMBEDDING_DIM];
        Self::embed_prompt(prompt, &mut i8_vec);

        for i in 0..DEFAULT_EMBEDDING_DIM {
            out_embedding[i] = (i8_vec[i] as f32) / 128.0;
        }
    }

    /// Route prompt via TernaryNcmRouter and return intent, confidence score, and Scottish response.
    pub fn route_and_generate<const NUM_CLASSES: usize>(
        prompt: &str,
        router: &TernaryNcmRouter<NUM_CLASSES>,
    ) -> (SystemIntent, i32, &'static str) {
        let mut feature_vec = [0i8; DEFAULT_EMBEDDING_DIM];
        Self::embed_prompt(prompt, &mut feature_vec);

        let (intent, score, _) = router.route_ternary(&feature_vec);
        let feedback = ScottishFeedbackLayer::get_phrase(intent);

        (intent, score, feedback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opure_hal::TernaryCentroid64;

    #[test]
    fn test_embed_prompt_deterministic() {
        let mut v1 = [0i8; 64];
        let mut v2 = [0i8; 64];
        NcmEndpointBridge::embed_prompt("Status check: core thermal load", &mut v1);
        NcmEndpointBridge::embed_prompt("Status check: core thermal load", &mut v2);
        assert_eq!(v1, v2);

        let mut v3 = [0i8; 64];
        NcmEndpointBridge::embed_prompt("Completely different query", &mut v3);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_route_and_generate() {
        let mut centroids = [TernaryCentroid64::empty(); 2];
        centroids[0] = TernaryCentroid64::new(0xFFFF_0000_FFFF_0000, 0x0000_FFFF_0000_FFFF);
        centroids[1] = TernaryCentroid64::new(0x0000_FFFF_0000_FFFF, 0xFFFF_0000_FFFF_0000);
        let intents = [SystemIntent::NominalSteadyState, SystemIntent::ThermalThrottleCore];
        let router = TernaryNcmRouter::new(centroids, intents);

        let (intent, _score, feedback) = NcmEndpointBridge::route_and_generate("Test prompt", &router);
        assert!(!feedback.is_empty());
        assert!(intent == SystemIntent::NominalSteadyState || intent == SystemIntent::ThermalThrottleCore);
    }
}
