//! Minimal NS16550A-compatible UART driver.
//!
//! TODO: obtain from FDT. This address is QEMU virt's hardcoded early-boot
//! UART; per the UART Policy in AGENTS.md, long-term this must be replaced
//! by runtime discovery via the device tree so the same driver also works
//! on VisionFive 2 (JH7110).
use core::fmt::{self, Write};
const UART_BASE: usize = 0x1000_0000;

/// Line Status Register offset; bit 5 (THRE) indicates the transmit holding
/// register is empty and ready to accept a new byte.
const LSR_OFFSET: usize = 5;
const LSR_THRE: u8 = 1 << 5;

pub fn init() {
    // QEMU's ns16550 model transmits without needing explicit line/baud
    // configuration, so there is nothing to do here yet.
}

pub fn putchar(c: u8) {
    unsafe {
        // UART MMIO read: poll Line Status Register until THR is empty.
        while (UART_BASE as *const u8).add(LSR_OFFSET).read_volatile() & LSR_THRE == 0 {}
        // UART MMIO write: transmit holding register (THR) at offset 0.
        (UART_BASE as *mut u8).write_volatile(c);
    }
}

pub fn puts(s: &str) {
    for b in s.bytes() {
        putchar(b);
    }
}

/// Zero-sized handle used to format values onto the UART via `core::fmt`
/// (e.g. `write!`/`writeln!`) without requiring an allocator.
pub struct Uart;

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        puts(s);
        Ok(())
    }
}

pub struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // drivers::uart::puts に委譲する（unsafe は puts 側で扱う想定）
        crate::drivers::uart::puts(s);
        Ok(())
    }
}

/// uart 用の print マクロ群
#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut w = crate::drivers::uart::UartWriter;
        // ignore errors in early-boot environment
        let _ = write!(&mut w, $($arg)*);
    }};
}

#[macro_export]
macro_rules! uprintln {
    () => {
        $crate::uprint!("\n")
    };
    ($fmt:expr) => {
        $crate::uprint!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::uprint!(concat!($fmt, "\n"), $($arg)*)
    };
}
