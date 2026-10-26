from pathlib import Path

src = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-bitmap-20261026/wrap-out/src/bitmap_bindings.rs")
text = src.read_text()


def replace_fn(text, signature, new_fn):
    pos = text.find(signature)
    if pos < 0:
        raise SystemExit("missing " + signature)
    header = text.rfind("#[", 0, pos)
    if header < 0 or pos - header > 80:
        header = pos
    brace = text.find("{", pos)
    depth = 0
    end = None
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
    if end is None:
        raise SystemExit("unclosed " + signature)
    return text[:header] + new_fn + text[end:]


text = replace_fn(
    text,
    "fn arch_set_bit(",
    """#[inline(always)]
unsafe extern "C" fn arch_set_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___set_bit(nr as ::core::ffi::c_ulong, addr);
}
""",
)
text = replace_fn(
    text,
    "fn arch_clear_bit(",
    """#[inline(always)]
unsafe extern "C" fn arch_clear_bit(
    mut nr: ::core::ffi::c_long,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___clear_bit(nr as ::core::ffi::c_ulong, addr);
}
""",
)
text = replace_fn(
    text,
    "fn arch___set_bit(",
    """#[inline(always)]
unsafe extern "C" fn arch___set_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___set_bit(nr, addr);
}
""",
)
text = replace_fn(
    text,
    "fn arch___clear_bit(",
    """#[inline(always)]
unsafe extern "C" fn arch___clear_bit(
    mut nr: ::core::ffi::c_ulong,
    mut addr: *mut ::core::ffi::c_ulong,
) {
    generic___clear_bit(nr, addr);
}
""",
)
text = replace_fn(
    text,
    "fn bitmap_copy_and_extend(",
    """#[inline]
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
""",
)
for line in (
    "use ::c2rust_asm_casts;\n",
    "use ::libc;\n",
    "use ::core::arch::asm;\n",
    "use ::c2rust_asm_casts::AsmCastTrait;\n",
):
    text = text.replace(line, "")
text = text.replace(
    "#[no_mangle]\npub static mut current_stack_pointer:",
    "pub static mut current_stack_pointer:",
    1,
)
if "asm!" in text or "c2rust_asm" in text or "libc::" in text:
    raise SystemExit("leftover asm/libc")
src.write_text(text)
print("adapted", len(text))
