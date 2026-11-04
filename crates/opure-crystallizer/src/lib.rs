//! OPURE Ternary Weight Crystallizer
//! 
//! Zero-copy weight ingestion, SafeTensors model squashing, and Absmean ternary quantization ({-1, 0, +1})
//! Packing 64-dimensional centroids into 16-byte dual bitmasks (4 per 64B cacheline).

#![no_std]
#![deny(warnings)]

pub mod squasher;

pub use squasher::*;

use opure_hal::{PackedTernaryCacheline, TernaryCentroid64};

/// Zero-copy unmanaged weight viewer for raw binary buffers or SafeTensors containers.
pub struct UnmanagedWeightView<'a> {
    data: &'a [u8],
}

impl<'a> UnmanagedWeightView<'a> {
    #[inline(always)]
    pub const fn from_raw_slice(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Read raw 64-element f32 slice at byte offset without allocating.
    pub fn read_64_f32(&self, byte_offset: usize) -> Option<&'a [f32; 64]> {
        let end = byte_offset.checked_add(64 * core::mem::size_of::<f32>())?;
        if end > self.data.len() {
            return None;
        }
        let slice = &self.data[byte_offset..end];
        if (slice.as_ptr() as usize) % core::mem::align_of::<f32>() != 0 {
            return None; // Unaligned
        }
        unsafe { Some(&*(slice.as_ptr() as *const [f32; 64])) }
    }

    /// Read raw 64-element i8 slice at byte offset without allocating.
    pub fn read_64_i8(&self, byte_offset: usize) -> Option<&'a [i8; 64]> {
        let end = byte_offset.checked_add(64)?;
        if end > self.data.len() {
            return None;
        }
        let slice = &self.data[byte_offset..end];
        unsafe { Some(&*(slice.as_ptr() as *const [i8; 64])) }
    }
}

/// Absmean 1.58-bit Ternary Quantizer.
pub struct AbsmeanTernaryQuantizer;

impl AbsmeanTernaryQuantizer {
    /// Quantize a 64-dimensional continuous float vector into a 16-byte `TernaryCentroid64`.
    pub fn quantize_f32_slice(weights: &[f32; 64]) -> TernaryCentroid64 {
        let mut abs_sum = 0.0f32;
        for &w in weights.iter() {
            abs_sum += if w < 0.0 { -w } else { w };
        }
        let gamma = abs_sum / 64.0;
        let threshold = 0.5 * gamma;

        let mut pos_mask = 0u64;
        let mut neg_mask = 0u64;

        for (i, &w) in weights.iter().enumerate() {
            if w > threshold {
                pos_mask |= 1u64 << i;
            } else if w < -threshold {
                neg_mask |= 1u64 << i;
            }
        }

        TernaryCentroid64::new(pos_mask, neg_mask)
    }

    /// Quantize a 64-dimensional normalized i8 vector into a 16-byte `TernaryCentroid64`.
    pub fn quantize_i8_slice(weights: &[i8; 64]) -> TernaryCentroid64 {
        let mut abs_sum: i32 = 0;
        for &w in weights.iter() {
            abs_sum += (w as i32).abs();
        }
        let gamma = abs_sum / 64;
        let threshold = gamma / 2;

        let mut pos_mask = 0u64;
        let mut neg_mask = 0u64;

        for (i, &w) in weights.iter().enumerate() {
            let val = w as i32;
            if val > threshold {
                pos_mask |= 1u64 << i;
            } else if val < -threshold {
                neg_mask |= 1u64 << i;
            }
        }

        TernaryCentroid64::new(pos_mask, neg_mask)
    }

    /// Packs 4 x 16-byte `TernaryCentroid64` structs into a single 64-byte `PackedTernaryCacheline`.
    #[inline(always)]
    pub fn crystallize_cacheline(
        c0: TernaryCentroid64,
        c1: TernaryCentroid64,
        c2: TernaryCentroid64,
        c3: TernaryCentroid64,
    ) -> PackedTernaryCacheline {
        PackedTernaryCacheline::new([c0, c1, c2, c3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absmean_quantize_f32() {
        let mut weights = [0.0f32; 64];
        weights[0] = 1.0;
        weights[1] = -1.0;
        weights[2] = 0.001;

        let centroid = AbsmeanTernaryQuantizer::quantize_f32_slice(&weights);
        assert_eq!(centroid.pos_mask & 1, 1);
        assert_eq!(centroid.neg_mask & 2, 2);
        assert_eq!(centroid.pos_mask & 4, 0);
        assert_eq!(centroid.neg_mask & 4, 0);
    }

    #[test]
    fn test_absmean_quantize_i8() {
        let mut weights = [0i8; 64];
        weights[0] = 50;
        weights[1] = -50;
        weights[2] = 0;

        let centroid = AbsmeanTernaryQuantizer::quantize_i8_slice(&weights);
        assert_eq!(centroid.pos_mask & 1, 1);
        assert_eq!(centroid.neg_mask & 2, 2);
        assert_eq!(centroid.pos_mask & 4, 0);
        assert_eq!(centroid.neg_mask & 4, 0);
    }

    #[test]
    fn test_cacheline_crystallization() {
        let c0 = TernaryCentroid64::new(1, 0);
        let c1 = TernaryCentroid64::new(2, 0);
        let c2 = TernaryCentroid64::new(4, 0);
        let c3 = TernaryCentroid64::new(8, 0);

        let cacheline = AbsmeanTernaryQuantizer::crystallize_cacheline(c0, c1, c2, c3);
        assert_eq!(cacheline.centroids[0], c0);
        assert_eq!(cacheline.centroids[1], c1);
        assert_eq!(cacheline.centroids[2], c2);
        assert_eq!(cacheline.centroids[3], c3);
    }

    #[test]
    fn test_safetensors_tensor_squashing() {
        let mut weights = [0.0f32; 128]; // 2 blocks of 64
        weights[0] = 1.5;
        weights[64] = -2.0;

        let mut crystal = SquashedModelCrystal::<16>::new();
        let squashed = SafeTensorsModelSquasher::squash_f32_tensor(&weights, &mut crystal);

        assert_eq!(squashed, 2);
        assert_eq!(crystal.count, 2);
        assert!(crystal.header.is_valid());
        assert_eq!(crystal.centroids[0].pos_mask & 1, 1);
        assert_eq!(crystal.centroids[1].neg_mask & 1, 1);
    }
}
