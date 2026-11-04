//! Extended Page Tables (EPT) 4-Level Address Translation
//!
//! Provides hardware-assisted nested paging for guest VMs with zero dynamic allocation.

/// EPT Entry Flag Bits
pub const EPT_READ: u64 = 1 << 0;
pub const EPT_WRITE: u64 = 1 << 1;
pub const EPT_EXECUTE: u64 = 1 << 2;
pub const EPT_MEMORY_TYPE_WB: u64 = 6 << 3; // Write-back
pub const EPT_PAGE_SIZE_2MB: u64 = 1 << 7;

/// 4KB-aligned EPT Page Table containing 512 64-bit entries.
#[repr(C, align(4096))]
pub struct EptTable {
    pub entries: [u64; 512],
}

impl EptTable {
    pub const fn new() -> Self {
        Self { entries: [0u64; 512] }
    }

    pub fn clear(&mut self) {
        self.entries.fill(0);
    }
}

/// Static 4-Level EPT Structure mapping up to 1GB of Guest Physical Memory.
#[repr(C, align(4096))]
pub struct EptPagingContext {
    pub pml4: EptTable,
    pub pdpt: EptTable,
    pub pd: EptTable,
}

impl EptPagingContext {
    pub const fn new() -> Self {
        Self {
            pml4: EptTable::new(),
            pdpt: EptTable::new(),
            pd: EptTable::new(),
        }
    }

    /// Sets up 1:1 identity EPT mapping for the first 1GB of Guest RAM using 2MB large pages.
    pub fn setup_identity_mapping_1gb(&mut self, _pml4_hpa: u64, pdpt_hpa: u64, pd_hpa: u64) {
        self.pml4.clear();
        self.pdpt.clear();
        self.pd.clear();

        // Point PML4[0] to PDPT
        self.pml4.entries[0] = pdpt_hpa | EPT_READ | EPT_WRITE | EPT_EXECUTE;

        // Point PDPT[0] to PD
        self.pdpt.entries[0] = pd_hpa | EPT_READ | EPT_WRITE | EPT_EXECUTE;

        // Map 512 x 2MB pages in PD = 1024MB (1GB)
        let mut gpa = 0u64;
        let mut idx = 0;
        while idx < 512 {
            self.pd.entries[idx] = gpa
                | EPT_READ
                | EPT_WRITE
                | EPT_EXECUTE
                | EPT_PAGE_SIZE_2MB
                | EPT_MEMORY_TYPE_WB;
            gpa += 2 * 1024 * 1024; // 2MB
            idx += 1;
        }
    }

    /// Returns the EPTP (EPT Pointer) value to be loaded into the VMCS.
    pub fn ept_pointer(&self, pml4_hpa: u64) -> u64 {
        // EPTP layout: bits 11:0: flags (memory type 6=WB, page walk length 3=4-level EPT), bits 63:12: PML4 address
        (pml4_hpa & 0x000F_FFFF_FFFF_F000) | (3 << 3) | 6
    }
}
