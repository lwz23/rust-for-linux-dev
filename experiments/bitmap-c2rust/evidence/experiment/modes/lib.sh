#!/bin/bash
ROOT=/home/zjp/kmiri-workspaces/rfl-uml-c2rust-bitmap-20261026
SRC=/home/zjp/kmiri-workspaces/rfl-uml-c2rust-20261021
BUILD=/home/zjp/kmiri-workspaces/rfl-uml-native-build
MIRI=/home/zjp/.rustup/toolchains/nightly-2026-07-21-x86_64-unknown-linux-gnu/bin/miri
run(){ extra="$1"; filter="$2"; log="$3"; cd "$BUILD"; timeout 700 env OBJTREE="$BUILD" "$MIRI" --test --edition=2021 \
    -Zbinary_dep_depinfo=y -Astable_features -Aunused_features \
    -Dnon_ascii_idents -Dunsafe_op_in_unsafe_fn -Wmissing_docs \
    -Wrust_2018_idioms -Wunreachable_pub -Wclippy::all \
    -Wclippy::as_ptr_cast_mut -Wclippy::as_underscore -Wclippy::cast_lossless \
    -Aclippy::collapsible_if -Aclippy::collapsible_match \
    -Wclippy::ignored_unit_patterns -Aclippy::incompatible_msrv \
    -Wclippy::mut_mut -Wclippy::needless_bitwise_bool \
    -Aclippy::needless_lifetimes -Wclippy::no_mangle_with_rust_abi \
    -Wclippy::ptr_as_ptr -Wclippy::ptr_cast_constness -Wclippy::ref_as_ptr \
    -Wclippy::undocumented_unsafe_blocks -Aclippy::uninlined_format_args \
    -Wclippy::unnecessary_safety_comment -Wclippy::unnecessary_safety_doc \
    -Aclippy::unwrap_or_default -Wrustdoc::missing_crate_level_docs \
    -Wrustdoc::unescaped_backticks \
    $extra -Zmiri-backtrace=full -Amissing-docs \
    --extern=c2rust_find_bit="$ROOT/lib/libc2rust_find_bit.rlib" \
    --extern=c2rust_bitmap_helpers="$ROOT/lib/libc2rust_bitmap_helpers.rlib" \
    -Zalways-encode-mir --sysroot=/home/zjp/.cache/miri \
    @./include/generated/rustc_cfg --cfg miri \
    --extern ffi=./rust/test/libffi.rlib \
    --extern build_error=./rust/test/libbuild_error.rlib \
    --extern macros=./rust/test/libmacros.so \
    --extern pin_init=./rust/test/libpin_init.rlib \
    --extern bindings=./rust/test/libbindings.rlib \
    --extern uapi=./rust/test/libuapi.rlib \
    --extern zerocopy=./rust/test/libzerocopy.rlib \
    --extern zerocopy_derive=./rust/test/libzerocopy_derive.so \
    -L./rust/test --crate-name kernel_miri \
    "$SRC/rust/kernel/lib.rs" -- --test-threads=1 $filter >"$log" 2>&1
  }
