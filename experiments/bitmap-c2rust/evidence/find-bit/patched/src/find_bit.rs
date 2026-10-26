extern "C" {
    fn __sw_hweight32(w: ::core::ffi::c_uint) -> ::core::ffi::c_uint;
    fn __sw_hweight64(w: __u64) -> ::core::ffi::c_ulong;
    fn __bitmap_weight(
        bitmap: *const ::core::ffi::c_ulong,
        nbits: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_uint;
    fn get_random_u8() -> u8_0;
    fn get_random_u16() -> u16_0;
    fn get_random_u32() -> u32_0;
    fn __get_random_u32_below(ceil: u32_0) -> u32_0;
}
pub type __u8 = ::core::ffi::c_uchar;
pub type __u16 = ::core::ffi::c_ushort;
pub type __u32 = ::core::ffi::c_uint;
pub type __u64 = ::core::ffi::c_ulonglong;
pub type u8_0 = __u8;
pub type u16_0 = __u16;
pub type u32_0 = __u32;
pub type u64_0 = __u64;
pub type __kernel_ulong_t = ::core::ffi::c_ulong;
pub type __kernel_size_t = __kernel_ulong_t;
pub type bool_0 = bool;
pub type uintptr_t = usize;
pub type size_t = __kernel_size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {}
pub const BITS_PER_LONG: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const BITS_PER_BYTE: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
#[no_mangle]
pub static mut current_stack_pointer: ::core::ffi::c_ulong = 0;
#[inline(always)]
unsafe extern "C" fn variable__ffs(mut word: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong {
    let mut num: ::core::ffi::c_ulong = 0;
    if word & 0xffffffff == 0 { num += 32; word >>= 32; }
    if word & 0xffff == 0 { num += 16; word >>= 16; }
    if word & 0xff == 0 { num += 8; word >>= 8; }
    if word & 0xf == 0 { num += 4; word >>= 4; }
    if word & 0x3 == 0 { num += 2; word >>= 2; }
    if word & 0x1 == 0 { num += 1; }
    return num;
}
#[inline(always)]
unsafe extern "C" fn __fls(mut word: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong {
    let mut num: i32 = 63;
    if word & (!0u64 << 32) == 0 { num -= 32; word <<= 32; }
    if word & (!0u64 << (64 - 16)) == 0 { num -= 16; word <<= 16; }
    if word & (!0u64 << (64 - 8)) == 0 { num -= 8; word <<= 8; }
    if word & (!0u64 << (64 - 4)) == 0 { num -= 4; word <<= 4; }
    if word & (!0u64 << (64 - 2)) == 0 { num -= 2; word <<= 2; }
    if word & (!0u64 << (64 - 1)) == 0 { num -= 1; }
    return num as ::core::ffi::c_ulong;
}
#[inline]
unsafe extern "C" fn __arch_hweight32(mut w: ::core::ffi::c_uint) -> ::core::ffi::c_uint {
    return __sw_hweight32(w);
}
#[inline]
unsafe extern "C" fn __arch_hweight64(mut w: __u64) -> ::core::ffi::c_ulong {
    return __sw_hweight64(w);
}
#[inline(always)]
unsafe extern "C" fn hweight_long(mut w: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong {
    return if ::core::mem::size_of::<::core::ffi::c_ulong>() as usize == 4 as usize {
        (if 0 != 0 {
            (((w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                + (w as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int) as ::core::ffi::c_uint)
                .wrapping_add(
                    (((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 8 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int) as ::core::ffi::c_uint,
                )
                .wrapping_add(
                    ((((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                        + ((w >> 16 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int)
                        as ::core::ffi::c_uint)
                        .wrapping_add(
                            (((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                + ((w >> 16 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int)
                                as ::core::ffi::c_uint,
                        ),
                )
        } else {
            __arch_hweight32(w as ::core::ffi::c_uint)
        }) as ::core::ffi::c_ulong
    } else if 0 != 0 {
        (((w as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
            != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int
            + (w as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                != 0) as ::core::ffi::c_int) as ::core::ffi::c_uint)
            .wrapping_add(
                ((w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 8 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int) as ::core::ffi::c_uint,
            )
            .wrapping_add(
                (((w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 16 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int) as ::core::ffi::c_uint)
                    .wrapping_add(
                        ((w as ::core::ffi::c_ulonglong
                            >> 16 as ::core::ffi::c_int
                            >> 8 as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 16 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int)
                            as ::core::ffi::c_uint,
                    ),
            )
            .wrapping_add(
                (((w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
                    + (w as ::core::ffi::c_ulonglong >> 32 as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int) as ::core::ffi::c_uint)
                    .wrapping_add(
                        ((w as ::core::ffi::c_ulonglong
                            >> 32 as ::core::ffi::c_int
                            >> 8 as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 8 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int)
                            as ::core::ffi::c_uint,
                    )
                    .wrapping_add(
                        (((w as ::core::ffi::c_ulonglong
                            >> 32 as ::core::ffi::c_int
                            >> 16 as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int
                            + (w as ::core::ffi::c_ulonglong
                                >> 32 as ::core::ffi::c_int
                                >> 16 as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                                != 0) as ::core::ffi::c_int)
                            as ::core::ffi::c_uint)
                            .wrapping_add(
                                ((w as ::core::ffi::c_ulonglong
                                    >> 32 as ::core::ffi::c_int
                                    >> 16 as ::core::ffi::c_int
                                    >> 8 as ::core::ffi::c_int
                                    & (1 as ::core::ffi::c_ulonglong) << 0 as ::core::ffi::c_int
                                    != 0) as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 1 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 2 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 3 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 4 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 5 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 6 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int
                                    + (w as ::core::ffi::c_ulonglong
                                        >> 32 as ::core::ffi::c_int
                                        >> 16 as ::core::ffi::c_int
                                        >> 8 as ::core::ffi::c_int
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << 7 as ::core::ffi::c_int
                                        != 0)
                                        as ::core::ffi::c_int)
                                    as ::core::ffi::c_uint,
                            ),
                    ),
            ) as ::core::ffi::c_ulong
    } else {
        __arch_hweight64(w as __u64)
    };
}
#[inline]
unsafe extern "C" fn fns(
    mut word: ::core::ffi::c_ulong,
    mut n: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    while word != 0 && {
        let fresh0 = n;
        n = n.wrapping_sub(1);
        fresh0 != 0
    } {
        word &= word.wrapping_sub(1 as ::core::ffi::c_ulong);
    }
    return (if word != 0 {
        if 0 != 0 {
            word.trailing_zeros() as i32 as ::core::ffi::c_ulong
        } else {
            variable__ffs(word)
        }
    } else {
        BITS_PER_LONG as ::core::ffi::c_ulong
    }) as ::core::ffi::c_uint;
}
#[inline(always)]
unsafe extern "C" fn find_next_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
    mut offset: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    if 0 != 0 && size <= BITS_PER_LONG as ::core::ffi::c_ulong && size > 0 as ::core::ffi::c_ulong {
        let mut val: ::core::ffi::c_ulong = 0;
        if (offset >= size) as ::core::ffi::c_int as ::core::ffi::c_long != 0 {
            return size;
        }
        val = *addr
            & (::core::mem::size_of::<C2RustUnnamed_1>() as ::core::ffi::c_int
                as ::core::ffi::c_ulong)
                .wrapping_add(
                    ((1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                        << (8 as usize)
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                            .wrapping_sub(1 as usize)
                            .wrapping_sub(
                                ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                    < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                    as ::core::ffi::c_int as usize,
                            ))
                    .wrapping_sub(1 as ::core::ffi::c_ulong)
                    .wrapping_add(
                        (1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                            << (8 as usize)
                                .wrapping_mul(
                                    ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                )
                                .wrapping_sub(1 as usize)
                                .wrapping_sub(
                                    ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                        as ::core::ffi::c_int
                                        as usize,
                                ),
                    ) << offset
                        & ((1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                            << (8 as usize)
                                .wrapping_mul(
                                    ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                )
                                .wrapping_sub(1 as usize)
                                .wrapping_sub(
                                    ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                        as ::core::ffi::c_int
                                        as usize,
                                ))
                        .wrapping_sub(1 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                << (8 as usize)
                                    .wrapping_mul(
                                        ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                    )
                                    .wrapping_sub(1 as usize)
                                    .wrapping_sub(
                                        ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ),
                        ) >> (::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                            .wrapping_mul(BITS_PER_BYTE as usize)
                            .wrapping_sub(1 as usize)
                            .wrapping_sub((size as usize).wrapping_sub(1 as usize)),
                );
        return if val != 0 {
            if 0 != 0 {
                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
            } else {
                variable__ffs(val)
            }
        } else {
            size
        };
    }
    return _find_next_bit(addr, size, offset);
}
#[inline(always)]
unsafe extern "C" fn find_first_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    if 0 != 0 && size <= BITS_PER_LONG as ::core::ffi::c_ulong && size > 0 as ::core::ffi::c_ulong {
        let mut val: ::core::ffi::c_ulong = *addr
            & (::core::mem::size_of::<C2RustUnnamed_0>() as ::core::ffi::c_int
                as ::core::ffi::c_ulong)
                .wrapping_add(
                    ((1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                        << (8 as usize)
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                            .wrapping_sub(1 as usize)
                            .wrapping_sub(
                                ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                    < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                    as ::core::ffi::c_int as usize,
                            ))
                    .wrapping_sub(1 as ::core::ffi::c_ulong)
                    .wrapping_add(
                        (1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                            << (8 as usize)
                                .wrapping_mul(
                                    ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                )
                                .wrapping_sub(1 as usize)
                                .wrapping_sub(
                                    ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                        as ::core::ffi::c_int
                                        as usize,
                                ),
                    ) << 0 as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                            << (8 as usize)
                                .wrapping_mul(
                                    ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                )
                                .wrapping_sub(1 as usize)
                                .wrapping_sub(
                                    ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                        as ::core::ffi::c_int
                                        as usize,
                                ))
                        .wrapping_sub(1 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                << (8 as usize)
                                    .wrapping_mul(
                                        ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                    )
                                    .wrapping_sub(1 as usize)
                                    .wrapping_sub(
                                        ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ),
                        ) >> (::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                            .wrapping_mul(BITS_PER_BYTE as usize)
                            .wrapping_sub(1 as usize)
                            .wrapping_sub((size as usize).wrapping_sub(1 as usize)),
                );
        return if val != 0 {
            if 0 != 0 {
                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
            } else {
                variable__ffs(val)
            }
        } else {
            size
        };
    }
    return _find_first_bit(addr, size);
}
#[inline(always)]
unsafe extern "C" fn find_nth_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
    mut n: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    if n >= size {
        return size;
    }
    if 0 != 0 && size <= BITS_PER_LONG as ::core::ffi::c_ulong && size > 0 as ::core::ffi::c_ulong {
        let mut val: ::core::ffi::c_ulong = *addr
            & (::core::mem::size_of::<C2RustUnnamed>() as ::core::ffi::c_int
                as ::core::ffi::c_ulong)
                .wrapping_add(
                    ((1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                        << (8 as usize)
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                            .wrapping_sub(1 as usize)
                            .wrapping_sub(
                                ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                    < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                    as ::core::ffi::c_int as usize,
                            ))
                    .wrapping_sub(1 as ::core::ffi::c_ulong)
                    .wrapping_add(
                        (1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                            << (8 as usize)
                                .wrapping_mul(
                                    ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                )
                                .wrapping_sub(1 as usize)
                                .wrapping_sub(
                                    ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                        as ::core::ffi::c_int
                                        as usize,
                                ),
                    ) << 0 as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                            << (8 as usize)
                                .wrapping_mul(
                                    ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                )
                                .wrapping_sub(1 as usize)
                                .wrapping_sub(
                                    ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                        as ::core::ffi::c_int
                                        as usize,
                                ))
                        .wrapping_sub(1 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                << (8 as usize)
                                    .wrapping_mul(
                                        ::core::mem::size_of::<::core::ffi::c_ulong>() as usize
                                    )
                                    .wrapping_sub(1 as usize)
                                    .wrapping_sub(
                                        ((-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ),
                        ) >> (::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                            .wrapping_mul(BITS_PER_BYTE as usize)
                            .wrapping_sub(1 as usize)
                            .wrapping_sub((size as usize).wrapping_sub(1 as usize)),
                );
        return if val != 0 {
            fns(val, n as ::core::ffi::c_uint) as ::core::ffi::c_ulong
        } else {
            size
        };
    }
    return __find_nth_bit(addr, size, n);
}
#[inline(always)]
unsafe extern "C" fn bitmap_weight(
    mut src: *const ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    if 0 != 0 && nbits <= BITS_PER_LONG as ::core::ffi::c_uint && nbits > 0 as ::core::ffi::c_uint {
        return hweight_long(
            *src & !(0 as ::core::ffi::c_ulong)
                >> (nbits.wrapping_neg()
                    & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_uint),
        ) as ::core::ffi::c_uint;
    }
    return __bitmap_weight(src, nbits);
}
#[inline(always)]
unsafe extern "C" fn bitmap_read(
    mut map: *const ::core::ffi::c_ulong,
    mut start: ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    let mut index: size_t = (start as size_t).wrapping_div(BITS_PER_LONG as size_t);
    let mut offset: ::core::ffi::c_ulong =
        start.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_ulong);
    let mut space: ::core::ffi::c_ulong =
        (BITS_PER_LONG as ::core::ffi::c_ulong).wrapping_sub(offset);
    let mut value_low: ::core::ffi::c_ulong = 0;
    let mut value_high: ::core::ffi::c_ulong = 0;
    if (nbits == 0 || nbits > 64 as ::core::ffi::c_ulong) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return 0 as ::core::ffi::c_ulong;
    }
    if space >= nbits {
        return *map.offset(index as isize) >> offset
            & !(0 as ::core::ffi::c_ulong)
                >> (nbits.wrapping_neg()
                    & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
    }
    value_low = *map.offset(index as isize)
        & !(0 as ::core::ffi::c_ulong)
            << (start & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
    value_high = *map.offset(index.wrapping_add(1 as size_t) as isize)
        & !(0 as ::core::ffi::c_ulong)
            >> (start.wrapping_add(nbits).wrapping_neg()
                & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
    return value_low >> offset | value_high << space;
}
#[inline(always)]
unsafe extern "C" fn is_power_of_2(mut n: ::core::ffi::c_ulong) -> bool_0 {
    return n.wrapping_sub(1 as ::core::ffi::c_ulong)
        < n ^ n.wrapping_sub(1 as ::core::ffi::c_ulong);
}
#[inline]
unsafe extern "C" fn get_random_u32_below(mut ceil: u32_0) -> u32_0 {
    if 0 == 0 {
        return __get_random_u32_below(ceil);
    }
    extern "C" {
        #[link_name = "__compiletime_assert_39"]
        fn __compiletime_assert_39_0() -> !;
    }
    if ceil == 0 {
        __compiletime_assert_39_0();
    }
    if ceil <= 1 as u32_0 {
        return 0 as u32_0;
    }
    loop {
        if ceil <= (1 as ::core::ffi::c_uint) << 8 as ::core::ffi::c_int {
            let mut mult: u32_0 = ceil.wrapping_mul(get_random_u8() as u32_0);
            if (is_power_of_2(ceil as ::core::ffi::c_ulong) as ::core::ffi::c_int != 0
                || mult as u8_0 as ::core::ffi::c_uint
                    >= ((1 as u32_0) << 8 as ::core::ffi::c_int).wrapping_rem(ceil))
                as ::core::ffi::c_int as ::core::ffi::c_long
                != 0
            {
                return mult >> 8 as ::core::ffi::c_int;
            }
        } else if ceil <= (1 as ::core::ffi::c_uint) << 16 as ::core::ffi::c_int {
            let mut mult_0: u32_0 = ceil.wrapping_mul(get_random_u16() as u32_0);
            if (is_power_of_2(ceil as ::core::ffi::c_ulong) as ::core::ffi::c_int != 0
                || mult_0 as u16_0 as ::core::ffi::c_uint
                    >= ((1 as u32_0) << 16 as ::core::ffi::c_int).wrapping_rem(ceil))
                as ::core::ffi::c_int as ::core::ffi::c_long
                != 0
            {
                return mult_0 >> 16 as ::core::ffi::c_int;
            }
        } else {
            let mut mult_1: u64_0 = (ceil as u64_0).wrapping_mul(get_random_u32() as u64_0);
            if (is_power_of_2(ceil as ::core::ffi::c_ulong) as ::core::ffi::c_int != 0
                || mult_1 as u32_0 >= ceil.wrapping_neg().wrapping_rem(ceil))
                as ::core::ffi::c_int as ::core::ffi::c_long
                != 0
            {
                return (mult_1 >> 32 as ::core::ffi::c_int) as u32_0;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn _find_first_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return ({
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut val: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = size;
        idx = 0 as ::core::ffi::c_ulong;
        while idx.wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong) < sz {
            val = *addr.offset(idx as isize);
            if val != 0 {
                sz = ({
                    let mut __UNIQUE_ID_x__42: ::core::ffi::c_ulong = (idx as ::core::ffi::c_ulong)
                        .wrapping_mul(64 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (if 0 != 0 {
                                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
                            } else {
                                variable__ffs(val) as ::core::ffi::c_ulong
                            }),
                        );
                    let mut __UNIQUE_ID_y__43: ::core::ffi::c_ulong = sz as ::core::ffi::c_ulong;
                    extern "C" {
                        #[link_name = "__compiletime_assert_44"]
                        fn __compiletime_assert_44_0() -> !;
                    }
                    if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_x__42 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_y__43 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) == 0
                    {
                        __compiletime_assert_44_0();
                    }
                    if __UNIQUE_ID_x__42 < __UNIQUE_ID_y__43 {
                        __UNIQUE_ID_x__42
                    } else {
                        __UNIQUE_ID_y__43
                    }
                }) as ::core::ffi::c_ulong;
                break;
            } else {
                idx = idx.wrapping_add(1);
            }
        }
        sz
    });
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_first_bit_45: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_first_and_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return ({
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut val: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = size;
        idx = 0 as ::core::ffi::c_ulong;
        while idx.wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong) < sz {
            val = *addr1.offset(idx as isize) & *addr2.offset(idx as isize);
            if val != 0 {
                sz = ({
                    let mut __UNIQUE_ID_x__46: ::core::ffi::c_ulong = (idx as ::core::ffi::c_ulong)
                        .wrapping_mul(64 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (if 0 != 0 {
                                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
                            } else {
                                variable__ffs(val) as ::core::ffi::c_ulong
                            }),
                        );
                    let mut __UNIQUE_ID_y__47: ::core::ffi::c_ulong = sz as ::core::ffi::c_ulong;
                    extern "C" {
                        #[link_name = "__compiletime_assert_48"]
                        fn __compiletime_assert_48_0() -> !;
                    }
                    if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_x__46 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_y__47 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) == 0
                    {
                        __compiletime_assert_48_0();
                    }
                    if __UNIQUE_ID_x__46 < __UNIQUE_ID_y__47 {
                        __UNIQUE_ID_x__46
                    } else {
                        __UNIQUE_ID_y__47
                    }
                }) as ::core::ffi::c_ulong;
                break;
            } else {
                idx = idx.wrapping_add(1);
            }
        }
        sz
    });
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_first_and_bit_49: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_first_andnot_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return ({
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut val: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = size;
        idx = 0 as ::core::ffi::c_ulong;
        while idx.wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong) < sz {
            val = *addr1.offset(idx as isize) & !*addr2.offset(idx as isize);
            if val != 0 {
                sz = ({
                    let mut __UNIQUE_ID_x__50: ::core::ffi::c_ulong = (idx as ::core::ffi::c_ulong)
                        .wrapping_mul(64 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (if 0 != 0 {
                                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
                            } else {
                                variable__ffs(val) as ::core::ffi::c_ulong
                            }),
                        );
                    let mut __UNIQUE_ID_y__51: ::core::ffi::c_ulong = sz as ::core::ffi::c_ulong;
                    extern "C" {
                        #[link_name = "__compiletime_assert_52"]
                        fn __compiletime_assert_52_0() -> !;
                    }
                    if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_x__50 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_y__51 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) == 0
                    {
                        __compiletime_assert_52_0();
                    }
                    if __UNIQUE_ID_x__50 < __UNIQUE_ID_y__51 {
                        __UNIQUE_ID_x__50
                    } else {
                        __UNIQUE_ID_y__51
                    }
                }) as ::core::ffi::c_ulong;
                break;
            } else {
                idx = idx.wrapping_add(1);
            }
        }
        sz
    });
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_first_andnot_bit_53: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_first_and_and_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut addr3: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return ({
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut val: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = size;
        idx = 0 as ::core::ffi::c_ulong;
        while idx.wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong) < sz {
            val = *addr1.offset(idx as isize)
                & *addr2.offset(idx as isize)
                & *addr3.offset(idx as isize);
            if val != 0 {
                sz = ({
                    let mut __UNIQUE_ID_x__54: ::core::ffi::c_ulong = (idx as ::core::ffi::c_ulong)
                        .wrapping_mul(64 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (if 0 != 0 {
                                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
                            } else {
                                variable__ffs(val) as ::core::ffi::c_ulong
                            }),
                        );
                    let mut __UNIQUE_ID_y__55: ::core::ffi::c_ulong = sz as ::core::ffi::c_ulong;
                    extern "C" {
                        #[link_name = "__compiletime_assert_56"]
                        fn __compiletime_assert_56_0() -> !;
                    }
                    if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_x__54 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_y__55 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) == 0
                    {
                        __compiletime_assert_56_0();
                    }
                    if __UNIQUE_ID_x__54 < __UNIQUE_ID_y__55 {
                        __UNIQUE_ID_x__54
                    } else {
                        __UNIQUE_ID_y__55
                    }
                }) as ::core::ffi::c_ulong;
                break;
            } else {
                idx = idx.wrapping_add(1);
            }
        }
        sz
    });
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_first_and_and_bit_57: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_first_zero_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return ({
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut val: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = size;
        idx = 0 as ::core::ffi::c_ulong;
        while idx.wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong) < sz {
            val = !*addr.offset(idx as isize);
            if val != 0 {
                sz = ({
                    let mut __UNIQUE_ID_x__58: ::core::ffi::c_ulong = (idx as ::core::ffi::c_ulong)
                        .wrapping_mul(64 as ::core::ffi::c_ulong)
                        .wrapping_add(
                            (if 0 != 0 {
                                val.trailing_zeros() as i32 as ::core::ffi::c_ulong
                            } else {
                                variable__ffs(val) as ::core::ffi::c_ulong
                            }),
                        );
                    let mut __UNIQUE_ID_y__59: ::core::ffi::c_ulong = sz as ::core::ffi::c_ulong;
                    extern "C" {
                        #[link_name = "__compiletime_assert_60"]
                        fn __compiletime_assert_60_0() -> !;
                    }
                    if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_x__58 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                        < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                    {
                        2 as ::core::ffi::c_int
                            + (0 != 0
                                && __UNIQUE_ID_y__59 as ::core::ffi::c_longlong
                                    >= 0 as ::core::ffi::c_longlong)
                                as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                                * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                    < 4 as usize)
                                    as ::core::ffi::c_int
                    }) == 0
                    {
                        __compiletime_assert_60_0();
                    }
                    if __UNIQUE_ID_x__58 < __UNIQUE_ID_y__59 {
                        __UNIQUE_ID_x__58
                    } else {
                        __UNIQUE_ID_y__59
                    }
                }) as ::core::ffi::c_ulong;
                break;
            } else {
                idx = idx.wrapping_add(1);
            }
        }
        sz
    });
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_first_zero_bit_61: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_next_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_ulong,
    mut start: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut mask: ::core::ffi::c_ulong = 0;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = nbits;
        let mut __start: ::core::ffi::c_ulong = start;
        if !((__start >= sz) as ::core::ffi::c_int as ::core::ffi::c_long != 0) {
            mask = !(0 as ::core::ffi::c_ulong)
                << (__start & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
            idx = __start.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong);
            tmp = *addr.offset(idx as isize) & mask;
            loop {
                if !(tmp == 0) {
                    current_block = 2473556513754201174;
                    break;
                }
                if idx
                    .wrapping_add(1 as ::core::ffi::c_ulong)
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    >= sz
                {
                    current_block = 14523784380283086299;
                    break;
                }
                idx = idx.wrapping_add(1);
                tmp = *addr.offset(idx as isize);
            }
            match current_block {
                14523784380283086299 => {}
                _ => {
                    sz = ({
                        let mut __UNIQUE_ID_x__62: ::core::ffi::c_ulong = (idx
                            as ::core::ffi::c_ulong)
                            .wrapping_mul(64 as ::core::ffi::c_ulong)
                            .wrapping_add(
                                (if 0 != 0 {
                                    tmp.trailing_zeros() as i32 as ::core::ffi::c_ulong
                                } else {
                                    variable__ffs(tmp) as ::core::ffi::c_ulong
                                }),
                            );
                        let mut __UNIQUE_ID_y__63: ::core::ffi::c_ulong =
                            sz as ::core::ffi::c_ulong;
                        extern "C" {
                            #[link_name = "__compiletime_assert_64"]
                            fn __compiletime_assert_64_0() -> !;
                        }
                        if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_x__62 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_y__63 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) == 0
                        {
                            __compiletime_assert_64_0();
                        }
                        if __UNIQUE_ID_x__62 < __UNIQUE_ID_y__63 {
                            __UNIQUE_ID_x__62
                        } else {
                            __UNIQUE_ID_y__63
                        }
                    }) as ::core::ffi::c_ulong;
                }
            }
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_next_bit_65: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn __find_nth_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
    mut n: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut sz: ::core::ffi::c_ulong = size;
        let mut nr: ::core::ffi::c_ulong = n;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut w: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0 as ::core::ffi::c_ulong;
        idx = 0 as ::core::ffi::c_ulong;
        loop {
            if !(idx
                .wrapping_add(1 as ::core::ffi::c_ulong)
                .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                <= sz)
            {
                current_block = 2473556513754201174;
                break;
            }
            if idx
                .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                .wrapping_add(nr)
                >= sz
            {
                current_block = 1917311967535052937;
                break;
            }
            tmp = *addr.offset(idx as isize);
            w = hweight_long(tmp);
            if w > nr {
                current_block = 7636934148935011583;
                break;
            }
            nr = nr.wrapping_sub(w);
            idx = idx.wrapping_add(1);
        }
        match current_block {
            2473556513754201174 => {
                if sz.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_ulong) != 0 {
                    tmp = *addr.offset(idx as isize)
                        & !(0 as ::core::ffi::c_ulong)
                            >> (sz.wrapping_neg()
                                & (BITS_PER_LONG - 1 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulong);
                }
                current_block = 7636934148935011583;
            }
            _ => {}
        }
        match current_block {
            7636934148935011583 => {
                sz = idx
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    .wrapping_add(fns(tmp, nr as ::core::ffi::c_uint) as ::core::ffi::c_ulong);
            }
            _ => {}
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable___find_nth_bit_66: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn __find_nth_and_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
    mut n: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut sz: ::core::ffi::c_ulong = size;
        let mut nr: ::core::ffi::c_ulong = n;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut w: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0 as ::core::ffi::c_ulong;
        idx = 0 as ::core::ffi::c_ulong;
        loop {
            if !(idx
                .wrapping_add(1 as ::core::ffi::c_ulong)
                .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                <= sz)
            {
                current_block = 2473556513754201174;
                break;
            }
            if idx
                .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                .wrapping_add(nr)
                >= sz
            {
                current_block = 1917311967535052937;
                break;
            }
            tmp = *addr1.offset(idx as isize) & *addr2.offset(idx as isize);
            w = hweight_long(tmp);
            if w > nr {
                current_block = 14423764793082218457;
                break;
            }
            nr = nr.wrapping_sub(w);
            idx = idx.wrapping_add(1);
        }
        match current_block {
            2473556513754201174 => {
                if sz.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_ulong) != 0 {
                    tmp = *addr1.offset(idx as isize)
                        & *addr2.offset(idx as isize)
                        & !(0 as ::core::ffi::c_ulong)
                            >> (sz.wrapping_neg()
                                & (BITS_PER_LONG - 1 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulong);
                }
                current_block = 14423764793082218457;
            }
            _ => {}
        }
        match current_block {
            14423764793082218457 => {
                sz = idx
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    .wrapping_add(fns(tmp, nr as ::core::ffi::c_uint) as ::core::ffi::c_ulong);
            }
            _ => {}
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable___find_nth_and_bit_67: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn __find_nth_and_andnot_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut addr3: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
    mut n: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut sz: ::core::ffi::c_ulong = size;
        let mut nr: ::core::ffi::c_ulong = n;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut w: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0 as ::core::ffi::c_ulong;
        idx = 0 as ::core::ffi::c_ulong;
        loop {
            if !(idx
                .wrapping_add(1 as ::core::ffi::c_ulong)
                .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                <= sz)
            {
                current_block = 2473556513754201174;
                break;
            }
            if idx
                .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                .wrapping_add(nr)
                >= sz
            {
                current_block = 1917311967535052937;
                break;
            }
            tmp = *addr1.offset(idx as isize)
                & *addr2.offset(idx as isize)
                & !*addr3.offset(idx as isize);
            w = hweight_long(tmp);
            if w > nr {
                current_block = 13918561925972089111;
                break;
            }
            nr = nr.wrapping_sub(w);
            idx = idx.wrapping_add(1);
        }
        match current_block {
            2473556513754201174 => {
                if sz.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_ulong) != 0 {
                    tmp = *addr1.offset(idx as isize)
                        & *addr2.offset(idx as isize)
                        & !*addr3.offset(idx as isize)
                        & !(0 as ::core::ffi::c_ulong)
                            >> (sz.wrapping_neg()
                                & (BITS_PER_LONG - 1 as ::core::ffi::c_int)
                                    as ::core::ffi::c_ulong);
                }
                current_block = 13918561925972089111;
            }
            _ => {}
        }
        match current_block {
            13918561925972089111 => {
                sz = idx
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    .wrapping_add(fns(tmp, nr as ::core::ffi::c_uint) as ::core::ffi::c_ulong);
            }
            _ => {}
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable___find_nth_and_andnot_bit_68: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_next_and_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_ulong,
    mut start: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut mask: ::core::ffi::c_ulong = 0;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = nbits;
        let mut __start: ::core::ffi::c_ulong = start;
        if !((__start >= sz) as ::core::ffi::c_int as ::core::ffi::c_long != 0) {
            mask = !(0 as ::core::ffi::c_ulong)
                << (__start & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
            idx = __start.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong);
            tmp = *addr1.offset(idx as isize) & *addr2.offset(idx as isize) & mask;
            loop {
                if !(tmp == 0) {
                    current_block = 2473556513754201174;
                    break;
                }
                if idx
                    .wrapping_add(1 as ::core::ffi::c_ulong)
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    >= sz
                {
                    current_block = 14523784380283086299;
                    break;
                }
                idx = idx.wrapping_add(1);
                tmp = *addr1.offset(idx as isize) & *addr2.offset(idx as isize);
            }
            match current_block {
                14523784380283086299 => {}
                _ => {
                    sz = ({
                        let mut __UNIQUE_ID_x__69: ::core::ffi::c_ulong = (idx
                            as ::core::ffi::c_ulong)
                            .wrapping_mul(64 as ::core::ffi::c_ulong)
                            .wrapping_add(
                                (if 0 != 0 {
                                    tmp.trailing_zeros() as i32 as ::core::ffi::c_ulong
                                } else {
                                    variable__ffs(tmp) as ::core::ffi::c_ulong
                                }),
                            );
                        let mut __UNIQUE_ID_y__70: ::core::ffi::c_ulong =
                            sz as ::core::ffi::c_ulong;
                        extern "C" {
                            #[link_name = "__compiletime_assert_71"]
                            fn __compiletime_assert_71_0() -> !;
                        }
                        if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_x__69 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_y__70 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) == 0
                        {
                            __compiletime_assert_71_0();
                        }
                        if __UNIQUE_ID_x__69 < __UNIQUE_ID_y__70 {
                            __UNIQUE_ID_x__69
                        } else {
                            __UNIQUE_ID_y__70
                        }
                    }) as ::core::ffi::c_ulong;
                }
            }
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_next_and_bit_72: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_next_andnot_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_ulong,
    mut start: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut mask: ::core::ffi::c_ulong = 0;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = nbits;
        let mut __start: ::core::ffi::c_ulong = start;
        if !((__start >= sz) as ::core::ffi::c_int as ::core::ffi::c_long != 0) {
            mask = !(0 as ::core::ffi::c_ulong)
                << (__start & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
            idx = __start.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong);
            tmp = *addr1.offset(idx as isize) & !*addr2.offset(idx as isize) & mask;
            loop {
                if !(tmp == 0) {
                    current_block = 2473556513754201174;
                    break;
                }
                if idx
                    .wrapping_add(1 as ::core::ffi::c_ulong)
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    >= sz
                {
                    current_block = 14523784380283086299;
                    break;
                }
                idx = idx.wrapping_add(1);
                tmp = *addr1.offset(idx as isize) & !*addr2.offset(idx as isize);
            }
            match current_block {
                14523784380283086299 => {}
                _ => {
                    sz = ({
                        let mut __UNIQUE_ID_x__73: ::core::ffi::c_ulong = (idx
                            as ::core::ffi::c_ulong)
                            .wrapping_mul(64 as ::core::ffi::c_ulong)
                            .wrapping_add(
                                (if 0 != 0 {
                                    tmp.trailing_zeros() as i32 as ::core::ffi::c_ulong
                                } else {
                                    variable__ffs(tmp) as ::core::ffi::c_ulong
                                }),
                            );
                        let mut __UNIQUE_ID_y__74: ::core::ffi::c_ulong =
                            sz as ::core::ffi::c_ulong;
                        extern "C" {
                            #[link_name = "__compiletime_assert_75"]
                            fn __compiletime_assert_75_0() -> !;
                        }
                        if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_x__73 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_y__74 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) == 0
                        {
                            __compiletime_assert_75_0();
                        }
                        if __UNIQUE_ID_x__73 < __UNIQUE_ID_y__74 {
                            __UNIQUE_ID_x__73
                        } else {
                            __UNIQUE_ID_y__74
                        }
                    }) as ::core::ffi::c_ulong;
                }
            }
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_next_andnot_bit_76: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_next_or_bit(
    mut addr1: *const ::core::ffi::c_ulong,
    mut addr2: *const ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_ulong,
    mut start: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut mask: ::core::ffi::c_ulong = 0;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = nbits;
        let mut __start: ::core::ffi::c_ulong = start;
        if !((__start >= sz) as ::core::ffi::c_int as ::core::ffi::c_long != 0) {
            mask = !(0 as ::core::ffi::c_ulong)
                << (__start & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
            idx = __start.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong);
            tmp = (*addr1.offset(idx as isize) | *addr2.offset(idx as isize)) & mask;
            loop {
                if !(tmp == 0) {
                    current_block = 2473556513754201174;
                    break;
                }
                if idx
                    .wrapping_add(1 as ::core::ffi::c_ulong)
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    >= sz
                {
                    current_block = 14523784380283086299;
                    break;
                }
                idx = idx.wrapping_add(1);
                tmp = *addr1.offset(idx as isize) | *addr2.offset(idx as isize);
            }
            match current_block {
                14523784380283086299 => {}
                _ => {
                    sz = ({
                        let mut __UNIQUE_ID_x__77: ::core::ffi::c_ulong = (idx
                            as ::core::ffi::c_ulong)
                            .wrapping_mul(64 as ::core::ffi::c_ulong)
                            .wrapping_add(
                                (if 0 != 0 {
                                    tmp.trailing_zeros() as i32 as ::core::ffi::c_ulong
                                } else {
                                    variable__ffs(tmp) as ::core::ffi::c_ulong
                                }),
                            );
                        let mut __UNIQUE_ID_y__78: ::core::ffi::c_ulong =
                            sz as ::core::ffi::c_ulong;
                        extern "C" {
                            #[link_name = "__compiletime_assert_79"]
                            fn __compiletime_assert_79_0() -> !;
                        }
                        if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_x__77 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_y__78 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) == 0
                        {
                            __compiletime_assert_79_0();
                        }
                        if __UNIQUE_ID_x__77 < __UNIQUE_ID_y__78 {
                            __UNIQUE_ID_x__77
                        } else {
                            __UNIQUE_ID_y__78
                        }
                    }) as ::core::ffi::c_ulong;
                }
            }
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_next_or_bit_80: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_next_zero_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut nbits: ::core::ffi::c_ulong,
    mut start: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    return '_c2rust_label: {
        let mut current_block: u64;
        let mut mask: ::core::ffi::c_ulong = 0;
        let mut idx: ::core::ffi::c_ulong = 0;
        let mut tmp: ::core::ffi::c_ulong = 0;
        let mut sz: ::core::ffi::c_ulong = nbits;
        let mut __start: ::core::ffi::c_ulong = start;
        if !((__start >= sz) as ::core::ffi::c_int as ::core::ffi::c_long != 0) {
            mask = !(0 as ::core::ffi::c_ulong)
                << (__start & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
            idx = __start.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong);
            tmp = !*addr.offset(idx as isize) & mask;
            loop {
                if !(tmp == 0) {
                    current_block = 2473556513754201174;
                    break;
                }
                if idx
                    .wrapping_add(1 as ::core::ffi::c_ulong)
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    >= sz
                {
                    current_block = 14523784380283086299;
                    break;
                }
                idx = idx.wrapping_add(1);
                tmp = !*addr.offset(idx as isize);
            }
            match current_block {
                14523784380283086299 => {}
                _ => {
                    sz = ({
                        let mut __UNIQUE_ID_x__81: ::core::ffi::c_ulong = (idx
                            as ::core::ffi::c_ulong)
                            .wrapping_mul(64 as ::core::ffi::c_ulong)
                            .wrapping_add(
                                (if 0 != 0 {
                                    tmp.trailing_zeros() as i32 as ::core::ffi::c_ulong
                                } else {
                                    variable__ffs(tmp) as ::core::ffi::c_ulong
                                }),
                            );
                        let mut __UNIQUE_ID_y__82: ::core::ffi::c_ulong =
                            sz as ::core::ffi::c_ulong;
                        extern "C" {
                            #[link_name = "__compiletime_assert_83"]
                            fn __compiletime_assert_83_0() -> !;
                        }
                        if (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_x__81 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) & (if (-(1 as ::core::ffi::c_int) as ::core::ffi::c_ulong)
                            < 1 as ::core::ffi::c_int as ::core::ffi::c_ulong
                        {
                            2 as ::core::ffi::c_int
                                + (0 != 0
                                    && __UNIQUE_ID_y__82 as ::core::ffi::c_longlong
                                        >= 0 as ::core::ffi::c_longlong)
                                    as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                                + 2 as ::core::ffi::c_int
                                    * ((::core::mem::size_of::<::core::ffi::c_ulong>() as usize)
                                        < 4 as usize)
                                        as ::core::ffi::c_int
                        }) == 0
                        {
                            __compiletime_assert_83_0();
                        }
                        if __UNIQUE_ID_x__81 < __UNIQUE_ID_y__82 {
                            __UNIQUE_ID_x__81
                        } else {
                            __UNIQUE_ID_y__82
                        }
                    }) as ::core::ffi::c_ulong;
                }
            }
        }
        sz
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_next_zero_bit_84: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _find_last_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    if size != 0 {
        let mut val: ::core::ffi::c_ulong = !(0 as ::core::ffi::c_ulong)
            >> (size.wrapping_neg()
                & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong);
        let mut idx: ::core::ffi::c_ulong = size
            .wrapping_sub(1 as ::core::ffi::c_ulong)
            .wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong);
        loop {
            val &= *addr.offset(idx as isize);
            if val != 0 {
                return idx
                    .wrapping_mul(BITS_PER_LONG as ::core::ffi::c_ulong)
                    .wrapping_add(__fls(val));
            }
            val = !(0 as ::core::ffi::c_ulong);
            let fresh1 = idx;
            idx = idx.wrapping_sub(1);
            if !(fresh1 != 0) {
                break;
            }
        }
    }
    return size;
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable__find_last_bit_85: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn find_next_clump8(
    mut clump: *mut ::core::ffi::c_ulong,
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
    mut offset: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    offset = find_next_bit(addr, size, offset);
    if offset == size {
        return size;
    }
    offset = (offset as ::core::ffi::c_ulong
        & !((8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as ::core::ffi::c_ulong))
        as ::core::ffi::c_ulong;
    *clump = bitmap_read(addr, offset, BITS_PER_BYTE as ::core::ffi::c_ulong);
    return offset;
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable_find_next_clump8_86: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn find_random_bit(
    mut addr: *const ::core::ffi::c_ulong,
    mut size: ::core::ffi::c_ulong,
) -> ::core::ffi::c_ulong {
    let mut w: ::core::ffi::c_int =
        bitmap_weight(addr, size as ::core::ffi::c_uint) as ::core::ffi::c_int;
    match w {
        0 => return size,
        1 => return find_first_bit(addr, size),
        _ => {
            return find_nth_bit(
                addr,
                size,
                get_random_u32_below(w as u32_0) as ::core::ffi::c_ulong,
            );
        }
    };
}
#[used]
#[link_section = ".discard.addressable"]
static mut __UNIQUE_ID_addressable_find_random_bit_87: *mut ::core::ffi::c_void =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
unsafe extern "C" fn run_static_initializers() {
    __UNIQUE_ID_addressable__find_first_bit_45 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_first_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_first_and_bit_49 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_first_and_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_first_andnot_bit_53 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_first_andnot_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_first_and_and_bit_57 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_first_and_and_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_first_zero_bit_61 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_first_zero_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_next_bit_65 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_next_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable___find_nth_bit_66 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        __find_nth_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable___find_nth_and_bit_67 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        __find_nth_and_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable___find_nth_and_andnot_bit_68 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        __find_nth_and_andnot_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_next_and_bit_72 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_next_and_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_next_andnot_bit_76 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_next_andnot_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_next_or_bit_80 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_next_or_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_next_zero_bit_84 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_next_zero_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable__find_last_bit_85 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        _find_last_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable_find_next_clump8_86 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *mut ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        find_next_clump8
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_ulong,
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
    __UNIQUE_ID_addressable_find_random_bit_87 = ::core::mem::transmute::<
        Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
        >,
        uintptr_t,
    >(Some(
        find_random_bit
            as unsafe extern "C" fn(
                *const ::core::ffi::c_ulong,
                ::core::ffi::c_ulong,
            ) -> ::core::ffi::c_ulong,
    )) as *mut ::core::ffi::c_void;
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
