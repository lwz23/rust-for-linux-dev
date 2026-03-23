#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: configure-debug-kernel.sh <memory-debug|concurrency-debug|leak-debug> <c|rust> [build-dir]

Applies the fixed rnull validation profile to a single reusable build
directory.
EOF
}

PROFILE="${1:-}"
IMPLEMENTATION="${2:-}"
BUILD_DIR="${3:-}"
BASE_CONFIG="${BASE_CONFIG:-$RFL_DEV_ROOT/build/.config}"

if [[ -z "$PROFILE" || -z "$IMPLEMENTATION" ]]; then
    usage >&2
    exit 1
fi

if [[ -z "$BUILD_DIR" ]]; then
    BUILD_DIR="$(rnull_default_build_dir)"
fi

mkdir -p "$BUILD_DIR"

if [[ -f "$BASE_CONFIG" ]]; then
    cp "$BASE_CONFIG" "$BUILD_DIR/.config"
elif [[ -f "$BUILD_DIR/.config" ]]; then
    :
else
    make -C "$KERNEL_SRC" O="$BUILD_DIR" LLVM=1 x86_64_defconfig
fi

"$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
    -e RUST \
    -e MODULES \
    -e MODULE_UNLOAD \
    -e DEVTMPFS \
    -e DEVTMPFS_MOUNT \
    -e CONFIGFS_FS \
    -e BLK_DEV \
    -d SAMPLES \
    -d SAMPLES_RUST \
    -d SAMPLE_RUST_MINIMAL \
    -d SAMPLE_RUST_PRINT \
    -d SAMPLE_RUST_HOSTPROGS \
    -d BLK_DEV_ZONED \
    -d BLK_DEV_NULL_BLK_FAULT_INJECTION \
    -d DEBUG_INFO_BTF

case "$IMPLEMENTATION" in
    c)
        "$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
            -m BLK_DEV_NULL_BLK \
            -d BLK_DEV_RUST_NULL
        ;;
    rust)
        "$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
            -d BLK_DEV_NULL_BLK \
            -m BLK_DEV_RUST_NULL
        ;;
    *)
        echo "unknown implementation: $IMPLEMENTATION" >&2
        exit 1
        ;;
esac

case "$PROFILE" in
    memory-debug)
        "$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
            -e FRAME_POINTER \
            -e KASAN \
            -e KASAN_GENERIC \
            -e SLUB_DEBUG \
            -e SLUB_DEBUG_ON \
            -e DEBUG_OBJECTS \
            -e REFCOUNT_FULL
        ;;
    concurrency-debug)
        "$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
            -e FRAME_POINTER \
            -e PROVE_LOCKING \
            -e PROVE_RCU \
            -e DEBUG_ATOMIC_SLEEP \
            -e DEBUG_SPINLOCK
        ;;
    leak-debug)
        "$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
            -e FRAME_POINTER \
            -e DEBUG_KMEMLEAK
        ;;
    *)
        echo "unknown profile: $PROFILE" >&2
        exit 1
        ;;
esac

make -C "$KERNEL_SRC" O="$BUILD_DIR" LLVM=1 olddefconfig

echo "Configured $PROFILE / $IMPLEMENTATION in $BUILD_DIR"
