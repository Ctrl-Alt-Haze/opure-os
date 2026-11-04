//! 4KB Content-Addressed Block & Superblock Architecture
//!
//! Enforces zero-allocation 4KB alignment matching CPU L1/L2 and NVMe physical page boundaries.

pub const FS_BLOCK_SIZE: usize = 4096;
pub const FS_SUPERBLOCK_MAGIC: u64 = 0x4F505552_46533031; // "OPURFS01"

/// 4KB-aligned physical disk block.
#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct FsBlock4K {
    pub data: [u8; FS_BLOCK_SIZE],
}

impl FsBlock4K {
    pub const fn empty() -> Self {
        Self {
            data: [0u8; FS_BLOCK_SIZE],
        }
    }

    /// Computes deterministic 64-bit Merkle digest across 4KB block data.
    #[inline(always)]
    pub fn compute_digest(&self) -> u64 {
        let mut digest: u64 = 0xCBF29CE484222325; // FNV-1a 64-bit offset basis
        let mut i = 0;
        while i < FS_BLOCK_SIZE {
            // Read 8 bytes at a time
            let chunk = u64::from_le_bytes([
                self.data[i],
                self.data[i + 1],
                self.data[i + 2],
                self.data[i + 3],
                self.data[i + 4],
                self.data[i + 5],
                self.data[i + 6],
                self.data[i + 7],
            ]);
            digest ^= chunk;
            digest = digest.wrapping_mul(0x100000001B3); // FNV-1a 64-bit prime
            i += 8;
        }
        digest
    }
}

/// OPURE-FS Root Superblock.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct OpureFsSuperblock {
    pub magic: u64,
    pub block_size: u32,
    pub total_blocks: u32,
    pub free_blocks: u32,
    pub root_morton_index: u32,
    pub merkle_root: u64,
    pub generation: u64,
    pub reserved: [u64; 4],
}

impl OpureFsSuperblock {
    pub const fn new(total_blocks: u32) -> Self {
        Self {
            magic: FS_SUPERBLOCK_MAGIC,
            block_size: FS_BLOCK_SIZE as u32,
            total_blocks,
            free_blocks: total_blocks.saturating_sub(1), // Block 0 is superblock
            root_morton_index: 0,
            merkle_root: 0,
            generation: 1,
            reserved: [0; 4],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == FS_SUPERBLOCK_MAGIC && self.block_size == (FS_BLOCK_SIZE as u32)
    }
}
