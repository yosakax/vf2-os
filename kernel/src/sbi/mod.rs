mod base;
mod reset;
mod timer;
pub use base::get_spec_version;
pub use reset::set_reset;
pub use timer::set_timer;

pub const EXT_BASE: usize = 0x10;
pub const EXT_TIME: usize = 0x5449_4d45;
pub const EXT_SRST: usize = 0x5352_5354;

pub const BASE_GET_SPEC_VERSION: usize = 0;
pub const TIME_SET_TIMER: usize = 0;
pub const SRST_SYSTEM_RESET: usize = 0;
