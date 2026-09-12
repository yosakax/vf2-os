use super::base::ecall;
use super::{EXT_SRST, SRST_SYSTEM_RESET};

pub const RESET_TYPE_SHUTDOWN: usize = 0;
pub const RESET_REASON_NONE: usize = 0;

pub fn set_reset() -> ! {
    let _ = ecall(
        EXT_SRST,
        SRST_SYSTEM_RESET,
        RESET_TYPE_SHUTDOWN,
        RESET_REASON_NONE,
        0,
        0,
        0,
        0,
    );

    loop {
        // System Resetができない場合に戻ってくることがあるのでwfiする
        unsafe { core::arch::asm!("wfi") };
    }
}
