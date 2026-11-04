#![no_std]
#![deny(warnings)]

//! # OPURE-FS: Deterministic Morton Z-Curve Content-Addressed File System
//!
//! Zero-allocation storage substrate for OPURE OS:
//! - Content-addressed 4KB physical blocks with FNV/Merkle cryptographic digests.
//! - 24-bit 3D Morton Z-Order spatial block indexing for sub-microsecond seek locality.
//! - Zero heap allocation throughout all read/write paths.
//! - Instant O(1) Merkle snapshot checkpoints and integrity validation.

pub mod block;
pub mod device;
pub mod file_system;
pub mod morton_tree;

pub use block::*;
pub use device::*;
pub use file_system::*;
pub use morton_tree::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superblock_format_and_validation() {
        let dev = VirtualBlockDevice::<16>::new();
        let fs = OpureFileSystem::<_, 16>::format(dev).expect("format failed");
        assert!(fs.superblock.is_valid());
        assert_eq!(fs.superblock.total_blocks, 16);
        assert_eq!(fs.superblock.free_blocks, 15);
    }

    #[test]
    fn test_morton_encoding_and_seek() {
        let key1 = MortonBlockNode::encode_morton(1, 2, 3);
        let key2 = MortonBlockNode::encode_morton(1, 2, 4);
        assert_ne!(key1, key2);

        let mut index = MortonBlockIndex::<16>::new();
        let node1 = MortonBlockNode::new(key1, 10, 0xAAAA);
        let node2 = MortonBlockNode::new(key2, 11, 0xBBBB);

        assert!(index.insert(node2).is_ok());
        assert!(index.insert(node1).is_ok());

        // Binary search retrieval
        let found = index.lookup(key1).expect("not found");
        assert_eq!(found.block_lba, 10);
        assert_eq!(found.content_digest, 0xAAAA);
    }

    #[test]
    fn test_content_addressed_read_write_and_snapshot() {
        let dev = VirtualBlockDevice::<16>::new();
        let mut fs = OpureFileSystem::<_, 16>::format(dev).expect("format failed");

        let mut sample_data = [0u8; FS_BLOCK_SIZE];
        sample_data[0..13].copy_from_slice(b"Hello OPUREFS");

        let (morton_key, digest) = fs.write_morton_block(5, 10, 2, &sample_data)
            .expect("write failed");
        assert_ne!(digest, 0);

        let mut read_buf = [0u8; FS_BLOCK_SIZE];
        let read_digest = fs.read_morton_block(morton_key, &mut read_buf)
            .expect("read failed");

        assert_eq!(read_digest, digest);
        assert_eq!(&read_buf[0..13], b"Hello OPUREFS");

        // Create snapshot
        let snap = fs.create_snapshot();
        assert_eq!(snap, fs.superblock.merkle_root);
        assert!(fs.is_snapshot_consistent());
    }
}
