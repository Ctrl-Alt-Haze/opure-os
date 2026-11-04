//! Intel VT-x / AMD-V Hardware Virtualization Primitives
//!
//! Provides zero-allocation VMCS (Virtual Machine Control Structure) management
//! and Ring -0 root execution context.

/// 4KB-aligned Virtual Machine Control Structure (VMCS) page.
#[repr(C, align(4096))]
pub struct VmcsRegion {
    pub revision_id: u32,
    pub abort_indicator: u32,
    pub data: [u8; 4088],
}

impl VmcsRegion {
    pub const VMX_REVISION_ID: u32 = 0x0000001F;

    pub const fn new() -> Self {
        Self {
            revision_id: Self::VMX_REVISION_ID,
            abort_indicator: 0,
            data: [0u8; 4088],
        }
    }

    /// Initializes VMCS revision header according to Intel SDM Volume 3C.
    pub fn init(&mut self, rev_id: u32) {
        self.revision_id = rev_id;
        self.abort_indicator = 0;
        self.data.fill(0);
    }
}

/// VM-Execution Pin-Based Controls
pub struct VmxPinControls;
impl VmxPinControls {
    pub const EXTERNAL_INTERRUPT_EXITING: u32 = 1 << 0;
    pub const NMI_EXITING: u32 = 1 << 3;
    pub const VIRTUAL_NMIS: u32 = 1 << 5;
    pub const PREEMPTION_TIMER: u32 = 1 << 6;
}

/// VM-Execution Primary Processor-Based Controls
pub struct VmxProcControls;
impl VmxProcControls {
    pub const INTERRUPT_WINDOW_EXITING: u32 = 1 << 2;
    pub const USE_TSC_OFFSETTING: u32 = 1 << 3;
    pub const HLT_EXITING: u32 = 1 << 7;
    pub const INVLPG_EXITING: u32 = 1 << 9;
    pub const MWAIT_EXITING: u32 = 1 << 10;
    pub const RDPMC_EXITING: u32 = 1 << 11;
    pub const RDTSC_EXITING: u32 = 1 << 12;
    pub const CR3_LOAD_EXITING: u32 = 1 << 15;
    pub const CR3_STORE_EXITING: u32 = 1 << 16;
    pub const CR8_LOAD_EXITING: u32 = 1 << 19;
    pub const CR8_STORE_EXITING: u32 = 1 << 20;
    pub const USE_TPR_SHADOW: u32 = 1 << 21;
    pub const MOV_DR_EXITING: u32 = 1 << 23;
    pub const UNCONDITIONAL_IO_EXITING: u32 = 1 << 24;
    pub const USE_IO_BITMAPS: u32 = 1 << 25;
    pub const USE_MSR_BITMAPS: u32 = 1 << 28;
    pub const SECONDARY_CONTROLS: u32 = 1 << 31;
}

/// VM-Execution Secondary Processor-Based Controls
pub struct VmxSecondaryProcControls;
impl VmxSecondaryProcControls {
    pub const VIRTUALIZE_APIC_ACCESSES: u32 = 1 << 0;
    pub const ENABLE_EPT: u32 = 1 << 1;
    pub const DESCRIPTOR_TABLE_EXITING: u32 = 1 << 2;
    pub const ENABLE_RDTSCP: u32 = 1 << 3;
    pub const VIRTUALIZE_X2APIC_MODE: u32 = 1 << 4;
    pub const ENABLE_VPID: u32 = 1 << 5;
    pub const WBINVD_EXITING: u32 = 1 << 6;
    pub const UNRESTRICTED_GUEST: u32 = 1 << 7;
    pub const ENABLE_VMFUNC: u32 = 1 << 13;
}
