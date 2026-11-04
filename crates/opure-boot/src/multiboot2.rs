//! Bare-Metal Multiboot2 Specification Support
//!
//! Provides zero-allocation parsing of Multiboot2 bootloader headers and information
//! tables received in physical registers (EAX=0x36D76289, EBX/RSI=physical pointer).

#![deny(warnings)]

pub const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36D76289;
pub const MULTIBOOT2_HEADER_MAGIC: u32     = 0xE85250D6;

pub const TAG_TYPE_END: u32                = 0;
pub const TAG_TYPE_CMDLINE: u32            = 1;
pub const TAG_TYPE_BOOT_LOADER_NAME: u32   = 2;
pub const TAG_TYPE_BASIC_MEMINFO: u32      = 4;
pub const TAG_TYPE_BOOTDEV: u32            = 5;
pub const TAG_TYPE_MMAP: u32               = 6;
pub const TAG_TYPE_FRAMEBUFFER: u32        = 8;

pub const MEMORY_AVAILABLE: u32            = 1;
pub const MEMORY_RESERVED: u32             = 2;
pub const MEMORY_ACPI_RECLAIMABLE: u32     = 3;
pub const MEMORY_NVS: u32                  = 4;
pub const MEMORY_BADRAM: u32               = 5;

/// Multiboot2 Information Header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Multiboot2InfoHeader {
    pub total_size: u32,
    pub reserved: u32,
}

/// Generic Multiboot2 Tag Header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Multiboot2TagHeader {
    pub tag_type: u32,
    pub size: u32,
}

/// Multiboot2 Memory Map Entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Multiboot2MmapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    pub reserved: u32,
}

/// Multiboot2 Framebuffer Tag
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Multiboot2FramebufferTag {
    pub tag_type: u32,
    pub size: u32,
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
    pub framebuffer_type: u8,
    pub reserved: u16,
}

/// Summary of hardware resources discovered via Multiboot2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveredHardwareInfo {
    pub total_usable_ram_bytes: u64,
    pub highest_physical_addr: u64,
    pub framebuffer_paddr: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub mmap_entries_count: usize,
}

impl DiscoveredHardwareInfo {
    pub const fn empty() -> Self {
        Self {
            total_usable_ram_bytes: 0,
            highest_physical_addr: 0,
            framebuffer_paddr: 0,
            framebuffer_width: 0,
            framebuffer_height: 0,
            mmap_entries_count: 0,
        }
    }
}

/// Zero-Allocation parser for Multiboot2 Information Structure
pub struct Multiboot2Parser;

impl Multiboot2Parser {
    /// Verify the 32-bit Multiboot2 boot magic in EAX.
    #[inline(always)]
    pub fn verify_magic(magic: u32) -> bool {
        magic == MULTIBOOT2_BOOTLOADER_MAGIC
    }

    /// Parse Multiboot2 info table from a raw byte slice with zero heap allocations.
    pub fn parse_info_slice(raw_bytes: &[u8]) -> Result<DiscoveredHardwareInfo, ()> {
        if raw_bytes.len() < core::mem::size_of::<Multiboot2InfoHeader>() {
            return Err(());
        }

        let header = unsafe { &*(raw_bytes.as_ptr() as *const Multiboot2InfoHeader) };
        let total_size = header.total_size as usize;
        if total_size > raw_bytes.len() || total_size < 8 {
            return Err(());
        }

        let mut info = DiscoveredHardwareInfo::empty();
        let mut offset = 8; // skip total_size and reserved

        while offset + 8 <= total_size {
            let tag_hdr = unsafe {
                &*(raw_bytes.as_ptr().add(offset) as *const Multiboot2TagHeader)
            };

            let tag_type = tag_hdr.tag_type;
            let tag_size = tag_hdr.size as usize;

            if tag_type == TAG_TYPE_END || tag_size == 0 {
                break;
            }

            if tag_type == TAG_TYPE_MMAP && offset + tag_size <= total_size {
                let entry_size = core::mem::size_of::<Multiboot2MmapEntry>();
                let entries_offset = offset + 16; // TagHeader (8) + entry_size (4) + entry_version (4)
                let mut cur = entries_offset;

                while cur + entry_size <= offset + tag_size {
                    let entry = unsafe {
                        &*(raw_bytes.as_ptr().add(cur) as *const Multiboot2MmapEntry)
                    };

                    if entry.entry_type == MEMORY_AVAILABLE {
                        info.total_usable_ram_bytes = info.total_usable_ram_bytes.saturating_add(entry.length);
                        let end_addr = entry.base_addr.saturating_add(entry.length);
                        if end_addr > info.highest_physical_addr {
                            info.highest_physical_addr = end_addr;
                        }
                    }
                    info.mmap_entries_count += 1;
                    cur += entry_size;
                }
            } else if tag_type == TAG_TYPE_FRAMEBUFFER && offset + core::mem::size_of::<Multiboot2FramebufferTag>() <= total_size {
                let fb = unsafe {
                    &*(raw_bytes.as_ptr().add(offset) as *const Multiboot2FramebufferTag)
                };
                info.framebuffer_paddr = fb.framebuffer_addr;
                info.framebuffer_width = fb.framebuffer_width;
                info.framebuffer_height = fb.framebuffer_height;
            }

            // Tags are 8-byte aligned
            offset += (tag_size + 7) & !7;
        }

        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiboot2_magic_verification() {
        assert!(Multiboot2Parser::verify_magic(0x36D76289));
        assert!(!Multiboot2Parser::verify_magic(0x1BADB002));
    }

    #[test]
    fn test_multiboot2_synthetic_mmap_parse() {
        let mut buffer = [0u8; 128];
        // Set total size = 64
        buffer[0..4].copy_from_slice(&64u32.to_le_bytes());
        buffer[4..8].copy_from_slice(&0u32.to_le_bytes());

        // Tag 6 (MMAP) at offset 8
        buffer[8..12].copy_from_slice(&TAG_TYPE_MMAP.to_le_bytes());
        buffer[12..16].copy_from_slice(&40u32.to_le_bytes()); // size: 8 + 8 + 24 = 40
        buffer[16..20].copy_from_slice(&24u32.to_le_bytes()); // entry size: 24
        buffer[20..24].copy_from_slice(&0u32.to_le_bytes());  // entry version

        // Entry 1: base 0x100000, length 0x8000000 (128 MB), type 1 (Available)
        buffer[24..32].copy_from_slice(&0x100000u64.to_le_bytes());
        buffer[32..40].copy_from_slice(&0x8000000u64.to_le_bytes());
        buffer[40..44].copy_from_slice(&MEMORY_AVAILABLE.to_le_bytes());

        let info = Multiboot2Parser::parse_info_slice(&buffer).expect("Parse failed");
        assert_eq!(info.total_usable_ram_bytes, 0x8000000);
        assert_eq!(info.highest_physical_addr, 0x100000 + 0x8000000);
        assert_eq!(info.mmap_entries_count, 1);
    }
}
