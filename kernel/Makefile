# Build and boot-chain automation for the kernel.
#
# See AGENTS.md ("Canonical Boot Flow", Milestone 2) for background on why
# the boot chain needs an objcopy -> mkimage -> bootm pipeline instead of a
# plain `cargo build`.
#
# Targets:
#   build       - cargo build (release)
#   kernel.bin  - flat binary suitable for mkimage
#   uimage      - uImage wrapper for U-Boot's `bootm`
#   qemu-kernel - direct `-kernel` boot (OpenSBI -> kernel), for quick
#                 regression checks against Milestone 1's expected output
#   qemu-uboot  - OpenSBI -> U-Boot boot, preloading the uImage for `bootm`
#                 (requires U_BOOT_BIN; see AGENTS.md Milestone 2)
#   clean       - remove build artifacts

CARGO ?= cargo
TARGET := riscv64gc-unknown-none-elf
PROFILE := release
KERNEL_ELF := target/$(TARGET)/$(PROFILE)/kernel

OBJCOPY ?= riscv64-linux-gnu-objcopy
MKIMAGE ?= mkimage

# Must match BASE_ADDRESS in linker.ld -- extracted automatically so the two
# can't silently drift apart.
LOAD_ADDR := $(shell sed -n 's/^BASE_ADDRESS = \(0x[0-9a-fA-F]*\);/\1/p' linker.ld)

# Scratch RAM address to preload the uImage at via QEMU's `-device loader`.
# Must not overlap U-Boot's own CONFIG_TEXT_BASE (0x80200000 for the
# qemu-riscv64_smode_defconfig U-Boot build).
PRELOAD_ADDR ?= 0x84000000

# Path to a locally-built U-Boot binary (see AGENTS.md Milestone 2). This
# lives outside the repo since U-Boot/OpenSBI are separate C projects, not
# built with cargo. Override with `make qemu-uboot U_BOOT_BIN=/path/to/u-boot.bin`.
U_BOOT_BIN ?= $(HOME)/repos/github.com/yosakax/vf2-os-external/u-boot/u-boot.bin

.PHONY: build uimage qemu-kernel qemu-uboot clean

build:
	$(CARGO) build --release

kernel.bin: build
	$(OBJCOPY) -O binary $(KERNEL_ELF) kernel.bin

uimage: kernel.bin
	$(MKIMAGE) -A riscv -O linux -T kernel -C none \
		-a $(LOAD_ADDR) -e $(LOAD_ADDR) -d kernel.bin uImage

qemu-kernel: build
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(KERNEL_ELF)

qemu-uboot: uimage
	@if [ ! -f "$(U_BOOT_BIN)" ]; then \
		echo "error: U-Boot binary not found at $(U_BOOT_BIN)"; \
		echo "  Build it per AGENTS.md Milestone 2, or override with U_BOOT_BIN=/path/to/u-boot.bin"; \
		exit 1; \
	fi
	@echo "At the U-Boot prompt, run:"
	@echo "  printenv fdtcontroladdr"
	@echo "  bootm $(PRELOAD_ADDR) - \$$fdtcontroladdr"
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(U_BOOT_BIN) \
		-device loader,file=uImage,addr=$(PRELOAD_ADDR),force-raw=on

clean:
	$(CARGO) clean
	rm -f kernel.bin uImage
