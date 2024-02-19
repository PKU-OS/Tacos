#[repr(C)]
pub struct Property {
    pub len: u32,
    pub nameoff: u32,
}

u32_array_from_raw_big_endian!(Property);
