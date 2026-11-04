//! Zero-Allocation WebAssembly Linear Memory Bridge
//!
//! Provides direct byte-level interaction between the host JavaScript / WebAssembly runtime
//! and OPURE's unmanaged data structures without garbage collection or serialization overhead.

use core::cell::UnsafeCell;
use opure_hal::{Aligned64, TernaryCentroid64};

pub struct SyncUnsafeCell<T>(pub UnsafeCell<T>);
unsafe impl<T> Sync for SyncUnsafeCell<T> {}
impl<T> SyncUnsafeCell<T> {
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }
    pub fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// 64KB static linear memory transfer buffer mapped directly into WebAssembly linear memory.
#[repr(C, align(64))]
pub struct WasmStaticBridge {
    pub raw_buffer: [u8; 65536],
}

static WASM_STATIC_BRIDGE: Aligned64<SyncUnsafeCell<WasmStaticBridge>> = Aligned64(SyncUnsafeCell::new(WasmStaticBridge {
    raw_buffer: [0u8; 65536],
}));

/// Retrieves a raw pointer to the static 64KB WASM transfer buffer.
///
/// In JavaScript:
/// ```javascript
/// const ptr = wasmInstance.exports.opure_wasm_get_static_buffer_ptr();
/// const memoryView = new Uint8Array(wasmInstance.exports.memory.buffer, ptr, 65536);
/// ```
#[no_mangle]
pub extern "C" fn opure_wasm_get_static_buffer_ptr() -> *mut u8 {
    unsafe {
        let bridge = &mut *WASM_STATIC_BRIDGE.0.get();
        bridge.raw_buffer.as_mut_ptr()
    }
}

/// Reads a 64-bit pair (`pos_mask`, `neg_mask`) from the transfer buffer at `byte_offset`.
pub fn read_centroid_from_buffer(byte_offset: usize) -> Option<TernaryCentroid64> {
    if byte_offset + 16 > 65536 {
        return None;
    }
    unsafe {
        let bridge = &*WASM_STATIC_BRIDGE.0.get();
        let pos_bytes: [u8; 8] = bridge.raw_buffer[byte_offset..byte_offset + 8]
            .try_into()
            .ok()?;
        let neg_bytes: [u8; 8] = bridge.raw_buffer[byte_offset + 8..byte_offset + 16]
            .try_into()
            .ok()?;
        Some(TernaryCentroid64::new(
            u64::from_le_bytes(pos_bytes),
            u64::from_le_bytes(neg_bytes),
        ))
    }
}

/// Writes a 64-bit pair (`pos_mask`, `neg_mask`) into the transfer buffer at `byte_offset`.
pub fn write_centroid_to_buffer(byte_offset: usize, centroid: &TernaryCentroid64) -> bool {
    if byte_offset + 16 > 65536 {
        return false;
    }
    unsafe {
        let bridge = &mut *WASM_STATIC_BRIDGE.0.get();
        let pos_bytes = centroid.pos_mask.to_le_bytes();
        let neg_bytes = centroid.neg_mask.to_le_bytes();
        bridge.raw_buffer[byte_offset..byte_offset + 8].copy_from_slice(&pos_bytes);
        bridge.raw_buffer[byte_offset + 8..byte_offset + 16].copy_from_slice(&neg_bytes);
        true
    }
}
