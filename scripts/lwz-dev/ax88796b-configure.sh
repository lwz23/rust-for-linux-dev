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

KERNEL_SRC=/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust
KERNEL_BUILD=/home/lwz/rfl-dev/build-ax88796b-blind

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

make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 olddefconfig

echo "Configured ax88796b benchmark kernel ($mode) in $KERNEL_BUILD/.config"
