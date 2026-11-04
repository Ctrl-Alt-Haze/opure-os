#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

#[cfg(target_os = "uefi")]
use core::panic::PanicInfo;

#[cfg(target_os = "uefi")]
#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: usize, _system_table: usize) -> usize {
    let _ = opure_boot::BareMetalBoot::spin_up_ring0_subsystems(
        16 * 1024 * 1024 * 1024,
        0x2000_0000,
    );
    0 // EFI_SUCCESS
}

#[cfg(target_os = "uefi")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(not(target_os = "uefi"))]
fn main() {
    // Verification stub for non-UEFI host targets
    let (handoff, router, agent, _) = opure_boot::BareMetalBoot::spin_up_ring0_subsystems(
        16 * 1024 * 1024 * 1024,
        0x2000_0000,
    );
    println!("[OPURE-BOOT] Host EFI test harness initialized. Usable RAM: {} MB", handoff.total_usable_ram_bytes / (1024 * 1024));
    let test_vec = [10i8; 64];
    let (intent, score, _) = router.route_ternary(&test_vec);
    println!("[OPURE-BOOT] Ring 0 NCM router validated: {:?} (Score: {}) | Agent: {}", intent, score, agent.agent_id);
}
