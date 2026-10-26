extern "C" {
    fn warn_slowpath_fmt(
        file: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        taint: ::core::ffi::c_uint,
        fmt: *const ::core::ffi::c_char,
        ...
    );
    fn __fortify_panic(reason: u8_0, avail: size_t, size: size_t) -> !;
    fn __read_overflow2();
    fn __read_overflow2_field(avail: size_t, wanted: size_t);
    fn __write_overflow();
    fn __write_overflow_field(avail: size_t, wanted: size_t);
}
pub type __u8 = ::core::ffi::c_uchar;
pub type u8_0 = __u8;
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const true_0: C2RustUnnamed = 1;
pub const false_0: C2RustUnnamed = 0;
pub type __kernel_ulong_t = ::core::ffi::c_ulong;
pub type __kernel_size_t = __kernel_ulong_t;
pub type bool_0 = bool;
pub type uintptr_t = usize;
pub type size_t = __kernel_size_t;
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct C2RustUnnamed_0(pub C2RustUnnamed_0_Inner);
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0_Inner {
    pub x: ::core::ffi::c_long,
}
#[allow(dead_code, non_upper_case_globals)]
const C2RustUnnamed_0_PADDING: usize = ::core::mem::size_of::<C2RustUnnamed_0>()
    - ::core::mem::size_of::<C2RustUnnamed_0_Inner>();
pub type fortify_func = ::core::ffi::c_uint;
pub const FORTIFY_FUNC_UNKNOWN: fortify_func = 15;
pub const FORTIFY_FUNC_strcpy: fortify_func = 14;
pub const FORTIFY_FUNC_kmemdup: fortify_func = 13;
pub const FORTIFY_FUNC_memchr_inv: fortify_func = 12;
pub const FORTIFY_FUNC_memchr: fortify_func = 11;
pub const FORTIFY_FUNC_memcmp: fortify_func = 10;
pub const FORTIFY_FUNC_memscan: fortify_func = 9;
pub const FORTIFY_FUNC_memmove: fortify_func = 8;
pub const FORTIFY_FUNC_memcpy: fortify_func = 7;
pub const FORTIFY_FUNC_memset: fortify_func = 6;
pub const FORTIFY_FUNC_strncat: fortify_func = 5;
pub const FORTIFY_FUNC_strcat: fortify_func = 4;
pub const FORTIFY_FUNC_strlcat: fortify_func = 3;
pub const FORTIFY_FUNC_strscpy: fortify_func = 2;
pub const FORTIFY_FUNC_strlen: fortify_func = 1;
pub const FORTIFY_FUNC_strnlen: fortify_func = 0;
pub const BITS_PER_LONG: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
#[inline]
unsafe extern "C" fn kasan_check_write(
    mut p: *const ::core::ffi::c_void,
    mut size: ::core::ffi::c_uint,
) -> bool_0 {
    return true_0 as ::core::ffi::c_int != 0;
}
pub const KCSAN_ACCESS_WRITE: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 0 as ::core::ffi::c_int;
pub const KCSAN_ACCESS_ATOMIC: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 2 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn kcsan_check_access(
    mut ptr: *const ::core::ffi::c_void,
    mut size: size_t,
    mut type_0: ::core::ffi::c_int,
) {}
pub const SIZE_MAX: size_t = !(0 as ::core::ffi::c_int as size_t);
pub static mut current_stack_pointer: ::core::ffi::c_ulong = 0;
#[inline(always)]
unsafe extern "C" fn generic___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    let mut mask: ::core::ffi::c_ulong = (1 as ::core::ffi::c_ulong)
        << nr.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_ulong);
    let mut p: *mut ::core::ffi::c_ulong = (addr as *mut ::core::ffi::c_ulong)
        .offset(nr.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong) as isize);
    *p |= mask;
}
#[inline(always)]
unsafe extern "C" fn generic___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    let mut mask: ::core::ffi::c_ulong = (1 as ::core::ffi::c_ulong)
        << nr.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_ulong);
    let mut p: *mut ::core::ffi::c_ulong = (addr as *mut ::core::ffi::c_ulong)
        .offset(nr.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong) as isize);
    *p &= !mask;
}
#[inline(always)]
unsafe extern "C" fn arch_set_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___set_bit(nr as ::core::ffi::c_ulong, addr);
}

#[inline(always)]
unsafe extern "C" fn arch___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___set_bit(nr, addr);
}

#[inline(always)]
unsafe extern "C" fn arch_clear_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___clear_bit(nr as ::core::ffi::c_ulong, addr);
}

#[inline(always)]
unsafe extern "C" fn arch___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___clear_bit(nr, addr);
}

#[inline(always)]
unsafe extern "C" fn instrument_write(
    mut v: *const ::core::ffi::c_void,
    mut size: size_t,
) {
    kasan_check_write(v, size as ::core::ffi::c_uint);
    kcsan_check_access(
        (v as *mut ::core::ffi::c_void as ::core::ffi::c_ulong)
            .wrapping_add(0 as ::core::ffi::c_ulong) as *mut ::core::ffi::c_void
            as *const ::core::ffi::c_void,
        size,
        KCSAN_ACCESS_WRITE,
    );
}
#[inline(always)]
unsafe extern "C" fn instrument_atomic_check_alignment(
    mut v: *const ::core::ffi::c_void,
    mut size: size_t,
) {}
#[inline(always)]
unsafe extern "C" fn instrument_atomic_write(
    mut v: *const ::core::ffi::c_void,
    mut size: size_t,
) {
    kasan_check_write(v, size as ::core::ffi::c_uint);
    kcsan_check_access(
        (v as *mut ::core::ffi::c_void as ::core::ffi::c_ulong)
            .wrapping_add(0 as ::core::ffi::c_ulong) as *mut ::core::ffi::c_void
            as *const ::core::ffi::c_void,
        size,
        KCSAN_ACCESS_ATOMIC | KCSAN_ACCESS_WRITE,
    );
    instrument_atomic_check_alignment(v, size);
}
#[inline(always)]
unsafe extern "C" fn set_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    instrument_atomic_write(
        addr.offset((nr / BITS_PER_LONG as ::core::ffi::c_long) as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_long>() as size_t,
    );
    arch_set_bit(nr, addr);
}
#[inline(always)]
unsafe extern "C" fn clear_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    instrument_atomic_write(
        addr.offset((nr / BITS_PER_LONG as ::core::ffi::c_long) as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_long>() as size_t,
    );
    arch_clear_bit(nr, addr);
}
#[inline(always)]
unsafe extern "C" fn ___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    instrument_write(
        addr.offset(nr.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong) as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_long>() as size_t,
    );
    arch___set_bit(nr, addr);
}
#[inline(always)]
unsafe extern "C" fn ___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    instrument_write(
        addr.offset(nr.wrapping_div(BITS_PER_LONG as ::core::ffi::c_ulong) as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_long>() as size_t,
    );
    arch___clear_bit(nr, addr);
}
#[inline(always)]
unsafe extern "C" fn fortify_memset_chk(
    mut size: __kernel_size_t,
    p_size: size_t,
    p_size_field: size_t,
) -> bool_0 {
    if 0 != 0 {
        if 0 != 0 && p_size_field < p_size && (0 != 0 && p_size < size) {
            __write_overflow();
        }
        if 0 != 0 && p_size_field < size {
            __write_overflow_field(p_size_field, size as size_t);
        }
    }
    if p_size != SIZE_MAX && p_size < size {
        __fortify_panic(
            ((FORTIFY_FUNC_memset as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
                | 1 as ::core::ffi::c_int) as u8_0,
            p_size,
            size as size_t,
        );
    }
    return false_0 as ::core::ffi::c_int != 0;
}
#[inline(always)]
unsafe extern "C" fn fortify_memcpy_chk(
    mut size: __kernel_size_t,
    p_size: size_t,
    q_size: size_t,
    p_size_field: size_t,
    q_size_field: size_t,
    func: u8_0,
) -> bool_0 {
    if 0 != 0 {
        if 0 != 0 && p_size_field < p_size && (0 != 0 && p_size < size) {
            __write_overflow();
        }
        if 0 != 0 && q_size_field < q_size && (0 != 0 && q_size < size) {
            __read_overflow2();
        }
        if 0 != 0 && p_size_field < size {
            __write_overflow_field(p_size_field, size as size_t);
        }
        if (0 as ::core::ffi::c_int != 0 || 0 != 0 && p_size_field < size)
            && (0 != 0 && q_size_field < size)
        {
            __read_overflow2_field(q_size_field, size as size_t);
        }
    }
    if p_size != SIZE_MAX && p_size < size {
        __fortify_panic(
            ((func as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
                | 1 as ::core::ffi::c_int) as u8_0,
            p_size,
            size as size_t,
        );
    } else if q_size != SIZE_MAX && q_size < size {
        __fortify_panic(
            ((func as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
                | 0 as ::core::ffi::c_int) as u8_0,
            q_size,
            size as size_t,
        );
    }
    if p_size_field != SIZE_MAX && p_size != p_size_field && p_size_field < size {
        return true_0 as ::core::ffi::c_int != 0;
    }
    return false_0 as ::core::ffi::c_int != 0;
}
#[inline]
unsafe extern "C" fn bitmap_copy_and_extend(
    mut to: *mut ::core::ffi::c_ulong,
    mut from: *const ::core::ffi::c_ulong,
    mut count: ::core::ffi::c_uint,
    mut size: ::core::ffi::c_uint,
) {
    let copy: ::core::ffi::c_uint = (count as usize)
        .wrapping_add(
            (::core::mem::size_of::<::core::ffi::c_long>() as usize)
                .wrapping_mul(8 as usize),
        )
        .wrapping_sub(1 as usize)
        .wrapping_div(
            (::core::mem::size_of::<::core::ffi::c_long>() as usize)
                .wrapping_mul(8 as usize),
        ) as ::core::ffi::c_uint;
    let nbytes = (copy as usize) * ::core::mem::size_of::<::core::ffi::c_long>();
    ::core::ptr::copy_nonoverlapping(from as *const u8, to as *mut u8, nbytes);
    if count.wrapping_rem(BITS_PER_LONG as ::core::ffi::c_uint) != 0 {
        *to.offset(copy.wrapping_sub(1 as ::core::ffi::c_uint) as isize)
            &= !(0 as ::core::ffi::c_ulong)
                >> (count.wrapping_neg()
                    & (BITS_PER_LONG - 1 as ::core::ffi::c_int) as ::core::ffi::c_uint);
    }
    let total = ((size
        .wrapping_add(
            (64 as ::core::ffi::c_int as ::core::ffi::c_uint)
                .wrapping_sub(1 as ::core::ffi::c_uint),
        )
        & !(64 as ::core::ffi::c_int as ::core::ffi::c_uint)
            .wrapping_sub(1 as ::core::ffi::c_uint))
        .wrapping_div(8 as ::core::ffi::c_uint)) as usize;
    let rest = total.wrapping_sub(nbytes);
    ::core::ptr::write_bytes(to.offset(copy as isize) as *mut u8, 0, rest);
}

#[no_mangle]
pub unsafe extern "C" fn rust_helper___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    if 0 != 0 && 0 != 0 && addr as uintptr_t != NULL as uintptr_t && 0 != 0 {
        generic___set_bit(nr, addr as *mut ::core::ffi::c_ulong);
    } else {
        ___set_bit(nr, addr as *mut ::core::ffi::c_ulong);
    };
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    if 0 != 0 && 0 != 0 && addr as uintptr_t != NULL as uintptr_t && 0 != 0 {
        generic___clear_bit(nr, addr as *mut ::core::ffi::c_ulong);
    } else {
        ___clear_bit(nr, addr as *mut ::core::ffi::c_ulong);
    };
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper_set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    set_bit(nr as ::core::ffi::c_long, addr);
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper_clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    clear_bit(nr as ::core::ffi::c_long, addr);
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper_bitmap_copy_and_extend(
    mut to: *mut ::core::ffi::c_ulong,
    mut from: *const ::core::ffi::c_ulong,
    mut count: ::core::ffi::c_uint,
    mut size: ::core::ffi::c_uint,
) {
    bitmap_copy_and_extend(to, from, count, size);
}
