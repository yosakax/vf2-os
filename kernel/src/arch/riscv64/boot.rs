use core::arch::global_asm;

// Minimal boot trampoline: OpenSBI hands control to `_start` in S-mode with
// a0 = hartid and a1 = pointer to the FDT blob. We only need to set up a
// stack before jumping into safe Rust; a0/a1 are left untouched so they are
// forwarded to `start` per the standard RISC-V calling convention.
global_asm!(
    r#"
    .section .text.entry
    .global _start
_start:
    la sp, __boot_stack_top
    call start
2:
    wfi
    j 2b
"#
);
