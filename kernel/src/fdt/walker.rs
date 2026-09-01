use crate::{drivers, uprintln};
use core::fmt::Write;

pub fn find_uart_base(fdt_ptr: usize) -> Option<bool> {
    let header = FdtHeader::new(fdt_ptr);
    writeln!(
        drivers::uart::Uart,
        "off_dt_struct = {:#0x}",
        header.off_dt_struct
    )
    .ok();
    header.show();

    Some(header.magic == 0xD00DFEED)
}

#[repr(C)]
pub struct FdtHeader {
    magic: u32,         // magic number (必ず0xD00DFEED)
    total_size: u32,    // DTB全体のサイズ(bytes)
    off_dt_struct: u32, // structure block の開始オフセット
    off_dt_string: u32, // strings blockの開始オフセット
    off_memrsvmap: u32, // memory reservation block の開始オフセット
    version: u32,       // format version (通常17)
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

impl FdtHeader {
    pub fn new(fdt_ptr: usize) -> Self {
        let var_name = fdt_ptr as *const FdtHeader;
        let mut raw_header = unsafe { &*var_name };
        let magic = u32::from_be(raw_header.magic);
        let total_size = u32::from_be(raw_header.total_size);
        let off_dt_struct = u32::from_be(raw_header.off_dt_struct);
        let off_dt_string = u32::from_be(raw_header.off_dt_string);
        let off_memrsvmap = u32::from_be(raw_header.off_memrsvmap);
        let version = u32::from_be(raw_header.version);
        let last_comp_version = u32::from_be(raw_header.last_comp_version);
        let boot_cpuid_phys = u32::from_be(raw_header.boot_cpuid_phys);
        let size_dt_strings = u32::from_be(raw_header.size_dt_strings);
        let size_dt_struct = u32::from_be(raw_header.size_dt_struct);

        Self {
            magic,
            total_size,
            off_dt_struct,
            off_dt_string,
            off_memrsvmap,
            version,
            last_comp_version,
            boot_cpuid_phys,
            size_dt_strings,
            size_dt_struct,
        }
    }

    pub fn show(&self) {
        uprintln!("magic = {:#x}", self.magic);
        uprintln!("total_size = {}", self.total_size);
        uprintln!("off_dt_struct = {:#x}", self.off_dt_struct);
        uprintln!("off_dt_string = {:#x}", self.off_dt_string);
        uprintln!("off_memrsvmap = {:#x}", self.off_memrsvmap);
        uprintln!("version = {}", self.version);
        uprintln!("last_comp_version = {}", self.last_comp_version);
        uprintln!("boot_cpuid_phys = {}", self.boot_cpuid_phys);
        uprintln!("size_dt_strings = {}", self.size_dt_strings);
        uprintln!("size_dt_struct = {}", self.size_dt_struct);
    }
}
