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
  `riscv64-*-linux-gnu-` GCC toolchain, **not** a `riscv64-*-elf-`/newlib
  toolchain — U-Boot's `CONFIG_EFI_LOADER` build links a couple of helper
  images (`helloworld.efi`, `efi_selftest`) with `-shared`, which most
  bare-metal `riscv64-*-elf-` linkers reject (`-shared not supported`); a
  `riscv64-*-linux-gnu-` linker supports it even though the resulting
  `u-boot.bin` itself is still a freestanding/bare-metal image. (U-Boot and
  OpenSBI are C projects, not built with `cargo`.)
- **Rust OS**: loaded into RAM and started by U-Boot via `bootm`, using a
  `mkimage`-wrapped uImage — **not** `bootelf`. U-Boot's `bootelf` command
  calls the ELF entry point as a generic C function, `entry(argc, argv)`
  (see `lib/elf.c`), so `a0`/`a1` end up holding `argc`/`argv`, not the
  `hartid`/FDT-pointer pair our kernel's `start(hartid, fdt_ptr)` expects.
  `bootm`'s Linux-style boot path (`arch/riscv/lib/bootm.c:boot_jump_linux`)
  instead calls `kernel(gd->arch.boot_hart, images->ft_addr)`, i.e. exactly
  the `a0 = hartid, a1 = fdt_addr` convention OpenSBI itself uses — so
  wrapping the kernel as a `uImage` (`-O linux -T kernel`) and booting it
  with `bootm <addr> - <fdt_addr>` is the way to preserve that convention
  through U-Boot. See Milestone 2 for the concrete commands.

Direct kernel boot via QEMU's `-kernel` option (skipping U-Boot) is permitted
only for initial bring-up and low-level debugging.

All major milestones must eventually be validated through U-Boot using `bootm`,
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
riscv64-linux-gnu-gcc   # used to build U-Boot/OpenSBI (C, not cargo)
# Must be a *-linux-gnu- toolchain, not *-elf-/newlib: U-Boot's EFI_LOADER
# build needs a linker that supports `-shared` (see "Canonical Boot Flow").
mkimage                 # from U-Boot's tools/ dir; wraps the kernel as a uImage for `bootm`
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
`bootm` (uImage) per "Canonical Boot Flow", keeping direct `-kernel` boot
only for low-level bring-up/debugging.

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

- Build U-Boot for QEMU virt:

  ```bash
  make qemu-riscv64_smode_defconfig
  make CROSS_COMPILE=riscv64-linux-gnu- -j$(nproc)   # NOT riscv64-*-elf-, see "Canonical Boot Flow"
  ```

  producing `u-boot.bin`.
- Boot OpenSBI -> U-Boot in QEMU using the bundled generic firmware:

  ```bash
  qemu-system-riscv64 -machine virt -nographic -bios default -kernel u-boot.bin
  ```

  and confirm the flow reaches the U-Boot prompt.
- Do **not** use `bootelf` to start the kernel — it calls the entry point as
  `entry(argc, argv)`, not `entry(hartid, fdt_addr)`, so the kernel would
  receive the wrong values in `a0`/`a1` (see "Canonical Boot Flow"). Instead,
  wrap the kernel as a U-Boot `uImage` and use `bootm`:

  ```bash
  riscv64-linux-gnu-objcopy -O binary kernel kernel.bin
  mkimage -A riscv -O linux -T kernel -C none \
    -a 0x80200000 -e 0x80200000 -d kernel.bin uImage
  ```

  (`-a`/`-e` must match the kernel's link address, i.e. `BASE_ADDRESS` in
  `linker.ld`.)
- Preload the uImage into RAM at an address that does **not** overlap
  U-Boot's own load address (`CONFIG_TEXT_BASE`, `0x80200000` for this
  defconfig) — e.g. `0x84000000` — and boot it, passing U-Boot's own control
  FDT address through so the kernel still receives a valid FDT pointer:

  ```bash
  qemu-system-riscv64 -machine virt -nographic -bios default -kernel u-boot.bin \
    -device loader,file=uImage,addr=0x84000000
  ```

  then, at the U-Boot prompt:

  ```text
  printenv fdtcontroladdr
  bootm 0x84000000 - $fdtcontroladdr
  ```

  (a virtio-blk-backed filesystem image with `load virtio 0 ...` in place of
  `-device loader` is the alternative that also generalizes to real
  hardware).
- Expected output is unchanged from Milestone 1 (`hello rust os` /
  `fdt_ptr = 0x...` / `fdt detected`), but reached via
  OpenSBI -> U-Boot -> `bootm` instead of `-kernel`.

Once this lands, prefer the U-Boot/`bootm` flow over direct `-kernel` boot
for validating new functionality (see "Canonical Boot Flow" and "Testing").

---

## Milestone 3

Demonstrate:

- UART base address obtained from FDT
- No hardcoded UART address in generic kernel code

Planned implementation steps:

- Extend `kernel/src/fdt/mod.rs` beyond magic-number validation into a
  minimal structure-block walker: parse the FDT header (`off_dt_struct`,
  `off_dt_strings`), then walk `FDT_BEGIN_NODE` / `FDT_PROP` /
  `FDT_END_NODE` tokens to find a node whose `compatible` property contains
  `"ns16550a"` (QEMU virt) — the same walker should also recognize
  `"snps,dw-apb-uart"` so the same code path works on VisionFive 2 without
  board-specific branches. No heap allocator exists yet (see Memory
  Management Policy), so this must be written as a zero-allocation,
  `&[u8]`-slice-based parser (no `Vec`/`String`), returning plain integers
  and byte-slice comparisons only.
  - Alternative: add the no_std, allocation-free `fdt` crate as a dependency
    instead of hand-rolling the walker. This would be the project's first
    external dependency — only do this if the hand-rolled parser proves
    too error-prone, and confirm the crate builds for
    `riscv64gc-unknown-none-elf` with no `alloc` feature required.
- Respect `#address-cells` / `#size-cells` (inherited from the parent
  `/soc` node, typically `2`/`2` on both QEMU virt and JH7110) when decoding
  the `reg` property, instead of assuming a fixed cell width.
- Add a function such as `fdt::find_uart_base(fdt_ptr: usize) -> Option<usize>`
  that returns the first `reg` address of the matching node.
- Refactor `kernel/src/drivers/uart.rs` so `UART_BASE` is no longer a fixed
  `const`: change `init()` to `init(base: usize)` and store the discovered
  address in a `static AtomicUsize` (safe without an allocator or locks),
  read by `putchar`/`puts` on every call. Keep the current
  `0x1000_0000` value only as a documented fallback used if FDT discovery
  fails before the UART is otherwise usable (e.g. for early panic
  messages), clearly marked `// TODO: remove once FDT discovery is proven
  reliable on both targets`.
- Update `kernel/src/main.rs`'s boot sequence: parse the FDT
  (`fdt::find_uart_base`) immediately after `fdt::check_magic`, then call
  `drivers::uart::init(base)` with the discovered address before any
  further output.
- Validate in QEMU virt: the discovered base must equal the current
  hardcoded `0x1000_0000`, so existing output
  (`hello rust os` / `fdt_ptr = 0x...` / `fdt detected`) is unchanged — this
  is a regression check, not a new visible behavior.
- Real VisionFive 2 hardware validation (confirming the same code resolves
  the correct JH7110 UART base from its DTB) is out of scope until board
  bring-up begins; QEMU virt validation is sufficient to close this
  milestone per "Current Development Target".

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
