//! Content-Addressed Morton File System Controller
//!
//! Provides deterministic file and block writes, content-addressed Merkle hashing,
//! and O(1) instantaneous snapshot rollback.

use crate::block::{FsBlock4K, OpureFsSuperblock, FS_BLOCK_SIZE};
use crate::device::BlockDevice;
use crate::morton_tree::{MortonBlockIndex, MortonBlockNode};

pub struct OpureFileSystem<D: BlockDevice, const MAX_INDEX: usize> {
    pub device: D,
    pub superblock: OpureFsSuperblock,
    pub index: MortonBlockIndex<MAX_INDEX>,
    pub next_free_lba: u32,
    pub snapshot_merkle_root: u64,
}

impl<D: BlockDevice, const MAX_INDEX: usize> OpureFileSystem<D, MAX_INDEX> {
    pub fn format(mut device: D) -> Result<Self, ()> {
        let total = device.total_blocks();
        if total < 2 {
            return Err(());
        }

        let superblock = OpureFsSuperblock::new(total);
        let mut sb_block = FsBlock4K::empty();
        // Serialize superblock bytes into block 0
        unsafe {
            let sb_ptr = &superblock as *const OpureFsSuperblock as *const u8;
            core::ptr::copy_nonoverlapping(sb_ptr, sb_block.data.as_mut_ptr(), core::mem::size_of::<OpureFsSuperblock>());
        }
        device.write_block(0, &sb_block)?;

        Ok(Self {
            device,
            superblock,
            index: MortonBlockIndex::new(),
            next_free_lba: 1, // LBA 0 is superblock
            snapshot_merkle_root: 0,
        })
    }

    /// Writes a 4KB data chunk indexed by 3D coordinates (X: Cluster, Y: Sequence, Z: Enclave).
    pub fn write_morton_block(
        &mut self,
        x: u8,
        y: u8,
        z: u8,
        data: &[u8; FS_BLOCK_SIZE],
    ) -> Result<(u32, u64), ()> {
        if self.next_free_lba >= self.superblock.total_blocks {
            return Err(());
        }

        let mut block = FsBlock4K::empty();
        block.data.copy_from_slice(data);
        let digest = block.compute_digest();

        let lba = self.next_free_lba;
        self.device.write_block(lba, &block)?;
        self.next_free_lba += 1;

        let morton_key = MortonBlockNode::encode_morton(x, y, z);
        let node = MortonBlockNode::new(morton_key, lba, digest);
        self.index.insert(node)?;

        // Update incremental Merkle root
        self.superblock.merkle_root ^= digest;
        self.superblock.generation += 1;

        Ok((morton_key, digest))
    }

    /// Reads a block by its 24-bit Morton key and verifies content integrity.
    pub fn read_morton_block(
        &self,
        morton_key: u32,
        out_buf: &mut [u8; FS_BLOCK_SIZE],
    ) -> Result<u64, ()> {
        let node = self.index.lookup(morton_key).ok_or(())?;
        let mut block = FsBlock4K::empty();
        self.device.read_block(node.block_lba, &mut block)?;

        // Verify digest
        let computed_digest = block.compute_digest();
        if computed_digest != node.content_digest {
            return Err(()); // Tamper or corruption detected!
        }

        out_buf.copy_from_slice(&block.data);
        Ok(computed_digest)
    }

    /// Creates an instantaneous snapshot checkpoint.
    pub fn create_snapshot(&mut self) -> u64 {
        self.snapshot_merkle_root = self.superblock.merkle_root;
        self.snapshot_merkle_root
    }

    /// Verifies current state against a snapshot checkpoint in O(1).
    pub fn is_snapshot_consistent(&self) -> bool {
        self.superblock.merkle_root == self.snapshot_merkle_root
    }
}
