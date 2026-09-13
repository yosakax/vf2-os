use core::arch::global_asm;

use crate::{kprintln, timer};

global_asm!(
    r#"
    .section .text.trap
    .align 2
    .global __trap_entry
__trap_entry:
    addi sp, sp, -288

    sd zero,   0(sp)
    sd ra,     8(sp)
    sd t0,    40(sp)
    addi t0, sp, 288
    sd t0,    16(sp)
    sd gp,    24(sp)
    sd tp,    32(sp)
    sd t1,    48(sp)
    sd t2,    56(sp)
    sd s0,    64(sp)
    sd s1,    72(sp)
    sd a0,    80(sp)
    sd a1,    88(sp)
    sd a2,    96(sp)
    sd a3,   104(sp)
    sd a4,   112(sp)
    sd a5,   120(sp)
    sd a6,   128(sp)
    sd a7,   136(sp)
    sd s2,   144(sp)
    sd s3,   152(sp)
    sd s4,   160(sp)
    sd s5,   168(sp)
    sd s6,   176(sp)
    sd s7,   184(sp)
    sd s8,   192(sp)
    sd s9,   200(sp)
    sd s10,  208(sp)
    sd s11,  216(sp)
    sd t3,   224(sp)
    sd t4,   232(sp)
    sd t5,   240(sp)
    sd t6,   248(sp)

    csrr t0, sstatus
    sd t0, 256(sp)
    csrr t0, sepc
    sd t0, 264(sp)
    csrr t0, scause
    sd t0, 272(sp)
    csrr t0, stval
    sd t0, 280(sp)

    mv a0, sp
    call handle_trap

    ld t0, 264(sp)
    csrw sepc, t0
    ld t0, 256(sp)
    csrw sstatus, t0

    ld ra,     8(sp)
    ld gp,    24(sp)
    ld tp,    32(sp)
    ld t0,    40(sp)
    ld t1,    48(sp)
    ld t2,    56(sp)
    ld s0,    64(sp)
    ld s1,    72(sp)
    ld a0,    80(sp)
    ld a1,    88(sp)
    ld a2,    96(sp)
    ld a3,   104(sp)
    ld a4,   112(sp)
    ld a5,   120(sp)
    ld a6,   128(sp)
    ld a7,   136(sp)
    ld s2,   144(sp)
    ld s3,   152(sp)
    ld s4,   160(sp)
    ld s5,   168(sp)
    ld s6,   176(sp)
    ld s7,   184(sp)
    ld s8,   192(sp)
    ld s9,   200(sp)
    ld s10,  208(sp)
    ld s11,  216(sp)
    ld t3,   224(sp)
    ld t4,   232(sp)
    ld t5,   240(sp)
    ld t6,   248(sp)
    ld sp,    16(sp)
    sret
"#
);

#[repr(C)]
pub struct TrapFrame {
    pub x: [usize; 32],
    pub sstatus: usize, // supervisor status register
    pub sepc: usize,    // supervisor exception program register
    // trap の原因
    // 最上位bitは割り込みか例外かを表し、
    // bit63をinterrupt flagとして扱う
    pub scause: usize,
    pub stval: usize, // supervisor trap value
}

#[inline]
pub fn write_stvec(address: usize) {
    unsafe {
        // supervisor traps
        // stvecの値をCSRからaddress変数に読み込む
        core::arch::asm!(
        "csrw stvec, {}",
        in(reg) address
        );
    }
}

const INTERRUPT_FLAG: usize = 1usize << 63;
const SUPERVISOR_TIMER_INTERRUPT: usize = 5;

fn is_supervisor_timer_interrupt(scause: usize) -> bool {
    scause & INTERRUPT_FLAG != 0 && (scause & !INTERRUPT_FLAG) == SUPERVISOR_TIMER_INTERRUPT
}

unsafe extern "C" {
    fn __trap_entry();
}

pub fn init() {
    write_stvec(__trap_entry as *const () as usize);
}

#[no_mangle]
pub extern "C" fn handle_trap(frame: &mut TrapFrame) {
    if is_supervisor_timer_interrupt(frame.scause) {
        timer::on_tick();
        return;
    }

    kprintln!("unhandled trap: scause = {:#x}", frame.scause);
    loop {
        unsafe { core::arch::asm!("wfi") };
    }
}

/// CSRによる割り込み制御
const SIE_STIE: usize = 1 << 5;
const SSTATUS_SIE: usize = 1 << 1;

#[inline]
pub fn enable_supervisor_timer_interrupt() {
    unsafe {
        // supervisor timerを有効化
        core::arch::asm!("csrs sie, {}", in(reg) SIE_STIE);
        // stvecを初期化した後supervisor global 割り込みを有効化
        core::arch::asm!("csrs sstatus, {}", in(reg) SSTATUS_SIE);
    }
}
