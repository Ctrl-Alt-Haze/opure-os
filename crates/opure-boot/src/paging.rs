//! Long-Mode 64-Bit Page Tables & Linear Physical Memory Mapping
//!
//! Configures 4-level x86_64 page tables (PML4, PDPT, PD, PT) mapping physical RAM linearly
//! into unmanaged virtual memory using 2MB huge pages.

#![deny(warnings)]

pub const PAGE_PRESENT: u64        = 1 << 0;
pub const PAGE_WRITABLE: u64       = 1 << 1;
pub const PAGE_USER: u64           = 1 << 2;
pub const PAGE_WRITE_THROUGH: u64  = 1 << 3;
pub const PAGE_CACHE_DISABLE: u64  = 1 << 4;
pub const PAGE_ACCESSED: u64       = 1 << 5;
pub const PAGE_DIRTY: u64          = 1 << 6;
pub const PAGE_HUGE_2MB: u64       = 1 << 7;
pub const PAGE_GLOBAL: u64         = 1 << 8;
pub const PAGE_NO_EXECUTE: u64     = 1 << 63;

pub const ENTRIES_PER_TABLE: usize = 512;
pub const PAGE_SIZE_2MB: u64       = 2 * 1024 * 1024;
pub const PAGE_SIZE_4KB: u64       = 4096;

/// 4KB Page Table structure (512 entries of 64-bit descriptors)
#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct PageTable {
    pub entries: [u64; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub const fn empty() -> Self {
        Self {
            entries: [0u64; ENTRIES_PER_TABLE],
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.entries = [0u64; ENTRIES_PER_TABLE];
    }
}

/// Zero-Allocation Long-Mode Page Table Harness
pub struct LongModePaging;

impl LongModePaging {
    /// Configure a linear identity + higher-half mapping for up to 4GB of physical RAM
    /// using 2MB huge pages across 1 PML4, 1 PDPT, and up to 4 Page Directories (PDs).
    pub fn setup_linear_mapping_2mb(
        pml4: &mut PageTable,
        pdpt: &mut PageTable,
        pds: &mut [PageTable],
        max_physical_bytes: u64,
        higher_half_offset: u64,
    ) -> usize {
        pml4.clear();
        pdpt.clear();

        // 1. Point PML4[0] (identity) and PML4[higher_half_idx] to PDPT
        let pdpt_flags = PAGE_PRESENT | PAGE_WRITABLE;
        let pdpt_paddr = pdpt as *const PageTable as u64;

        pml4.entries[0] = pdpt_paddr | pdpt_flags;

        let higher_pml4_idx = ((higher_half_offset >> 39) & 0x1FF) as usize;
        if higher_pml4_idx != 0 {
            pml4.entries[higher_pml4_idx] = pdpt_paddr | pdpt_flags;
        }

        // 2. Map PDs into PDPT
        let num_pds_needed = (((max_physical_bytes + 0x3FFF_FFFF) >> 30) as usize).min(pds.len());
        let pd_flags = PAGE_PRESENT | PAGE_WRITABLE;

        let mut pages_mapped = 0;
        for i in 0..num_pds_needed {
            pds[i].clear();
            let pd_paddr = &pds[i] as *const PageTable as u64;
            pdpt.entries[i] = pd_paddr | pd_flags;

            // Map 512 entries of 2MB pages per PD (1GB per PD)
            for j in 0..ENTRIES_PER_TABLE {
                let phys_addr = ((i * ENTRIES_PER_TABLE + j) as u64) * PAGE_SIZE_2MB;
                if phys_addr >= max_physical_bytes {
                    break;
                }
                pds[i].entries[j] = phys_addr | PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE_2MB;
                pages_mapped += 1;
            }
        }

        pages_mapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table_size_and_alignment() {
        assert_eq!(core::mem::size_of::<PageTable>(), 4096);
        assert_eq!(core::mem::align_of::<PageTable>(), 4096);
    }

    #[test]
    fn test_linear_paging_setup_1gb() {
        let mut pml4 = PageTable::empty();
        let mut pdpt = PageTable::empty();
        let mut pds = [PageTable::empty(); 2];

        // Map 512MB (256 pages of 2MB)
        let mapped = LongModePaging::setup_linear_mapping_2mb(
            &mut pml4,
            &mut pdpt,
            &mut pds,
            512 * 1024 * 1024,
            0xFFFF_8000_0000_0000,
        );

        assert_eq!(mapped, 256);
        assert_ne!(pml4.entries[0], 0);
        assert_ne!(pml4.entries[256], 0); // higher half index
        assert_eq!(pds[0].entries[0] & PAGE_HUGE_2MB, PAGE_HUGE_2MB);
        assert_eq!(pds[0].entries[1], PAGE_SIZE_2MB | PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE_2MB);
    }
}
