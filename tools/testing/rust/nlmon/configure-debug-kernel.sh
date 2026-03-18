#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: configure-debug-kernel.sh <memory-debug|concurrency-debug|leak-debug> [build-dir]

Creates a dedicated nlmon debug-kernel build directory and applies the fixed
configuration profile required by the stage-two test plan.
EOF
}

PROFILE="${1:-}"
BUILD_DIR="${2:-}"
BASE_CONFIG="${BASE_CONFIG:-$RFL_DEV_ROOT/build/.config}"

if [[ -z "$PROFILE" ]]; then
    usage >&2
    exit 1
fi

if [[ -z "$BUILD_DIR" ]]; then
    BUILD_DIR="$(nlmon_default_build_dir "$PROFILE")"
fi

mkdir -p "$BUILD_DIR"

if [[ -f "$BASE_CONFIG" ]]; then
    cp "$BASE_CONFIG" "$BUILD_DIR/.config"
else
    make -C "$KERNEL_SRC" O="$BUILD_DIR" LLVM=1 x86_64_defconfig
fi

"$KERNEL_SRC/scripts/config" --file "$BUILD_DIR/.config" \
    -e RUST \
    -e MODULES \
    -e MODULE_UNLOAD \
    -e DEVTMPFS \
    -e DEVTMPFS_MOUNT \
    -e DEBUG_FS \
    -e NET_NS \
    -e PACKET \
    -e IPV6 \
    -e IP_ADVANCED_ROUTER \
    -e IP_MULTIPLE_TABLES \
    -m NLMON \
    -m DUMMY \
    -m VETH \
    -m BRIDGE \
    -m STP \
    -m LLC \
    -d DEBUG_INFO_BTF

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
            -e DEBUG_SPINLOCK \
            -e DEBUG_NET
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

require_config() {
    local symbol="$1"
    if ! grep -Eq "^CONFIG_${symbol}=(y|m)$" "$BUILD_DIR/.config"; then
        echo "warning: CONFIG_${symbol} is not enabled in $BUILD_DIR/.config" >&2
    fi
}

case "$PROFILE" in
    memory-debug)
        require_config KASAN
        require_config KASAN_GENERIC
        require_config SLUB_DEBUG_ON
        require_config DEBUG_OBJECTS
        require_config REFCOUNT_FULL
        ;;
    concurrency-debug)
        require_config PROVE_LOCKING
        require_config PROVE_RCU
        require_config DEBUG_ATOMIC_SLEEP
        require_config DEBUG_SPINLOCK
        require_config DEBUG_NET
        ;;
    leak-debug)
        require_config DEBUG_KMEMLEAK
        ;;
esac

echo "Configured $PROFILE kernel in $BUILD_DIR"
