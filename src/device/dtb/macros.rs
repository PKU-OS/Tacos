#[macro_export]
macro_rules! u32_array_from_raw_big_endian {
    ($u32_array_struct: ident) => {
        const LENGTH: usize = core::mem::size_of::<$u32_array_struct>() / 4;
        impl $u32_array_struct {
            pub fn from_raw(base: usize) -> Self {
                let raw: &[u32; LENGTH] = unsafe { core::mem::transmute(base) };
                let data: [u32; LENGTH] = core::array::from_fn(|i| u32::from_be(raw[i]));

                unsafe { core::mem::transmute(data) }
            }
        }
    };
}
