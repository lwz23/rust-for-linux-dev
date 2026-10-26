#!/usr/bin/env bash
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

