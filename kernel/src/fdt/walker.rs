use crate::{drivers, kprint, kprintln};
use core::fmt::Write;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FdtToken {
    BeginNode = 0x1,
    EndNode = 0x2,
    Prop = 0x3,
    Nop = 0x4,
    End = 0x9,
}

impl FdtToken {
    fn from_raw(value: u32) -> Option<Self> {
        match value {
            0x1 => Some(Self::BeginNode),
            0x2 => Some(Self::EndNode),
            0x3 => Some(Self::Prop),
            0x4 => Some(Self::Nop),
            0x9 => Some(Self::End),
            _ => None,
        }
    }
}

pub fn find_uart_base(fdt_ptr: usize) -> Option<bool> {
    let header = FdtHeader::new(fdt_ptr);
    writeln!(
        drivers::uart::Uart,
        "off_dt_struct = {:#0x}",
        header.off_dt_struct
    )
    .ok();
    header.show();
    let struct_base = (fdt_ptr + header.off_dt_struct as usize) as *const u8;
    let token = read_be32(struct_base);
    kprintln!("first token = {:#x}", token);
    let mut p = struct_base;
    while true {
        let token = FdtToken::from_raw(read_be32(p))?;
        kprintln!("{:#010x}", token as u32);
        match token {
            FdtToken::BeginNode => {
                p = unsafe { p.add(4) };
                p = skip_cstr(p);
                p = align4(p);
            }
            FdtToken::EndNode => {
                p = unsafe { p.add(4) };
            }
            FdtToken::Prop => {
                let len = read_be32(unsafe { p.add(4) }) as usize;
                let nameoff = read_be32(unsafe { p.add(8) }) as usize;
                let strings_base = (fdt_ptr + header.off_dt_string as usize) as *const u8;
                let name_ptr = unsafe { strings_base.add(nameoff) };
                let prop_name = cstr_bytes(name_ptr);
                // kprintln!("len = {}", len);
                kprint!("prop_name = ");
                for b in prop_name.iter() {
                    kprint!("{}", *b as char);
                }
                kprintln!();
                let value_ptr = unsafe { p.add(12) };
                if len > 0 {
                    let value_name = cstr_bytes(value_ptr);
                    kprint!("value_name = ");
                    for b in value_name.iter() {
                        kprint!("{}", *b as char);
                    }
                    kprintln!();
                }
                p = unsafe { value_ptr.add(len) };
                p = align4(p);
            }

            FdtToken::Nop => {
                p = unsafe { p.add(4) };
            }
            FdtToken::End => {
                break;
            }
        }
    }

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
        let raw_header = unsafe { &*var_name };

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
        kprintln!("magic = {:#x}", self.magic);
        kprintln!("total_size = {}", self.total_size);
        kprintln!("off_dt_struct = {:#x}", self.off_dt_struct);
        kprintln!("off_dt_string = {:#x}", self.off_dt_string);
        kprintln!("off_memrsvmap = {:#x}", self.off_memrsvmap);
        kprintln!("version = {}", self.version);
        kprintln!("last_comp_version = {}", self.last_comp_version);
        kprintln!("boot_cpuid_phys = {}", self.boot_cpuid_phys);
        kprintln!("size_dt_strings = {}", self.size_dt_strings);
        kprintln!("size_dt_struct = {}", self.size_dt_struct);
    }
}

fn read_be32(ptr: *const u8) -> u32 {
    unsafe { u32::from_be_bytes([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)]) }
}

fn cstr_len(ptr: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    len
}
fn cstr_bytes<'a>(ptr: *const u8) -> &'a [u8] {
    let len = cstr_len(ptr);
    unsafe { core::slice::from_raw_parts(ptr, len) }
}

fn align4(ptr: *const u8) -> *const u8 {
    let addr = ptr as usize;
    ((addr + 3) & !3) as *const u8
}

fn skip_cstr(mut ptr: *const u8) -> *const u8 {
    unsafe {
        while *ptr != 0 {
            ptr = ptr.add(1);
        }
        // NUL文字も飛ばす
        ptr.add(1)
    }
}
