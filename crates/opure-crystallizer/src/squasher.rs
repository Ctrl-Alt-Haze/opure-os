//! SafeTensors Model Ingestion & 1.58-Bit Ternary Squashing Engine
//!
//! Converts floating-point and integer tensor weights directly into
//! 16-byte `TernaryCentroid64` prototypes and packed cachelines without heap allocation.

use crate::AbsmeanTernaryQuantizer;
use opure_hal::TernaryCentroid64;

pub const OPURE_CRYSTAL_MAGIC: u64 = 0x4F505552_43525953; // "OPURCRYS"

/// Serialized header for a compiled 1.58-bit OPURE Crystal Model.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct OpureCrystalHeader {
    pub magic: u64,
    pub model_version: u32,
    pub num_centroids: u32,
    pub quant_method: u32, // 1 = Absmean 1.58-bit
    pub original_dim: u32,
    pub reserved: [u64; 6],
}

impl OpureCrystalHeader {
    pub const fn new(num_centroids: u32) -> Self {
        Self {
            magic: OPURE_CRYSTAL_MAGIC,
            model_version: 1,
            num_centroids,
            quant_method: 1,
            original_dim: 64,
            reserved: [0; 6],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == OPURE_CRYSTAL_MAGIC
    }
}

/// Unmanaged container storing up to `MAX_NODES` squashed 1.58-bit centroids.
#[repr(C, align(64))]
pub struct SquashedModelCrystal<const MAX_NODES: usize> {
    pub header: OpureCrystalHeader,
    pub centroids: [TernaryCentroid64; MAX_NODES],
    pub count: usize,
}

impl<const MAX_NODES: usize> SquashedModelCrystal<MAX_NODES> {
    pub const fn new() -> Self {
        Self {
            header: OpureCrystalHeader::new(0),
            centroids: [TernaryCentroid64::new(0, 0); MAX_NODES],
            count: 0,
        }
    }

    /// Appends a squashed 1.58-bit centroid into the crystal model.
    pub fn push_centroid(&mut self, centroid: TernaryCentroid64) -> Result<(), ()> {
        if self.count >= MAX_NODES {
            return Err(());
        }
        self.centroids[self.count] = centroid;
        self.count += 1;
        self.header.num_centroids = self.count as u32;
        Ok(())
    }
}

/// SafeTensors / Binary Tensor Squasher
pub struct SafeTensorsModelSquasher;

impl SafeTensorsModelSquasher {
    /// Squashes continuous `f32` weights into 1.58-bit ternary centroids.
    pub fn squash_f32_tensor<const MAX_NODES: usize>(
        weights: &[f32],
        crystal: &mut SquashedModelCrystal<MAX_NODES>,
    ) -> usize {
        let num_blocks = weights.len() / 64;
        let mut squashed_count = 0;

        for block_idx in 0..num_blocks {
            let offset = block_idx * 64;
            let slice: &[f32; 64] = weights[offset..offset + 64].try_into().unwrap();
            let centroid = AbsmeanTernaryQuantizer::quantize_f32_slice(slice);
            if crystal.push_centroid(centroid).is_ok() {
                squashed_count += 1;
            } else {
                break;
            }
        }

        squashed_count
    }

    /// Squashes quantized `i8` embedding / weight matrices into 1.58-bit ternary centroids.
    pub fn squash_i8_tensor<const MAX_NODES: usize>(
        weights: &[i8],
        crystal: &mut SquashedModelCrystal<MAX_NODES>,
    ) -> usize {
        let num_blocks = weights.len() / 64;
        let mut squashed_count = 0;

        for block_idx in 0..num_blocks {
            let offset = block_idx * 64;
            let slice: &[i8; 64] = weights[offset..offset + 64].try_into().unwrap();
            let centroid = AbsmeanTernaryQuantizer::quantize_i8_slice(slice);
            if crystal.push_centroid(centroid).is_ok() {
                squashed_count += 1;
            } else {
                break;
            }
        }

        squashed_count
    }
}
