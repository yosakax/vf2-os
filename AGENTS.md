# AGENTS.md

## Project Overview

This repository contains a hobby operating system project targeting:

- Final hardware target: StarFive VisionFive 2 (JH7110)
- ISA: RISC-V 64-bit (RV64GC)
- Language: Rust

During early development, QEMU `virt` is the primary execution environment.

The objective is to develop a small educational operating system while keeping hardware-specific assumptions to a minimum.

See "Canonical Boot Flow" below for the full boot chain this project targets.

---

## Canonical Boot Flow

The canonical development and deployment boot flow is:

```text
QEMU virt
-> OpenSBI
-> U-Boot
-> Rust OS

VisionFive 2
-> BootROM
-> OpenSBI
-> U-Boot
-> Rust OS
```

Development should aim to keep these environments as similar as possible.

- **OpenSBI**: on QEMU virt, the platform's bundled generic `fw_dynamic` image
  (`-bios default`) is sufficient — OpenSBI's `platform/generic` support
  covers `virt` via its FDT, so no custom OpenSBI build is required for QEMU.
  VisionFive 2 requires a JH7110-capable OpenSBI build (also `platform/generic`);
  this is only needed once hardware bring-up starts.
- **U-Boot**: built from upstream U-Boot source using the `qemu-riscv64_smode_defconfig`
  (S-mode config, since OpenSBI already did M-mode init) for QEMU, and
  `starfive_visionfive2_defconfig` for real hardware. Cross-compile with a
  `riscv64-*-elf-` or `riscv64-*-linux-gnu-` GCC toolchain (U-Boot and OpenSBI
  are C projects, not built with `cargo`).
- **Rust OS**: loaded and started by U-Boot via `bootelf`.

Direct kernel boot via QEMU's `-kernel` option (skipping U-Boot) is permitted
only for initial bring-up and low-level debugging.

All major milestones must eventually be validated through U-Boot using `bootelf`,
not just via direct `-kernel` boot.

---

## Development Philosophy

### Prefer Portable Code

Avoid encoding JH7110-specific addresses or assumptions in generic kernel code.

Bad:

```rust
const UART_BASE: usize = 0xXXXXXXXX;
```

Good:

```rust
let uart_base = discover_from_fdt();
```

Hardware-specific implementations should live in dedicated platform modules.

---

## Boot Assumptions

The kernel is loaded by U-Boot.

The kernel must assume:

- OpenSBI is already running
- Machine-mode firmware initialization is complete
- The kernel executes in supervisor mode

The kernel should not attempt to replace OpenSBI functionality.

---

## Current Development Target

Unless explicitly stated otherwise, all new functionality should work on:

- QEMU `virt`
- VisionFive 2

QEMU compatibility takes priority during early development.

---

## Toolchain

Target:

```text
riscv64gc-unknown-none-elf
```

Required tooling for the Rust kernel:

```bash
rustup target add riscv64gc-unknown-none-elf
cargo
rust-src
qemu-system-riscv64
```

Required tooling for the boot chain (U-Boot / OpenSBI, see "Canonical Boot Flow"):

```bash
riscv64-unknown-elf-gcc   # or riscv64-*-linux-gnu-gcc; used to build U-Boot/OpenSBI (C, not cargo)
# OpenSBI generic fw_dynamic is bundled with QEMU (-bios default); no separate build needed for QEMU.
```

---

## Repository Structure

Expected layout:

```text
kernel/
├── src/
│   ├── arch/
│   │   └── riscv64/
│   ├── drivers/
│   ├── fdt/
│   ├── memory/
│   ├── sbi/
│   └── main.rs
│
├── linker.ld
├── Cargo.toml
└── build.rs
```

---

## Architectural Priorities

Implement features in approximately this order:

1. Boot
2. UART output
3. FDT parsing
4. SBI interface
5. Timer interrupts
6. Trap handling
7. Physical memory allocator
8. Sv39 virtual memory
9. Scheduler
10. Userspace support

Avoid working on filesystems, networking, or GUI until the above items are present.

---

## Device Tree Policy

Device Tree is the primary source of hardware information.

The kernel should obtain:

- UART base address
- Memory layout
- Interrupt controller information
- Timer information

from the FDT whenever possible.

Do not hardcode values that can be sourced from FDT.

Temporary exceptions are acceptable during bring-up but should be marked:

```rust
// TODO: obtain from FDT
```

---

## UART Policy

Early boot logging may initially use the QEMU virt UART address.

However, the long-term design must support:

- QEMU virt
- VisionFive 2

through runtime device discovery.

---

## Unsafe Rust Guidelines

Unsafe code is allowed when required for:

- MMIO access
- CSR access
- Context switching
- Trap entry/exit

Every unsafe block should have a short justification comment.

Example:

```rust
unsafe {
    // UART MMIO write
    uart_dr.write_volatile(byte);
}
```

---

## Memory Management Policy

Do not allocate memory dynamically during early boot.

Initial code should rely only on:

- static variables
- stack allocation

A heap allocator may be introduced after a physical page allocator exists.

---

## Testing

Preferred workflow:

```bash
cargo build
cargo fmt
cargo clippy
cargo test
```

Runtime validation:

```bash
qemu-system-riscv64 \
  -machine virt \
  -nographic \
  -bios default \
  -kernel <kernel>
```

Until Milestone 2 (U-Boot/OpenSBI boot chain) lands, this direct `-kernel`
boot is the validation method. Afterwards, prefer booting through U-Boot's
`bootelf` per "Canonical Boot Flow", keeping direct `-kernel` boot only for
low-level bring-up/debugging.

All new functionality should be testable in QEMU before targeting VisionFive 2 hardware.

---

## Milestone 1

Successful boot in QEMU with:

- Rust kernel entry
- UART output
- "Hello World"
- FDT pointer received
- FDT magic validated

Expected output:

```text
hello rust os
fdt detected
```

---

## Milestone 2

Boot the Rust kernel through the full canonical chain in QEMU instead of a
direct `-kernel` boot:

- Build U-Boot for QEMU virt: `make qemu-riscv64_smode_defconfig`, then
  `make CROSS_COMPILE=riscv64-unknown-elf- -j$(nproc)` to produce `u-boot.bin`.
- Boot OpenSBI -> U-Boot in QEMU using the bundled generic firmware:

  ```bash
  qemu-system-riscv64 -machine virt -nographic -bios default -kernel u-boot.bin
  ```

  and confirm the flow reaches the U-Boot prompt.
- Load the Rust kernel ELF into RAM and hand off control from the U-Boot
  prompt via `bootelf`. For QEMU-only bring-up, preload the ELF with
  `-device loader,file=<kernel-elf>,addr=0x80200000,cpu-num=0` and run
  `bootelf 0x80200000` at the U-Boot prompt (a virtio-blk-backed filesystem
  image with `load virtio 0 ...` is the alternative that also generalizes to
  real hardware).
- Expected output is unchanged from Milestone 1 (`hello rust os` /
  `fdt detected`), but reached via OpenSBI -> U-Boot -> `bootelf` instead of
  `-kernel`.

Once this lands, prefer the U-Boot/`bootelf` flow over direct `-kernel` boot
for validating new functionality (see "Canonical Boot Flow" and "Testing").

---

## Milestone 3

Demonstrate:

- UART base address obtained from FDT
- No hardcoded UART address in generic kernel code

---

## Non-Goals

Until core kernel infrastructure exists, do not prioritize:

- GUI
- Graphics acceleration
- Networking stack
- USB stack
- Filesystems
- Application ecosystem

Focus on kernel fundamentals first.
