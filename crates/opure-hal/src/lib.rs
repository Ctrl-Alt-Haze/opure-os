//! OPURE Hardware Abstraction Layer (HAL)
//! Zero-allocation memory models, strict cacheline alignment, and native SIMD primitives.

#![no_std]
#![deny(warnings)]

use core::ops::{Deref, DerefMut};

/// Cacheline aligned buffer (64 bytes), eliminating false sharing and ensuring AVX-512 alignment.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Aligned64<T>(pub T);

impl<T> Aligned64<T> {
    #[inline(always)]
    pub const fn new(val: T) -> Self {
        Self(val)
    }
}

impl<T> Deref for Aligned64<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Aligned64<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A 64-dimensional ternary prototype centroid (Weights in {-1, 0, +1}).
/// Stored as dual 64-bit masks: pos_mask (+1) and neg_mask (-1).
/// Total size: exactly 16 bytes!
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TernaryCentroid64 {
    pub pos_mask: u64,
    pub neg_mask: u64,
}

impl TernaryCentroid64 {
    pub const fn empty() -> Self {
        Self {
            pos_mask: 0,
            neg_mask: 0,
        }
    }

    #[inline(always)]
    pub const fn new(pos_mask: u64, neg_mask: u64) -> Self {
        Self { pos_mask, neg_mask }
    }

    /// Number of active non-zero ternary weights (L0 norm)
    #[inline(always)]
    pub fn non_zero_count(&self) -> u32 {
        self.pos_mask.count_ones() + self.neg_mask.count_ones()
    }

    /// L0 norm alias for non_zero_count
    #[inline(always)]
    pub fn l0_norm(&self) -> u32 {
        self.non_zero_count()
    }
}

/// Cacheline-packed 4x Ternary Centroids.
/// Fits 4 full 64D ternary class prototypes into a single 64-byte L1 cacheline.
/// Achieves 4x footprint reduction over i8 (256 bytes) and 16x reduction over f32 (1024 bytes).
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct PackedTernaryCacheline {
    pub centroids: [TernaryCentroid64; 4],
}

impl PackedTernaryCacheline {
    pub const fn empty() -> Self {
        Self {
            centroids: [TernaryCentroid64::empty(); 4],
        }
    }

    pub const fn new(centroids: [TernaryCentroid64; 4]) -> Self {
        Self { centroids }
    }
}

/// A zero-allocation fixed-size slab allocator designed for statically mapped memory blocks.
#[repr(C, align(64))]
pub struct StaticSlab<T, const CAPACITY: usize> {
    storage: [core::mem::MaybeUninit<T>; CAPACITY],
    len: usize,
}

impl<T, const CAPACITY: usize> StaticSlab<T, CAPACITY> {
    pub const fn new() -> Self {
        Self {
            storage: [const { core::mem::MaybeUninit::uninit() }; CAPACITY],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn push(&mut self, item: T) -> Result<usize, ()> {
        if self.len >= CAPACITY {
            return Err(());
        }
        let idx = self.len;
        self.storage[idx].write(item);
        self.len += 1;
        Ok(idx)
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(self.storage[self.len].assume_init_read()) }
        }
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            unsafe { Some(self.storage[idx].assume_init_ref()) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            unsafe { Some(self.storage[idx].assume_init_mut()) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        // Drop initialized items
        for i in 0..self.len {
            unsafe {
                self.storage[i].assume_init_drop();
            }
        }
        self.len = 0;
    }
}

impl<T, const CAPACITY: usize> Drop for StaticSlab<T, CAPACITY> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Native Hardware SIMD & Multiplier-Less Bitwise Kernels.
pub mod simd {
    use super::TernaryCentroid64;

    /// Multiplier-less ternary inner product between a 64D ternary centroid and an i8 vector.
    /// Accumulates purely through addition and subtraction without hardware integer multipliers.
    #[inline(always)]
    pub fn ternary_dot_u64(centroid: &TernaryCentroid64, input: &[i8; 64]) -> i32 {
        let mut accum: i32 = 0;
        let mut pos = centroid.pos_mask;
        let mut neg = centroid.neg_mask;

        while pos != 0 {
            let bit = pos.trailing_zeros() as usize;
            accum += input[bit] as i32;
            pos &= pos - 1; // Clear lowest set bit
        }

        while neg != 0 {
            let bit = neg.trailing_zeros() as usize;
            accum -= input[bit] as i32;
            neg &= neg - 1; // Clear lowest set bit
        }

        accum
    }

    /// Pure POPCOUNT inner product between two 64D ternary vectors.
    /// Zero multiplication, zero memory reads. Fully hardware popcount accelerated.
    #[inline(always)]
    pub fn ternary_dot_ternary(a: &TernaryCentroid64, b: &TernaryCentroid64) -> i32 {
        let pos_matches = (a.pos_mask & b.pos_mask).count_ones() + (a.neg_mask & b.neg_mask).count_ones();
        let neg_matches = (a.pos_mask & b.neg_mask).count_ones() + (a.neg_mask & b.pos_mask).count_ones();
        (pos_matches as i32) - (neg_matches as i32)
    }

    /// Multiplier-less squared Euclidean distance metric between a 64D ternary centroid and an i8 vector.
    /// Formula: sum( (x_i - w_i)^2 ) = sum(x_i^2) - 2 * ternary_dot(w, x) + non_zero_count(w)
    /// Since sum(x_i^2) is constant across all centroids during a classification pass,
    /// relative distance is determined by: (non_zero_count - 2 * dot).
    #[inline(always)]
    pub fn ternary_relative_dist_sq(centroid: &TernaryCentroid64, input: &[i8; 64]) -> i32 {
        let dot = ternary_dot_u64(centroid, input);
        (centroid.non_zero_count() as i32) - (dot << 1)
    }

    /// Compute 8-bit integer dot product with zero floating-point operations.
    #[inline(always)]
    pub fn dot_i8_i8(a: &[i8], b: &[i8]) -> i32 {
        debug_assert_eq!(a.len(), b.len());
        let len = a.len();
        let mut sum: i32 = 0;

        let chunks = len / 8;
        for i in 0..chunks {
            let base = i * 8;
            sum += (a[base] as i32) * (b[base] as i32)
                + (a[base + 1] as i32) * (b[base + 1] as i32)
                + (a[base + 2] as i32) * (b[base + 2] as i32)
                + (a[base + 3] as i32) * (b[base + 3] as i32)
                + (a[base + 4] as i32) * (b[base + 4] as i32)
                + (a[base + 5] as i32) * (b[base + 5] as i32)
                + (a[base + 6] as i32) * (b[base + 6] as i32)
                + (a[base + 7] as i32) * (b[base + 7] as i32);
        }

        for i in (chunks * 8)..len {
            sum += (a[i] as i32) * (b[i] as i32);
        }

        sum
    }

    /// Compute L2 squared distance between two i8 vectors without floats.
    #[inline(always)]
    pub fn l2_dist_sq_i8(a: &[i8], b: &[i8]) -> u32 {
        debug_assert_eq!(a.len(), b.len());
        let len = a.len();
        let mut dist: u32 = 0;

        for i in 0..len {
            let diff = (a[i] as i32) - (b[i] as i32);
            dist += (diff * diff) as u32;
        }

        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned64_alignment() {
        let aligned = Aligned64::new([0u8; 128]);
        let ptr = &aligned.0 as *const _ as usize;
        assert_eq!(ptr % 64, 0, "Buffer must be strictly 64-byte aligned");
    }

    #[test]
    fn test_packed_ternary_cacheline_size() {
        assert_eq!(core::mem::size_of::<TernaryCentroid64>(), 16);
        assert_eq!(core::mem::size_of::<PackedTernaryCacheline>(), 64);
        assert_eq!(core::mem::align_of::<PackedTernaryCacheline>(), 64);
    }

    #[test]
    fn test_ternary_dot_multiplierless() {
        // Pos on bit 0 and 1, Neg on bit 2
        let centroid = TernaryCentroid64::new(0b0011, 0b0100);
        let mut input = [0i8; 64];
        input[0] = 10;
        input[1] = 20;
        input[2] = 5;

        let dot = simd::ternary_dot_u64(&centroid, &input);
        // 10 + 20 - 5 = 25
        assert_eq!(dot, 25);
    }

    #[test]
    fn test_ternary_dot_popcount() {
        let a = TernaryCentroid64::new(0b0011, 0b1100);
        let b = TernaryCentroid64::new(0b0001, 0b1000);
        // Bit 0: a=+1, b=+1 -> +1
        // Bit 1: a=+1, b=0  -> 0
        // Bit 2: a=-1, b=0  -> 0
        // Bit 3: a=-1, b=-1 -> +1
        // Total = 2
        let dot = simd::ternary_dot_ternary(&a, &b);
        assert_eq!(dot, 2);
    }
}
