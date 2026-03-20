#!/usr/bin/env bash
set -euo pipefail

source /home/lwz/rfl-dev/env.sh

KERNEL_SRC=/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust
BENCH_DIR="$KERNEL_SRC/Documentation/rust/lwz-dev/c2rust-benchmarks/ax88796b"

git -C "$KERNEL_SRC" diff --check

AX88796B_DEBUG=1 "$KERNEL_SRC/scripts/lwz-dev/ax88796b-build.sh" c
AX88796B_DEBUG=1 "$KERNEL_SRC/scripts/lwz-dev/ax88796b-build.sh" rust

"$KERNEL_SRC/scripts/lwz-dev/ax88796b-kunit.sh" c
"$KERNEL_SRC/scripts/lwz-dev/ax88796b-kunit.sh" rust

python3 "$KERNEL_SRC/scripts/lwz-dev/ax88796b-runtime-diff.py" \
    compare \
    "$BENCH_DIR/runtime-c.yaml" \
    "$BENCH_DIR/runtime-rust.yaml" \
    "$BENCH_DIR/runtime_diff.yaml"

echo "ax88796b engineering-grade checks passed"
