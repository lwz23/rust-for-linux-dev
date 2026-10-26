use ::core::arch::asm;
extern "C" {
    fn warn_slowpath_fmt(
        file: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        taint: ::core::ffi::c_uint,
        fmt: *const ::core::ffi::c_char,
        ...
    );
}
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
#[no_mangle]
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
    if 0 != 0 {
        asm!(
            "lock orb {1:l},({0})\n", in (reg) & mut * ((addr as * mut
            ::core::ffi::c_void).offset((nr >> 3 as ::core::ffi::c_int) as isize) as *
            mut ::core::ffi::c_char), inlateout(reg_byte) (1 as ::core::ffi::c_int) <<
            (nr & 7 as ::core::ffi::c_long) => _, options(preserves_flags, att_syntax)
        );
    } else {
        asm!(
            "lock  btsq  {1},({0})\n", in (reg) & * (addr as * mut ::core::ffi::c_long),
            inlateout(Ir) nr => _, options(preserves_flags, att_syntax)
        );
    };
}
#[inline(always)]
unsafe extern "C" fn arch___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    asm!(
        " btsq  {1},({0})\n", in (reg) & * (addr as * mut ::core::ffi::c_long),
        inlateout(Ir) nr => _, options(preserves_flags, att_syntax)
    );
}
#[inline(always)]
unsafe extern "C" fn arch_clear_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    if 0 != 0 {
        asm!(
            "lock andb {1:l},({0})\n", in (reg) & mut * ((addr as * mut
            ::core::ffi::c_void).offset((nr >> 3 as ::core::ffi::c_int) as isize) as *
            mut ::core::ffi::c_char), inlateout(reg_byte) ! ((1 as ::core::ffi::c_int) <<
            (nr & 7 as ::core::ffi::c_long)) => _, options(preserves_flags, att_syntax)
        );
    } else {
        asm!(
            "lock  btrq  {1},({0})\n", in (reg) & * (addr as * mut ::core::ffi::c_long),
            inlateout(Ir) nr => _, options(preserves_flags, att_syntax)
        );
    };
}
#[inline(always)]
unsafe extern "C" fn arch___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    asm!(
        " btrq  {1},({0})\n", in (reg) & * (addr as * mut ::core::ffi::c_long),
        inlateout(Ir) nr => _, options(preserves_flags, att_syntax)
    );
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
#[no_mangle]
pub unsafe extern "C" fn rust_helper___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) -> ::core::ffi::c_int {
    if 0 != 0 && 0 != 0 && addr as uintptr_t != NULL as uintptr_t && 0 != 0 {
        generic___set_bit(nr, addr as *mut ::core::ffi::c_ulong);
    } else {
        ___set_bit(nr, addr as *mut ::core::ffi::c_ulong);
    };
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) -> ::core::ffi::c_int {
    if 0 != 0 && 0 != 0 && addr as uintptr_t != NULL as uintptr_t && 0 != 0 {
        generic___clear_bit(nr, addr as *mut ::core::ffi::c_ulong);
    } else {
        ___clear_bit(nr, addr as *mut ::core::ffi::c_ulong);
    };
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper_set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) -> ::core::ffi::c_int {
    set_bit(nr as ::core::ffi::c_long, addr);
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn rust_helper_clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) -> ::core::ffi::c_int {
    clear_bit(nr as ::core::ffi::c_long, addr);
    panic!("Reached end of non-void function without returning");
}
