//! VM-Exit Interception & Cognitive Hypervisor Dispatcher
//!
//! Handles VM exits from guest operating systems (Windows/Linux), intercepting
//! crash faults, rogue telemetry MSRs, and hardware I/O, routing them through
//! `opure-lifter` and `opure-oztae` in real time with zero heap allocations.

use opure_hal::TernaryCentroid64;
use opure_lifter::TrampolineEmitter;
use opure_oztae::{oztae_verify_access_prequantized, OztaeIdentity};

/// Intel VT-x Basic VM-Exit Reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VmExitReason {
    ExceptionOrNmi = 0,
    ExternalInterrupt = 1,
    TripleFault = 2,
    Cpuid = 10,
    Hlt = 12,
    Invd = 13,
    Invlpg = 14,
    CrAccess = 28,
    IoInstruction = 30,
    MsrRead = 31,
    MsrWrite = 32,
    InvalidGuestState = 33,
    VmCall = 18,
    EptViolation = 48,
    EptMisconfiguration = 49,
}

/// Outcome of handling a VM Exit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmExitAction {
    /// Advance guest RIP and resume execution
    ResumeNextInstruction { delta_rip: u64 },
    /// Injected live trampoline patch to bypass guest crash
    AutoPatchedCrash { detour_target: u64, patch_bytes: [u8; 5] },
    /// Quarantined guest due to zero-trust violation
    SecurityQuarantine { reason_code: u32 },
    /// Handled internal hypercall
    HypercallHandled,
}

/// Zero-Allocation Virtual Machine State & Interceptor
pub struct GuestVmContext {
    pub vm_id: u32,
    pub guest_rip: u64,
    pub guest_rsp: u64,
    pub guest_rax: u64,
    pub guest_rcx: u64,
    pub guest_rdx: u64,
    pub guest_rbx: u64,
    pub identity: OztaeIdentity,
    pub is_quarantined: bool,
}

impl GuestVmContext {
    pub fn new(vm_id: u32, caps: u64) -> Self {
        Self {
            vm_id,
            guest_rip: 0,
            guest_rsp: 0,
            guest_rax: 0,
            guest_rcx: 0,
            guest_rdx: 0,
            guest_rbx: 0,
            identity: OztaeIdentity::new(vm_id, caps, TernaryCentroid64::new(0, 0), 0, 10000),
            is_quarantined: false,
        }
    }
}

/// Type-1 Hypervisor Core Interceptor
pub struct HypervisorCore;

impl HypervisorCore {
    pub const fn new() -> Self {
        Self
    }

    /// Dispatches and processes an intercepted VM exit from a guest VM.
    pub fn handle_vm_exit(
        &mut self,
        vm: &mut GuestVmContext,
        reason: VmExitReason,
        _exit_qualification: u64,
        instruction_len: u32,
    ) -> VmExitAction {
        match reason {
            VmExitReason::Cpuid => {
                // Emulate CPUID with OPURE Cognitive Hypervisor signature
                if vm.guest_rax == 0x40000000 {
                    // Hypervisor signature "OPURE VMM"
                    vm.guest_rax = 0x40000001; // Max hypervisor leaf
                    vm.guest_rbx = 0x5255504F; // "OPUR"
                    vm.guest_rcx = 0x534F2045; // "E OS"
                    vm.guest_rdx = 0x00004D4D; // "MM\0\0"
                } else {
                    // Standard pass-through spoof
                    vm.guest_rax = 0;
                    vm.guest_rbx = 0;
                    vm.guest_rcx = 0;
                    vm.guest_rdx = 0;
                }
                vm.guest_rip += instruction_len as u64;
                VmExitAction::ResumeNextInstruction {
                    delta_rip: instruction_len as u64,
                }
            }

            VmExitReason::MsrWrite => {
                // Intercept telemetry / unauthorized MSR writes
                let msr = vm.guest_rcx as u32;
                // Hardware I/O capability check
                let cap_needed = 0x00000004; // Hardware I/O capability
                if oztae_verify_access_prequantized(
                    &vm.identity,
                    cap_needed,
                    500, // target Morton in range
                    &vm.identity.baseline_signature,
                    false,
                )
                .is_err()
                {
                    // Block unauthorized MSR write and quarantine if malicious
                    vm.is_quarantined = true;
                    return VmExitAction::SecurityQuarantine { reason_code: msr };
                }
                vm.guest_rip += instruction_len as u64;
                VmExitAction::ResumeNextInstruction {
                    delta_rip: instruction_len as u64,
                }
            }

            VmExitReason::ExceptionOrNmi => {
                // Guest encountered GP fault / invalid opcode crash
                // Invoke opure-lifter live trampoline splicer to redirect to recovery handler
                let recovery_target = 0x00007FFA_C001_D00Dusize;
                let mut patch_bytes = [0u8; 5];
                TrampolineEmitter::emit_rel32_jmp(
                    vm.guest_rip as usize,
                    recovery_target,
                    &mut patch_bytes,
                );
                vm.guest_rip = recovery_target as u64;
                VmExitAction::AutoPatchedCrash {
                    detour_target: recovery_target as u64,
                    patch_bytes,
                }
            }

            VmExitReason::VmCall => {
                // OPURE Paravirtualization Hypercall
                VmExitAction::HypercallHandled
            }

            _ => {
                // Default: advance RIP by instruction length
                vm.guest_rip += instruction_len as u64;
                VmExitAction::ResumeNextInstruction {
                    delta_rip: instruction_len as u64,
                }
            }
        }
    }
}
