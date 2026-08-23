OUTPUT_ARCH(riscv)
ENTRY(_start)

/* OpenSBI (fw_dynamic) jumps here in S-mode on both QEMU virt and
   VisionFive 2. */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(4096);
    __boot_stack_bottom = .;
    . += 0x10000; /* 64 KiB boot stack */
    __boot_stack_top = .;
}
