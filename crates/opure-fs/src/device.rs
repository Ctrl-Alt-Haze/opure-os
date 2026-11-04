//! Block Device Interface & Zero-Allocation Virtual Drive

use crate::block::FsBlock4K;

/// Trait for physical or virtual block storage devices.
pub trait BlockDevice {
    fn read_block(&self, lba: u32, buf: &mut FsBlock4K) -> Result<(), ()>;
    fn write_block(&mut self, lba: u32, buf: &FsBlock4K) -> Result<(), ()>;
    fn total_blocks(&self) -> u32;
}

/// In-memory zero-allocation virtual block device.
#[repr(C, align(4096))]
pub struct VirtualBlockDevice<const BLOCKS: usize> {
    pub blocks: [FsBlock4K; BLOCKS],
}

impl<const BLOCKS: usize> VirtualBlockDevice<BLOCKS> {
    pub const fn new() -> Self {
        Self {
            blocks: [FsBlock4K::empty(); BLOCKS],
        }
    }
}

impl<const BLOCKS: usize> BlockDevice for VirtualBlockDevice<BLOCKS> {
    fn read_block(&self, lba: u32, buf: &mut FsBlock4K) -> Result<(), ()> {
        let idx = lba as usize;
        if idx >= BLOCKS {
            return Err(());
        }
        buf.data.copy_from_slice(&self.blocks[idx].data);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &FsBlock4K) -> Result<(), ()> {
        let idx = lba as usize;
        if idx >= BLOCKS {
            return Err(());
        }
        self.blocks[idx].data.copy_from_slice(&buf.data);
        Ok(())
    }

    fn total_blocks(&self) -> u32 {
        BLOCKS as u32
    }
}
