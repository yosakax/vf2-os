use super::{BASE_GET_SPEC_VERSION, EXT_BASE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SbiError {
    Success,
    Failed, // -1
    NotSupported,
    InvalidParam,
    Denied,
    InvalidAddress,
    AlreadyAvailable,
    AlreadyStarted,
    AlreadyStopped,
    NoShmem, // Shared memory not available
    InvalidState,
    BadRange,
    TimeOut,
    ErrIo,
    DeniedLocked,
    Unknown(isize),
}

impl SbiError {
    pub const fn from_raw(error: isize) -> Self {
        match error {
            0 => Self::Success,
            -1 => Self::Failed,
            -2 => Self::NotSupported,
            -3 => Self::InvalidParam,
            -4 => Self::Denied,
            -5 => Self::InvalidAddress,
            -6 => Self::AlreadyAvailable,
            -7 => Self::AlreadyStarted,
            -8 => Self::AlreadyStopped,
            -9 => Self::NoShmem,
            -10 => Self::InvalidState,
            -11 => Self::BadRange,
            -12 => Self::TimeOut,
            -13 => Self::ErrIo,
            -14 => Self::DeniedLocked,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct SbiRet {
    pub error: SbiError,
    pub value: usize,
}

#[inline(always)]
///  OpenSBIのecallの汎用関数
///
/// * `extension`:
/// * `function`:
/// * `arg0`:
/// * `arg1`:
/// * `arg2`:
pub fn ecall(
    extension: usize,
    function: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> SbiRet {
    let error: isize;
    let value: usize;

    unsafe {
        core::arch::asm!("ecall",
        inlateout("a0") arg0 as isize => error,
        inlateout("a1") arg1 as usize => value,
        in("a2") arg2,
        in("a3") arg3,
        in("a4") arg4,
        in("a5") arg5,
        in("a6") function,
        in("a7") extension
        );
    }
    SbiRet {
        error: SbiError::from_raw(error),
        value,
    }
}

pub fn get_spec_version() -> Result<usize, SbiError> {
    let ret = ecall(EXT_BASE, BASE_GET_SPEC_VERSION, 0, 0, 0, 0, 0, 0);
    if ret.error == SbiError::Success {
        Ok(ret.value)
    } else {
        Err(ret.error)
    }
}
