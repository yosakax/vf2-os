use super::base::{ecall, SbiError};
use super::{EXT_TIME, TIME_SET_TIMER};

pub fn set_timer(time: u64) -> Result<(), SbiError> {
    let ret = ecall(EXT_TIME, TIME_SET_TIMER, time as usize, 0, 0, 0, 0, 0);

    if ret.error == SbiError::Success {
        Ok(())
    } else {
        Err(ret.error)
    }
}
