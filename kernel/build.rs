fn main() {
    // Link against our custom linker script so the kernel is placed at the
    // address OpenSBI jumps to in S-mode on QEMU virt / VisionFive 2.
    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}
