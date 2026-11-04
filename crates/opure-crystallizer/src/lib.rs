//! OPURE Ternary Weight Crystallizer
//! 
//! Zero-copy weight ingestion and Absmean ternary quantization ({-1, 0, +1})
//! Packing 64-dimensional centroids into 16-byte dual bitmasks (4 per 64B cacheline).

#![no_std]
#![deny(warnings)]

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
    /// 
    /// Formula:
    ///   gamma = (1 / 64) * sum(|W_i|)
    ///   threshold = 0.5 * gamma
    ///   W_ternary = +1 if W_i > threshold, -1 if W_i < -threshold, else 0
    pub fn quantize_f32_slice(weights: &[f32; 64]) -> TernaryCentroid64 {
        let mut abs_sum = 0.0f32;
        for &w in weights.iter() {
            abs_sum += if w < 0.0 { -w } else { w };
        }

        let gamma = abs_sum / 64.0;
        let threshold = 0.5 * gamma;

        let mut pos_mask: u64 = 0;
        let mut neg_mask: u64 = 0;

        for (i, &w) in weights.iter().enumerate() {
            if w > threshold {
                pos_mask |= 1u64 << i;
            } else if w < -threshold {
                neg_mask |= 1u64 << i;
            }
        }

        TernaryCentroid64::new(pos_mask, neg_mask)
    }

    /// Quantize a 64-dimensional i8 vector (e.g. integer telemetry) into a 16-byte `TernaryCentroid64`.
    pub fn quantize_i8_slice(weights: &[i8; 64]) -> TernaryCentroid64 {
        let mut abs_sum: u32 = 0;
        for &w in weights.iter() {
            abs_sum += (w as i16).unsigned_abs() as u32;
        }

        // gamma = abs_sum / 64; threshold = gamma / 2 = abs_sum / 128
        let threshold = (abs_sum / 128) as i8;

        let mut pos_mask: u64 = 0;
        let mut neg_mask: u64 = 0;

        for (i, &w) in weights.iter().enumerate() {
            if w > threshold {
                pos_mask |= 1u64 << i;
            } else if w < -threshold {
                neg_mask |= 1u64 << i;
            }
        }

        TernaryCentroid64::new(pos_mask, neg_mask)
    }

    /// Batch crystallize 4 external weight slices into a single 64-byte `PackedTernaryCacheline`.
    pub fn crystallize_cacheline_f32(batch: [&[f32; 64]; 4]) -> PackedTernaryCacheline {
        PackedTernaryCacheline::new([
            Self::quantize_f32_slice(batch[0]),
            Self::quantize_f32_slice(batch[1]),
            Self::quantize_f32_slice(batch[2]),
            Self::quantize_f32_slice(batch[3]),
        ])
    }

    /// Batch crystallize 4 i8 weight slices into a single 64-byte `PackedTernaryCacheline`.
    pub fn crystallize_cacheline_i8(batch: [&[i8; 64]; 4]) -> PackedTernaryCacheline {
        PackedTernaryCacheline::new([
            Self::quantize_i8_slice(batch[0]),
            Self::quantize_i8_slice(batch[1]),
            Self::quantize_i8_slice(batch[2]),
            Self::quantize_i8_slice(batch[3]),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absmean_quantize_f32() {
        let mut weights = [0.0f32; 64];
        weights[0] = 1.0;
        weights[1] = 0.8;
        weights[2] = -0.9;
        weights[3] = 0.01; // Below threshold (0.01 < gamma/2) -> 0

        let centroid = AbsmeanTernaryQuantizer::quantize_f32_slice(&weights);
        // Bit 0 and 1 must be set in pos_mask
        assert_eq!(centroid.pos_mask & 0b0011, 0b0011);
        // Bit 2 must be set in neg_mask
        assert_eq!(centroid.neg_mask & 0b0100, 0b0100);
        // Bit 3 must be 0 in both
        assert_eq!(centroid.pos_mask & 0b1000, 0);
        assert_eq!(centroid.neg_mask & 0b1000, 0);
    }

    #[test]
    fn test_absmean_quantize_i8() {
        let mut weights = [0i8; 64];
        weights[10] = 80;
        weights[20] = -75;

        let centroid = AbsmeanTernaryQuantizer::quantize_i8_slice(&weights);
        assert_ne!(centroid.pos_mask & (1u64 << 10), 0);
        assert_ne!(centroid.neg_mask & (1u64 << 20), 0);
    }

    #[test]
    fn test_cacheline_crystallization() {
        let w0 = [10i8; 64];
        let w1 = [-10i8; 64];
        let w2 = [20i8; 64];
        let w3 = [-20i8; 64];

        let line = AbsmeanTernaryQuantizer::crystallize_cacheline_i8([&w0, &w1, &w2, &w3]);
        assert_eq!(line.centroids.len(), 4);
    }
}
