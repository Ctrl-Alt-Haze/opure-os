//! Bare-Metal UEFI System Table & Memory Map Parser
//!
//! Provides zero-allocation parsing of UEFI 64-bit Boot Services memory descriptors
//! and Graphics Output Protocol (GOP) linear framebuffer parameters.

#![deny(warnings)]

pub const EFI_SYSTEM_TABLE_SIGNATURE: u64 = 0x5453595320494249; // "IBI SYST"

pub const EFI_RESERVED_MEMORY_TYPE: u32   = 0;
pub const EFI_LOADER_CODE: u32            = 1;
pub const EFI_LOADER_DATA: u32            = 2;
pub const EFI_BOOT_SERVICES_CODE: u32     = 3;
pub const EFI_BOOT_SERVICES_DATA: u32     = 4;
pub const EFI_RUNTIME_SERVICES_CODE: u32  = 5;
pub const EFI_RUNTIME_SERVICES_DATA: u32  = 6;
pub const EFI_CONVENTIONAL_MEMORY: u32    = 7;
pub const EFI_UNUSABLE_MEMORY: u32        = 8;
pub const EFI_ACPI_RECLAIM_MEMORY: u32    = 9;

/// UEFI Table Header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EfiTableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}

/// UEFI Memory Descriptor (EFI_MEMORY_DESCRIPTOR)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EfiMemoryDescriptor {
    pub type_: u32,
    pub pad: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

/// Discovered UEFI Hardware Information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UefiDiscoveredInfo {
    pub total_usable_ram_bytes: u64,
    pub conventional_pages_count: u64,
    pub highest_physical_addr: u64,
    pub descriptors_count: usize,
}

impl UefiDiscoveredInfo {
    pub const fn empty() -> Self {
        Self {
            total_usable_ram_bytes: 0,
            conventional_pages_count: 0,
            highest_physical_addr: 0,
            descriptors_count: 0,
        }
    }
}

/// Zero-Allocation UEFI Memory Map Analyzer
pub struct UefiMemoryParser;

impl UefiMemoryParser {
    /// Parse a contiguous array of `EfiMemoryDescriptor` entries over a static buffer.
    pub fn parse_memory_map(
        descriptors_bytes: &[u8],
        descriptor_size: usize,
        descriptor_version: u32,
    ) -> Result<UefiDiscoveredInfo, ()> {
        let _ = descriptor_version;
        let min_desc_size = core::mem::size_of::<EfiMemoryDescriptor>();
        if descriptor_size < min_desc_size || descriptors_bytes.is_empty() {
            return Err(());
        }

        let mut info = UefiDiscoveredInfo::empty();
        let total_bytes = descriptors_bytes.len();
        let mut offset = 0;

        while offset + descriptor_size <= total_bytes {
            let desc = unsafe {
                &*(descriptors_bytes.as_ptr().add(offset) as *const EfiMemoryDescriptor)
            };

            // 4KB per EFI page
            let region_size = desc.number_of_pages.saturating_mul(4096);
            let end_paddr = desc.physical_start.saturating_add(region_size);

            if desc.type_ == EFI_CONVENTIONAL_MEMORY || desc.type_ == EFI_LOADER_DATA {
                info.total_usable_ram_bytes = info.total_usable_ram_bytes.saturating_add(region_size);
                info.conventional_pages_count = info.conventional_pages_count.saturating_add(desc.number_of_pages);
                if end_paddr > info.highest_physical_addr {
                    info.highest_physical_addr = end_paddr;
                }
            }

            info.descriptors_count += 1;
            offset += descriptor_size;
        }

        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uefi_mmap_parse() {
        let desc_size = core::mem::size_of::<EfiMemoryDescriptor>();
        let mut buffer = [0u8; 128];

        let desc1 = EfiMemoryDescriptor {
            type_: EFI_CONVENTIONAL_MEMORY,
            pad: 0,
            physical_start: 0x1000_0000,
            virtual_start: 0,
            number_of_pages: 1024, // 4MB
            attribute: 0xF,
        };

        let desc2 = EfiMemoryDescriptor {
            type_: EFI_RESERVED_MEMORY_TYPE,
            pad: 0,
            physical_start: 0x2000_0000,
            virtual_start: 0,
            number_of_pages: 512,
            attribute: 0,
        };

        unsafe {
            let ptr = buffer.as_mut_ptr() as *mut EfiMemoryDescriptor;
            *ptr = desc1;
            *ptr.add(1) = desc2;
        }

        let info = UefiMemoryParser::parse_memory_map(&buffer[0..desc_size * 2], desc_size, 1)
            .expect("UEFI parse failed");

        assert_eq!(info.descriptors_count, 2);
        assert_eq!(info.conventional_pages_count, 1024);
        assert_eq!(info.total_usable_ram_bytes, 1024 * 4096);
        assert_eq!(info.highest_physical_addr, 0x1000_0000 + 1024 * 4096);
    }
}
