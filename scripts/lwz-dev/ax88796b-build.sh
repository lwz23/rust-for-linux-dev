#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "Usage: $0 <c|rust> [make-target]" >&2
    exit 1
fi

mode="$1"
target="${2:-}"

source /home/lwz/rfl-dev/env.sh

DEFAULT_KERNEL_SRC=/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust
DEFAULT_KERNEL_BUILD=/home/lwz/rfl-dev/build-ax88796b-blind
KERNEL_SRC="${AX88796B_KERNEL_SRC:-$DEFAULT_KERNEL_SRC}"
KERNEL_BUILD="${AX88796B_KERNEL_BUILD:-$DEFAULT_KERNEL_BUILD}"
CONFIGURE="$KERNEL_SRC/scripts/lwz-dev/ax88796b-configure.sh"

"$CONFIGURE" "$mode"

make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 rustavailable

if [[ -n "$target" ]]; then
    make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 -j"$(nproc)" "$target"
else
    make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 -j"$(nproc)" bzImage modules
fi

echo "Built ax88796b benchmark kernel ($mode) in $KERNEL_BUILD"
