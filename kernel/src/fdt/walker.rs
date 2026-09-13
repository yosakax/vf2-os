use crate::{kprint, kprintln};

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

#[derive(Clone, Copy)]
struct NodeState {
    address_cells: usize,
    size_cells: usize,
    is_uart: bool,
    reg_ptr: *const u8,
    reg_len: usize,
    has_reg: bool,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            address_cells: 2,
            size_cells: 1,
            is_uart: false,
            reg_ptr: core::ptr::null(),
            reg_len: 0,
            has_reg: false,
        }
    }
}

const MAX_NODE_DEPTH: usize = 16;

pub fn find_uart_base(fdt_ptr: usize) -> Option<usize> {
    let header = FdtHeader::new(fdt_ptr);
    let struct_base = (fdt_ptr + header.off_dt_struct as usize) as *const u8;
    let mut p = struct_base;

    let mut stack = [NodeState::default(); MAX_NODE_DEPTH];
    let mut depth = 0;
    loop {
        let token = FdtToken::from_raw(read_be32(p))?;
        match token {
            FdtToken::BeginNode => {
                if depth >= MAX_NODE_DEPTH {
                    return None;
                }

                // NOTE: 新しいNodeを生やす
                let parent = if depth != 0 {
                    stack[depth - 1]
                } else {
                    NodeState::default()
                };

                stack[depth] = NodeState {
                    address_cells: parent.address_cells,
                    size_cells: parent.size_cells,
                    is_uart: false,
                    reg_ptr: core::ptr::null(),
                    reg_len: 0,
                    has_reg: false,
                };
                depth += 1;

                p = unsafe { p.add(4) };
                p = skip_cstr(p);
                p = align4(p);
            }
            FdtToken::EndNode => {
                if depth == 0 {
                    return None;
                }
                let node = stack[depth - 1];
                if node.is_uart && node.has_reg {
                    return decode_reg_address(
                        node.reg_ptr,
                        node.reg_len,
                        node.address_cells,
                        node.size_cells,
                    );
                }

                depth -= 1;
                p = unsafe { p.add(4) };
            }
            FdtToken::Prop => {
                let len = read_be32(unsafe { p.add(4) }) as usize;
                let nameoff = read_be32(unsafe { p.add(8) }) as usize;

                let strings_base = (fdt_ptr + header.off_dt_string as usize) as *const u8;
                let name_ptr = unsafe { strings_base.add(nameoff) };
                let prop_name = cstr_bytes(name_ptr);
                let value_ptr = unsafe { p.add(12) };

                let mut node = stack[depth - 1];

                if prop_name == b"compatible" {
                    node.is_uart |= compatible_contains_uart(value_ptr, len);
                } else if prop_name == b"reg" {
                    node.reg_ptr = value_ptr;
                    node.reg_len = len;
                    node.has_reg = true;
                } else if prop_name == b"#address-cells" {
                    let address_cells = read_be32(value_ptr) as usize;
                    node.address_cells = address_cells;
                } else if prop_name == b"#size-cells" {
                    let size_cells = read_be32(value_ptr) as usize;
                    node.size_cells = size_cells;
                }

                stack[depth - 1] = node;

                p = unsafe { value_ptr.add(len) };
                p = align4(p);
            }

            FdtToken::Nop => {
                p = unsafe { p.add(4) };
            }
            FdtToken::End => {
                // NOTE: ここにくるということは、uartを見つけられずにおわったということ。
                return None;
            }
        }
    }
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

    #[allow(dead_code)]
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

fn compatible_contains_uart(value: *const u8, len: usize) -> bool {
    const UART_COMPATIBLES: [&[u8]; 2] = [b"ns16550a", b"snps,dw-apb-uart"];

    let mut offset = 0;
    while offset < len {
        let start = offset;
        while offset < len && unsafe { *value.add(offset) } != 0 {
            offset += 1;
        }

        let compatible = unsafe { core::slice::from_raw_parts(value.add(start), offset - start) };

        if UART_COMPATIBLES.contains(&compatible) {
            return true;
        }

        if offset == len {
            break;
        }

        offset += 1;
    }

    false
}

fn decode_reg_address(
    value: *const u8,
    len: usize,
    address_cells: usize, // 1なら32bit, 2なら64bit
    size_cells: usize,    // なにこれ
) -> Option<usize> {
    if address_cells == 0 || address_cells > 2 || size_cells == 0 {
        return None;
    }

    let entry_cells = address_cells.checked_add(size_cells)?;
    let entry_len = entry_cells.checked_mul(4)?;
    if len < entry_len {
        return None;
    }

    let address = match address_cells {
        1 => read_be32(value) as u64,
        2 => {
            let high = read_be32(value) as u64;
            let low = read_be32(unsafe { value.add(4) }) as u64;
            // high low の並びの64bitで返す
            (high << 32) | low
        }
        _ => return None,
    };

    usize::try_from(address).ok()
}

pub fn find_timebase_frequency(fdt_ptr: usize) -> Option<u64> {
    let header = FdtHeader::new(fdt_ptr);
    let struct_base = (fdt_ptr + header.off_dt_struct as usize) as *const u8;
    let mut p = struct_base;
    let mut is_cpu: bool = false;
    let mut cpu_depth = 0;
    let mut depth = 0;

    loop {
        let token = FdtToken::from_raw(read_be32(p))?;
        match token {
            FdtToken::BeginNode => {
                depth += 1;
                p = unsafe { p.add(4) };
                let node_name = cstr_bytes(p);
                if node_name == b"cpus" {
                    is_cpu = true;
                    cpu_depth = depth;
                }
                p = skip_cstr(p);
                p = align4(p);
            }
            FdtToken::EndNode => {
                if is_cpu && depth == cpu_depth {
                    is_cpu = false;
                }
                depth -= 1;
                p = unsafe { p.add(4) };
            }
            FdtToken::Prop => {
                let len = read_be32(unsafe { p.add(4) }) as usize;
                let nameoff = read_be32(unsafe { p.add(8) }) as usize;

                let strings_base = (fdt_ptr + header.off_dt_string as usize) as *const u8;
                let name_ptr = unsafe { strings_base.add(nameoff) };
                let prop_name = cstr_bytes(name_ptr);
                let value_ptr = unsafe { p.add(12) };

                if prop_name == b"timebase-frequency" && is_cpu {
                    let timebase_frequency = read_be32(value_ptr);
                    return Some(timebase_frequency as u64);
                }

                p = unsafe { value_ptr.add(len) };
                p = align4(p);
            }

            FdtToken::Nop => {
                p = unsafe { p.add(4) };
            }
            FdtToken::End => {
                // NOTE: ここにくるということは、uartを見つけられずにおわったということ。
                return None;
            }
        }
    }
}
