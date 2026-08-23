#![no_std]
#![no_main]

mod arch;
mod drivers;
mod fdt;
mod memory;
mod sbi;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // SAFETY: wfi just halts the hart until an interrupt; always safe.
        unsafe { core::arch::asm!("wfi") };
    }
}

/// Entry point called from `arch::riscv64::boot::_start` once the stack is
/// set up. `hartid` and `fdt_ptr` are forwarded from the a0/a1 registers,
/// which OpenSBI populates per the RISC-V supervisor boot convention.
#[no_mangle]
extern "C" fn start(_hartid: usize, fdt_ptr: usize) -> ! {
    drivers::uart::init();
    drivers::uart::puts("hello rust os\n");

    if fdt::check_magic(fdt_ptr) {
        drivers::uart::puts("fdt detected\n");
    }

    loop {
        // SAFETY: wfi just halts the hart until an interrupt; always safe.
        unsafe { core::arch::asm!("wfi") };
    }
}
