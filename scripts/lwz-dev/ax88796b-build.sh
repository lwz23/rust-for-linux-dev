#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "Usage: $0 <c|rust> [make-target]" >&2
    exit 1
fi

mode="$1"
target="${2:-}"

source /home/lwz/rfl-dev/env.sh

KERNEL_SRC=/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust
KERNEL_BUILD=/home/lwz/rfl-dev/build-ax88796b-blind
CONFIGURE="$KERNEL_SRC/scripts/lwz-dev/ax88796b-configure.sh"

"$CONFIGURE" "$mode"

make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 rustavailable

if [[ -n "$target" ]]; then
    make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 -j"$(nproc)" "$target"
else
    make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 -j"$(nproc)" bzImage modules
fi

echo "Built ax88796b benchmark kernel ($mode) in $KERNEL_BUILD"
