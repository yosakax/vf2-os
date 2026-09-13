use crate::kprintln;
use crate::sbi::set_timer;
use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);
static INTERVAL: AtomicU64 = AtomicU64::new(0);
static NEXT_DEADLINE: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn read_time() -> u64 {
    let value: u64;
    unsafe {
        core::arch::asm!("rdtime {}", out(reg) value);
    }
    value
}

pub fn on_tick() {
    let interval = INTERVAL.load(Ordering::Relaxed);
    let ticks = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    let now = read_time();
    let mut next_deadline = NEXT_DEADLINE.load(Ordering::Relaxed) + interval;

    if next_deadline <= now {
        next_deadline += interval;
    }

    NEXT_DEADLINE.store(next_deadline, Ordering::Relaxed);

    match set_timer(next_deadline) {
        Ok(()) => {}
        Err(_error) => {
            kprintln!("SBI set_timer failed at tick {}", ticks);
            loop {
                unsafe { core::arch::asm!("wfi") };
            }
        }
    };
    if ticks % 100 == 0 {
        kprintln!("timer ticks: {}", ticks);
    }
}

pub fn init(time_frequency: u64, interval_ms: u64) -> Result<(), &'static str> {
    let interval = time_frequency
        .checked_mul(interval_ms)
        .and_then(|value| value.checked_div(1000))
        .ok_or("invalid time-base-frequency")?;

    if interval == 0 {
        return Err("timer interval is zero");
    }

    let deadline = read_time()
        .checked_add(interval)
        .ok_or("timer deadline overflow")?;

    INTERVAL.store(interval, Ordering::Relaxed);
    NEXT_DEADLINE.store(deadline, Ordering::Relaxed);

    set_timer(deadline).map_err(|_| "SBI set_timer failed")
}
