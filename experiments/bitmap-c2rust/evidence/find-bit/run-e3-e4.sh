#!/bin/bash
# E3 rename control and E4 injections. Does not touch /home/zjp/KMiri/linux.
set -u
ROOT=/home/zjp/kmiri-workspaces/rfl-uml-c2rust-e2
SRC=/home/zjp/kmiri-workspaces/rfl-uml-c2rust-20261021
BUILD=/home/zjp/kmiri-workspaces/rfl-uml-native-build
C2RUST=/home/zjp/kmiri-workspaces/c2rust/target/release/c2rust
RUSTC=/home/zjp/.rustup/toolchains/nightly-2026-07-21-x86_64-unknown-linux-gnu/bin/rustc
MIRI=/home/zjp/.rustup/toolchains/nightly-2026-07-21-x86_64-unknown-linux-gnu/bin/miri
RUSTFMT=/home/zjp/.rustup/toolchains/nightly-2026-07-21-x86_64-unknown-linux-gnu/bin/rustfmt
STATUS=$ROOT/e3-e4.status
GOOD_RS=$ROOT/patched/src/find_bit.rs
GOOD_RLIB=$ROOT/patched/libc2rust_find_bit.rlib
BITMAP=$SRC/rust/kernel/bitmap.rs
CF=$SRC/lib/find_bit.c
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTUP_TOOLCHAIN=nightly-2026-07-21

status() { echo "$(date +%H:%M:%S) $*" | tee -a "$STATUS"; }

run_miri() {
  local rlib="$1" logfile="$2"
  shift 2
  status "miri start $* -> $(basename "$logfile")"
  cd "$BUILD"
  set +e
  timeout 700 env OBJTREE="$BUILD" "$MIRI" --test --edition=2021 \
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
    -Zmiri-tree-borrows -Zmiri-backtrace=full -Amissing-docs \
    --extern=c2rust_find_bit="$rlib" \
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
    "$SRC/rust/kernel/lib.rs" -- "$@" >"$logfile" 2>&1
  local rc=$?
  set -e
  status "miri rc=$rc $(basename "$logfile")"
  echo "$rc"
}

build_rlib() {
  local src="$1" outdir="$2"
  mkdir -p "$outdir"
  status "rustc $(basename "$src") -> $outdir"
  "$RUSTC" --edition=2021 -Z always-encode-mir --crate-type rlib \
    --crate-name c2rust_find_bit -A warnings --out-dir "$outdir" "$src"
}

post_one() {
  local target="$1"
  python3 - "$target" << 'PY'
import sys
from pathlib import Path
p = Path(sys.argv[1])
text = p.read_text()
ffs_new = '#[inline(always)]\nunsafe extern "C" fn variable__ffs(mut word: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong {\n    let mut num: ::core::ffi::c_ulong = 0;\n    if word & 0xffffffff == 0 { num += 32; word >>= 32; }\n    if word & 0xffff == 0 { num += 16; word >>= 16; }\n    if word & 0xff == 0 { num += 8; word >>= 8; }\n    if word & 0xf == 0 { num += 4; word >>= 4; }\n    if word & 0x3 == 0 { num += 2; word >>= 2; }\n    if word & 0x1 == 0 { num += 1; }\n    return num;\n}\n'
fls_new = '#[inline(always)]\nunsafe extern "C" fn __fls(mut word: ::core::ffi::c_ulong) -> ::core::ffi::c_ulong {\n    let mut num: i32 = 63;\n    if word & (!0u64 << 32) == 0 { num -= 32; word <<= 32; }\n    if word & (!0u64 << (64 - 16)) == 0 { num -= 16; word <<= 16; }\n    if word & (!0u64 << (64 - 8)) == 0 { num -= 8; word <<= 8; }\n    if word & (!0u64 << (64 - 4)) == 0 { num -= 4; word <<= 4; }\n    if word & (!0u64 << (64 - 2)) == 0 { num -= 2; word <<= 2; }\n    if word & (!0u64 << (64 - 1)) == 0 { num -= 1; }\n    return num as ::core::ffi::c_ulong;\n}\n'

def replace_fn(text, name, new):
    key = "fn " + name
    pos = text.find(key)
    if pos < 0:
        raise SystemExit("missing " + name)
    header = text.rfind("#[inline", 0, pos)
    brace = text.find("{", pos)
    depth = 0
    i = brace
    end = None
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
        i += 1
    if end is None:
        raise SystemExit("unclosed " + name)
    return text[:header] + new + text[end+1:]

text = replace_fn(text, "variable__ffs", ffs_new)
text = replace_fn(text, "__fls", fls_new)
text = text.replace("use ::core::arch::asm;\n", "")
parts = text.split("\n#[no_mangle]\n")
fixed = 0
out = [parts[0]]
for part in parts[1:]:
    if "return '_c2rust_label" in part and "let mut sz:" in part and "\n        sz\n    };" not in part:
        marker = "\n    };\n"
        pos = part.rfind(marker)
        if pos < 0:
            raise SystemExit("no label close")
        part = part[:pos] + "\n        sz" + part[pos:]
        fixed += 1
    out.append(part)
text = "\n#[no_mangle]\n".join(out)
if "asm!(" in text:
    raise SystemExit("asm remains: " + str(text.count("asm!(")))
p.write_text(text)
print("fixed", fixed, "bytes", len(text))
PY
}

: > "$STATUS"
status "start"
sha256sum "$MIRI" "$C2RUST" "$GOOD_RS" "$GOOD_RLIB" "$BITMAP" "$CF" | tee "$ROOT/e3-e4-before.sha256"
echo "ORIGINAL $(git -C /home/zjp/KMiri/linux status --porcelain | wc -l)" | tee -a "$STATUS"
echo "ISOLATED $(git -C "$SRC" status --porcelain)" | tee -a "$STATUS"
cp -a "$BITMAP" "$ROOT/bitmap.rs.before-e4"
cp -a "$GOOD_RS" "$ROOT/find_bit.rs.before-e4"
cp -a "$GOOD_RLIB" "$ROOT/libc2rust_find_bit.rlib.before-e4"

status "E3 rename"
mkdir -p "$ROOT/renamed"
python3 - << 'PY'
from pathlib import Path
src = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-e2/find_bit.rs.before-e4")
dst = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-e2/renamed/find_bit.rs")
text = src.read_text()
n = text.count("_find_next_bit")
text2 = text.replace("_find_next_bit", "_find_next_bit_absent")
if "_find_next_bit(" in text2.replace("_find_next_bit_absent", ""):
    raise SystemExit("rename leftover call")
dst.write_text(text2)
print("renamed occurrences", n)
PY
build_rlib "$ROOT/renamed/find_bit.rs" "$ROOT/renamed"
nm -g "$ROOT/renamed/"*.o 2>/dev/null | grep -E "find_next_bit" | tee "$ROOT/e3-rename.nm" || true
if grep -q " _find_next_bit$" "$ROOT/e3-rename.nm"; then
  status "E3 FAIL symbol still exported"
else
  status "E3 renamed symbol absent from nm"
fi
E3RC=$(run_miri "$ROOT/renamed/libc2rust_find_bit.rlib" "$ROOT/e3-rename-tb.log" bitmap_borrow)
status "E3 rename miri_rc=$E3RC"
grep -n -E "error:|can't call foreign|undefined symbol|_find_next_bit|test result:" "$ROOT/e3-rename-tb.log" | tail -30 | tee -a "$STATUS"

status "E3 restore six"
E3R=$(run_miri "$GOOD_RLIB" "$ROOT/e3-restore-tb.log" bitmap_borrow bitmap_copy bitmap_set_clear_find bitmap_vec_new owned_bitmap_out_of_bounds)
status "E3 restore miri_rc=$E3R"
grep -n -E "test bitmap::|test result:" "$ROOT/e3-restore-tb.log" | tee -a "$STATUS"

status "E4 C inject"
python3 - << 'PY'
from pathlib import Path
p = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-20261021/lib/find_bit.c")
text = p.read_text()
old = """unsigned long _find_next_bit(const unsigned long *addr, unsigned long nbits, unsigned long start)
{
	return FIND_NEXT_BIT(addr[idx], /* nop */, nbits, start);
}
"""
new = """unsigned long _find_next_bit(const unsigned long *addr, unsigned long nbits, unsigned long start)
{
	/* KMIRI_E4_C_OOB: one word past the bitmap sized by nbits. */
	unsigned long kmiri_oob = *(volatile const unsigned long *)(addr + (nbits / BITS_PER_LONG));
	(void)kmiri_oob;
	return FIND_NEXT_BIT(addr[idx], /* nop */, nbits, start);
}
"""
if text.count(old) != 1:
    raise SystemExit("c pattern count " + str(text.count(old)))
p.write_text(text.replace(old, new, 1))
print("c patched")
PY
git -C "$SRC" diff -- lib/find_bit.c > "$ROOT/e4-c-oob.diff"
rm -rf "$ROOT/oob-raw"
mkdir -p "$ROOT/oob-raw"
set +e
"$C2RUST" transpile --emit-modules --emit-no-std --fail-on-error --overwrite-existing --disable-refactoring \
  --output-dir "$ROOT/oob-raw" "$ROOT/find_bit_compile_commands.json" \
  > "$ROOT/e4-c-transpile.log" 2>&1
TRC=$?
set -e
status "E4 transpile rc=$TRC"
if [ "$TRC" -ne 0 ]; then
  tail -40 "$ROOT/e4-c-transpile.log" | tee -a "$STATUS"
  status "E4 transpile failed, restoring C"
  git -C "$SRC" checkout -- lib/find_bit.c
  status "stopped after transpile failure"
  exit 2
fi
OOB_RS=$(find "$ROOT/oob-raw" -name find_bit.rs | head -1)
status "oob rs $OOB_RS"
post_one "$OOB_RS" | tee -a "$STATUS"
if ! grep -n "kmiri_oob\|read_volatile" "$OOB_RS" | head -20 | tee "$ROOT/e4-c-read-hits.txt"; then
  status "E4 injected read missing from rust"
fi
build_rlib "$OOB_RS" "$ROOT/oob"
E4C=$(run_miri "$ROOT/oob/libc2rust_find_bit.rlib" "$ROOT/e4-c-oob-tb.log" bitmap_borrow)
status "E4 C miri_rc=$E4C"
grep -n -E "error:|test result:|memory access|Undefined Behavior" "$ROOT/e4-c-oob-tb.log" | head -40 | tee -a "$STATUS"

status "restore C and regenerate"
git -C "$SRC" checkout -- lib/find_bit.c
if git -C "$SRC" diff -- lib/find_bit.c | grep -q .; then
  status "C NOT clean"
else
  status "C clean"
fi
rm -rf "$ROOT/regen-raw"
mkdir -p "$ROOT/regen-raw"
set +e
"$C2RUST" transpile --emit-modules --emit-no-std --fail-on-error --overwrite-existing --disable-refactoring \
  --output-dir "$ROOT/regen-raw" "$ROOT/find_bit_compile_commands.json" \
  > "$ROOT/e4-regen-transpile.log" 2>&1
RRC=$?
set -e
status "regen transpile rc=$RRC"
REG_RS=$(find "$ROOT/regen-raw" -name find_bit.rs | head -1)
post_one "$REG_RS" | tee -a "$STATUS"
cp -a "$GOOD_RS" "$ROOT/regen-a.rs"
cp -a "$REG_RS" "$ROOT/regen-b.rs"
set +e
"$RUSTFMT" --edition 2021 "$ROOT/regen-a.rs" "$ROOT/regen-b.rs" > "$ROOT/e4-regen-rustfmt.log" 2>&1
RFC=$?
set -e
status "rustfmt rc=$RFC"
if cmp -s "$ROOT/regen-a.rs" "$ROOT/regen-b.rs"; then
  status "regen matches postprocessed find_bit.rs after rustfmt"
else
  diff -u "$ROOT/regen-a.rs" "$ROOT/regen-b.rs" | head -80 > "$ROOT/e4-regen.diff" || true
  status "regen DIFF bytes $(wc -c < "$ROOT/e4-regen.diff")"
fi

status "E4 rust length"
python3 - << 'PY'
from pathlib import Path
p = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-20261021/rust/kernel/bitmap.rs")
text = p.read_text()
old = "bindings::_find_next_bit(self.as_ptr(), self.len(), start)"
new = "bindings::_find_next_bit(self.as_ptr(), self.len().saturating_add(64), start)"
if text.count(old) != 1:
    raise SystemExit("rust call count " + str(text.count(old)))
p.write_text(text.replace(old, new, 1))
print("rust length patched")
PY
diff -u "$ROOT/bitmap.rs.before-e4" "$BITMAP" > "$ROOT/e4-rust-len.diff" || true
E4R=$(run_miri "$GOOD_RLIB" "$ROOT/e4-rust-len-tb.log" bitmap_borrow)
status "E4 rust miri_rc=$E4R"
grep -n -E "error:|test result:|memory access|Undefined Behavior" "$ROOT/e4-rust-len-tb.log" | head -40 | tee -a "$STATUS"
cp -a "$ROOT/bitmap.rs.before-e4" "$BITMAP"
if cmp -s "$ROOT/bitmap.rs.before-e4" "$BITMAP"; then
  status "bitmap.rs restored"
else
  status "bitmap.rs RESTORE FAILED"
fi

status "final six"
FIN=$(run_miri "$GOOD_RLIB" "$ROOT/e4-restore-tb.log" bitmap_borrow bitmap_copy bitmap_set_clear_find bitmap_vec_new owned_bitmap_out_of_bounds)
status "final miri_rc=$FIN"
grep -n -E "test bitmap::|test result:" "$ROOT/e4-restore-tb.log" | tee -a "$STATUS"
echo "ORIGINAL_END $(git -C /home/zjp/KMiri/linux status --porcelain | wc -l)" | tee -a "$STATUS"
echo "ISOLATED_END $(git -C "$SRC" status --porcelain)" | tee -a "$STATUS"
sha256sum "$GOOD_RS" "$GOOD_RLIB" "$BITMAP" "$CF" | tee "$ROOT/e3-e4-after.sha256"
status "done"
