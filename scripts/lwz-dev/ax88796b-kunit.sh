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
BUILD_ROOT=/home/lwz/rfl-dev/build-ax88796b-blind
BUILD_DIR="$BUILD_ROOT/kunit-$mode"
BENCH_DIR="$KERNEL_SRC/Documentation/rust/lwz-dev/c2rust-benchmarks/ax88796b"
LOG_DIR="$BENCH_DIR/logs"
RAW_LOG="$LOG_DIR/kunit-$mode.log"
NORMALIZED="$BENCH_DIR/runtime-$mode.yaml"

mkdir -p "$BUILD_DIR" "$LOG_DIR"

if [[ "$mode" == "rust" ]]; then
    MODE_FRAGMENT="$BENCH_DIR/rust.kunitconfig"
else
    MODE_FRAGMENT="$BENCH_DIR/c.kunitconfig"
fi

AX88796B_KERNEL_BUILD="$BUILD_DIR" \
AX88796B_CONFIG_FRAGMENTS="$BENCH_DIR/base.kunitconfig:$MODE_FRAGMENT" \
"$KERNEL_SRC/scripts/lwz-dev/ax88796b-build.sh" "$mode" bzImage

python3 "$KERNEL_SRC/tools/testing/kunit/kunit.py" exec \
    --arch=x86_64 \
    --build_dir="$BUILD_DIR" \
    --raw_output=all \
    --timeout=180 \
    ax88796b_bench | tee "$RAW_LOG"

python3 "$KERNEL_SRC/scripts/lwz-dev/ax88796b-runtime-diff.py" \
    normalize "$mode" "$RAW_LOG" "$NORMALIZED"

if grep -Eq '(^|[[:space:]])not ok[[:space:]]' "$RAW_LOG"; then
    echo "ax88796b-kunit.sh: KUnit reported failures in $RAW_LOG" >&2
    exit 1
fi

echo "Captured ax88796b KUnit runtime ($mode) in $NORMALIZED"
