//! OPURE Bare-Metal Bootloader & Stage-0 Kernel Loader (Phase 7)
//!
//! Boots OPURE OS directly on physical x86_64 hardware via UEFI or Multiboot2,
//! establishing 4-level linear page tables, Local APIC Fibonacci timers,
//! and spinning up Ring 0 Ternary NCM routing & OZTAE security enclaves without host OS.

#![no_std]
#![deny(warnings)]

pub mod apic;
pub mod entry;
pub mod multiboot2;
pub mod paging;
pub mod uefi;

pub use apic::{ApicFibonacciTimer, FIBONACCI_DELTAS};
pub use entry::{cpu_flags, BareMetalBoot, KernelHandoffContext};
pub use multiboot2::{DiscoveredHardwareInfo, Multiboot2Parser};
pub use paging::{LongModePaging, PageTable, PAGE_PRESENT, PAGE_WRITABLE, PAGE_HUGE_2MB};
pub use uefi::{UefiDiscoveredInfo, UefiMemoryParser};
