#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <c|rust>" >&2
    exit 1
fi

mode="$1"
case "$mode" in
    c|rust)
        ;;
    *)
        echo "Unsupported mode: $mode" >&2
        exit 1
        ;;
esac

source /home/lwz/rfl-dev/env.sh

DEFAULT_KERNEL_SRC=/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust
DEFAULT_KERNEL_BUILD=/home/lwz/rfl-dev/build-ax88796b-blind
KERNEL_SRC="${AX88796B_KERNEL_SRC:-$DEFAULT_KERNEL_SRC}"
KERNEL_BUILD="${AX88796B_KERNEL_BUILD:-$DEFAULT_KERNEL_BUILD}"

mkdir -p "$KERNEL_BUILD"

make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 x86_64_defconfig

"$KERNEL_SRC/scripts/config" --file "$KERNEL_BUILD/.config" \
    -e RUST \
    -e MODULES \
    -e MODULE_UNLOAD \
    -e PHYLIB \
    -e AX88796B_PHY \
    -d DEBUG_INFO_BTF

if [[ "$mode" == "rust" ]]; then
    "$KERNEL_SRC/scripts/config" --file "$KERNEL_BUILD/.config" \
        -e RUST_PHYLIB_ABSTRACTIONS \
        -e AX88796B_RUST_PHY
else
    "$KERNEL_SRC/scripts/config" --file "$KERNEL_BUILD/.config" \
        -d AX88796B_RUST_PHY
fi

if [[ "${AX88796B_DEBUG:-0}" == "1" ]]; then
    "$KERNEL_SRC/scripts/config" --file "$KERNEL_BUILD/.config" \
        -e DEBUG_KERNEL \
        -e PROVE_LOCKING \
        -e DEBUG_MUTEXES \
        -e REFCOUNT_FULL \
        -e UBSAN \
        -e KASAN \
        -e KASAN_INLINE \
        -d RANDOMIZE_BASE
fi

if [[ -n "${AX88796B_CONFIG_FRAGMENTS:-}" ]]; then
    IFS=':' read -r -a fragments <<< "${AX88796B_CONFIG_FRAGMENTS}"
    for fragment in "${fragments[@]}"; do
        if [[ -n "$fragment" ]]; then
            cat "$fragment" >> "$KERNEL_BUILD/.config"
        fi
    done
fi

make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 olddefconfig

echo "Configured ax88796b benchmark kernel ($mode) in $KERNEL_BUILD/.config"
