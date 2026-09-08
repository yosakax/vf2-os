//! Minimal flattened device tree (FDT) header inspection.
//!
//! This only validates the magic number for Milestone 1. Full parsing
//! (memory layout, interrupt controller, timer, UART node lookup) will be
//! added when device discovery is implemented.

mod walker;
pub use walker::find_uart_base;

const FDT_MAGIC: u32 = 0xd00d_feed;

/// Reads the FDT header magic at `fdt_ptr` and checks it matches the
/// expected big-endian value defined by the Devicetree Specification.
pub fn check_magic(fdt_ptr: usize) -> bool {
    if fdt_ptr == 0 {
        return false;
    }

    // // SAFETY: fdt_ptr is provided by OpenSBI in the `a1` register per the
    // // RISC-V supervisor boot convention and is expected to point to a valid
    // // FDT blob whose first field is a 32-bit big-endian magic number.
    let magic = unsafe { core::ptr::read_volatile(fdt_ptr as *const u32) };
    u32::from_be(magic) == FDT_MAGIC
}
