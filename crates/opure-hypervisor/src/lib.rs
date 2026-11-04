#![no_std]
#![deny(warnings)]

//! # OPURE-HYPERVISOR: Type-1 Bare-Metal Micro-Virtualization
//!
//! Direct-to-metal Type-1 Hypervisor using Intel VT-x / AMD-V virtualization extensions.
//! Boots into Ring -0 root mode, hosts legacy operating systems (Windows/Linux) in
//! unmanaged virtual machines, intercepts exits, blocks rogue telemetry, and splices
//! crashes mid-flight via `opure-lifter`.

pub mod ept;
pub mod exit_handler;
pub mod vmx;

pub use ept::*;
pub use exit_handler::*;
pub use vmx::*;

#[cfg(test)]
mod tests {
    use super::*;
    use opure_oztae::CAP_ALL;

    #[test]
    fn test_vmcs_region_alignment_and_init() {
        let mut vmcs = VmcsRegion::new();
        assert_eq!(core::mem::size_of::<VmcsRegion>(), 4096);
        assert_eq!(core::mem::align_of::<VmcsRegion>(), 4096);

        vmcs.init(0x1F);
        assert_eq!(vmcs.revision_id, 0x1F);
        assert_eq!(vmcs.abort_indicator, 0);
    }

    #[test]
    fn test_ept_identity_mapping_1gb() {
        let mut ept = EptPagingContext::new();
        ept.setup_identity_mapping_1gb(0x1000, 0x2000, 0x3000);

        // Verify PML4 entry points to PDPT
        assert_ne!(ept.pml4.entries[0], 0);
        assert_eq!(ept.pml4.entries[0] & 0x7, EPT_READ | EPT_WRITE | EPT_EXECUTE);

        // Verify PDPT points to PD
        assert_ne!(ept.pdpt.entries[0], 0);

        // Verify first 2MB large page entry
        assert_eq!(
            ept.pd.entries[0] & (EPT_PAGE_SIZE_2MB | EPT_READ | EPT_WRITE | EPT_EXECUTE),
            EPT_PAGE_SIZE_2MB | EPT_READ | EPT_WRITE | EPT_EXECUTE
        );

        // Verify 512th entry maps 1022MB..1024MB
        let last_entry = ept.pd.entries[511];
        assert_eq!(last_entry & 0xFFE0_0000, 511 * 2 * 1024 * 1024);
    }

    #[test]
    fn test_vm_exit_cpuid_intercept_and_signature() {
        let mut hypervisor = HypervisorCore::new();
        let mut vm = GuestVmContext::new(1, CAP_ALL);
        vm.guest_rax = 0x40000000;

        let action = hypervisor.handle_vm_exit(&mut vm, VmExitReason::Cpuid, 0, 2);
        assert_eq!(action, VmExitAction::ResumeNextInstruction { delta_rip: 2 });
        assert_eq!(vm.guest_rip, 2);
        assert_eq!(vm.guest_rax, 0x40000001);
        assert_eq!(vm.guest_rbx, 0x5255504F); // "OPUR"
    }

    #[test]
    fn test_vm_exit_crash_autopatch_detour() {
        let mut hypervisor = HypervisorCore::new();
        let mut vm = GuestVmContext::new(2, CAP_ALL);
        vm.guest_rip = 0x00007FFA_DEAD_BEEF;

        let action = hypervisor.handle_vm_exit(&mut vm, VmExitReason::ExceptionOrNmi, 0, 0);
        match action {
            VmExitAction::AutoPatchedCrash { detour_target, patch_bytes } => {
                assert_eq!(detour_target, 0x00007FFA_C001_D00D);
                assert_eq!(vm.guest_rip, 0x00007FFA_C001_D00D);
                assert_eq!(patch_bytes[0], 0xE9); // JMP rel32 opcode
            }
            _ => panic!("Expected AutoPatchedCrash action"),
        }
    }
}
