from pathlib import Path

path = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-20261021/rust/kernel/bitmap.rs")
backup = Path("/home/zjp/kmiri-workspaces/rfl-uml-c2rust-bitmap-20261026/bitmap.rs.before-helpers")
text = path.read_text()
if not backup.exists():
    backup.write_text(text)

if "extern crate c2rust_bitmap_helpers;" not in text:
    needle = "extern crate c2rust_find_bit;\n"
    if needle not in text:
        raise SystemExit("missing find_bit extern")
    text = text.replace(
        needle,
        needle + "#[cfg(miri)]\nextern crate c2rust_bitmap_helpers;\n",
        1,
    )


def match_brace(text, open_at):
    depth = 0
    for i in range(open_at, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i + 1
    raise SystemExit("unbalanced")


def function_span(text, name):
    key = "pub fn " + name + "("
    pos = text.find(key)
    if pos < 0:
        raise SystemExit("missing fn " + name)
    brace = text.find("{", pos)
    end = match_brace(text, brace)
    return pos, end


def unwrap_cfg(fn_text, name):
    marker = "#[cfg(miri)]"
    pos = fn_text.find(marker)
    if pos < 0:
        raise SystemExit(name + " has no cfg(miri)")
    brace = fn_text.find("{", pos)
    end = match_brace(fn_text, brace)
    fn_text = fn_text[:pos] + fn_text[end:]
    marker = "#[cfg(not(miri))]"
    pos = fn_text.find(marker)
    if pos < 0:
        raise SystemExit(name + " has no cfg(not(miri))")
    brace = fn_text.find("{", pos)
    end = match_brace(fn_text, brace)
    inner = fn_text[brace + 1 : end - 1]
    fn_text = fn_text[:pos] + inner + fn_text[end:]
    if "#[cfg(miri)]" in fn_text or "bindings::" not in fn_text:
        raise SystemExit(name + " rewrite failed")
    return fn_text


for name in ("set_bit", "clear_bit", "copy_and_extend", "last_bit", "next_zero_bit"):
    start, end = function_span(text, name)
    text = text[:start] + unwrap_cfg(text[start:end], name) + text[end:]

if "extern crate c2rust_bitmap_helpers;" not in text:
    raise SystemExit("extern missing")
path.write_text(text)
print("patched", path)
