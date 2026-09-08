#![no_std]
#![no_main]

mod arch;
mod drivers;
mod fdt;
mod memory;
mod sbi;

use core::panic::PanicInfo;

use self::drivers::uart::FALLBACK_UART_BASE;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("\npanic!");
    kprintln!("{}", _info);
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
    drivers::uart::init(FALLBACK_UART_BASE);

    // `.ok()` discards the Result: writes to the UART can't actually fail,
    // and there's no error channel to report to this early in boot anyway.
    if !fdt::check_magic(fdt_ptr) {
        panic!("FDT magic is invalid");
    }

    let Some(uart_address) = fdt::find_uart_base(fdt_ptr) else {
        panic!("UART not found in FDT");
    };

    drivers::uart::init(uart_address);

    kprintln!("uart_address found: {:#x}", uart_address);
    kprintln!("hello from vf2 kernel!");
    loop {
        // SAFETY: wfi just halts the hart until an interrupt; always safe.
        unsafe { core::arch::asm!("wfi") };
    }
}
