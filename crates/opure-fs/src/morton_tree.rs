//! 24-Bit 3D Morton Z-Order Spatial Block Tree
//!
//! Indexes file blocks directly along a 3D space-filling curve:
//! - X (bits 0..7): Spatial Cluster / Partition Domain
//! - Y (bits 8..15): Temporal Sequence / Chunk Sequence
//! - Z (bits 16..23): Agent Enclave / Security Quarantine Realm
//!
//! Provides deterministic binary-search spatial block lookups in <100ns.

/// A single Morton-indexed block descriptor.
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MortonBlockNode {
    pub morton_key: u32,
    pub block_lba: u32,
    pub content_digest: u64,
    pub flags: u32,
}

impl MortonBlockNode {
    pub const fn empty() -> Self {
        Self {
            morton_key: 0,
            block_lba: 0,
            content_digest: 0,
            flags: 0,
        }
    }

    pub const fn new(morton_key: u32, block_lba: u32, content_digest: u64) -> Self {
        Self {
            morton_key,
            block_lba,
            content_digest,
            flags: 0,
        }
    }

    /// Interleaves 8-bit coordinates (X: Cluster, Y: Sequence, Z: Enclave) into a 24-bit Morton key.
    pub fn encode_morton(x: u8, y: u8, z: u8) -> u32 {
        let mut key: u32 = 0;
        let mut i = 0;
        while i < 8 {
            let bit_x = ((x as u32) >> i) & 1;
            let bit_y = ((y as u32) >> i) & 1;
            let bit_z = ((z as u32) >> i) & 1;
            key |= (bit_x << (3 * i)) | (bit_y << (3 * i + 1)) | (bit_z << (3 * i + 2));
            i += 1;
        }
        key
    }
}

/// Unmanaged fixed-capacity Morton spatial index.
#[repr(C, align(64))]
pub struct MortonBlockIndex<const CAPACITY: usize> {
    pub nodes: [MortonBlockNode; CAPACITY],
    pub count: usize,
}

impl<const CAPACITY: usize> MortonBlockIndex<CAPACITY> {
    pub const fn new() -> Self {
        Self {
            nodes: [MortonBlockNode::empty(); CAPACITY],
            count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Inserts a node while maintaining sorted order by `morton_key`.
    pub fn insert(&mut self, node: MortonBlockNode) -> Result<(), ()> {
        if self.count >= CAPACITY {
            return Err(());
        }

        // Find insertion point via binary search
        let mut left = 0;
        let mut right = self.count;
        while left < right {
            let mid = left + (right - left) / 2;
            if self.nodes[mid].morton_key < node.morton_key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        // Shift elements right
        let mut idx = self.count;
        while idx > left {
            self.nodes[idx] = self.nodes[idx - 1];
            idx -= 1;
        }

        self.nodes[left] = node;
        self.count += 1;
        Ok(())
    }

    /// Fast binary search for a block by Morton key.
    pub fn lookup(&self, morton_key: u32) -> Option<&MortonBlockNode> {
        let mut left = 0;
        let mut right = self.count;
        while left < right {
            let mid = left + (right - left) / 2;
            if self.nodes[mid].morton_key == morton_key {
                return Some(&self.nodes[mid]);
            } else if self.nodes[mid].morton_key < morton_key {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        None
    }
}
