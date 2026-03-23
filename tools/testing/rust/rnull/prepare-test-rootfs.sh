#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: prepare-test-rootfs.sh --build-dir <dir> --implementation <c|rust> [options]

Options:
  --rootfs-dir <dir>          Staging directory
  --rootfs-image <path>       Output initramfs image
  --scenario <name>           baseline|lifecycle|matrix-core|io-core
  --lifecycle-loops <n>       Default: 8
EOF
}

BUILD_DIR=""
IMPLEMENTATION=""
ROOTFS_DIR=""
ROOTFS_IMAGE=""
SCENARIO="baseline"
LIFECYCLE_LOOPS="8"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --build-dir)
            BUILD_DIR="$2"
            shift 2
            ;;
        --implementation)
            IMPLEMENTATION="$2"
            shift 2
            ;;
        --rootfs-dir)
            ROOTFS_DIR="$2"
            shift 2
            ;;
        --rootfs-image)
            ROOTFS_IMAGE="$2"
            shift 2
            ;;
        --scenario)
            SCENARIO="$2"
            shift 2
            ;;
        --lifecycle-loops)
            LIFECYCLE_LOOPS="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [[ -z "$BUILD_DIR" || -z "$IMPLEMENTATION" ]]; then
    usage >&2
    exit 1
fi

if [[ -z "$ROOTFS_DIR" ]]; then
    ROOTFS_DIR="$(rnull_default_rootfs_dir)"
fi

if [[ -z "$ROOTFS_IMAGE" ]]; then
    ROOTFS_IMAGE="$(rnull_default_rootfs_image)"
fi

case "$IMPLEMENTATION" in
    c)
        TARGET_MODULE_NAME="null_blk"
        TARGET_MODULE_BASENAME="null_blk.ko"
        TARGET_MODULE_REL="drivers/block/null_blk/null_blk.ko"
        ;;
    rust)
        TARGET_MODULE_NAME="rnull"
        TARGET_MODULE_BASENAME="rnull.ko"
        TARGET_MODULE_REL="drivers/block/rnull/rnull.ko"
        ;;
    *)
        echo "unknown implementation: $IMPLEMENTATION" >&2
        exit 1
        ;;
esac

rm -rf "$ROOTFS_DIR"
mkdir -p \
    "$ROOTFS_DIR/bin" \
    "$ROOTFS_DIR/dev" \
    "$ROOTFS_DIR/etc" \
    "$ROOTFS_DIR/lib/modules" \
    "$ROOTFS_DIR/proc" \
    "$ROOTFS_DIR/run" \
    "$ROOTFS_DIR/sbin" \
    "$ROOTFS_DIR/sys" \
    "$ROOTFS_DIR/tmp" \
    "$ROOTFS_DIR/usr/bin" \
    "$ROOTFS_DIR/usr/libexec"

cat > "$ROOTFS_DIR/etc/passwd" <<'EOF'
root:x:0:0:root:/root:/bin/sh
EOF

cat > "$ROOTFS_DIR/etc/group" <<'EOF'
root:x:0:
EOF

rnull_stage_busybox_applets

for bin_path in \
    /usr/sbin/blockdev \
    /usr/bin/lsblk
do
    rnull_install_binary_with_deps "$bin_path"
done

guest_tool="$(mktemp "$SCRIPT_DIR/rnull-io-tool.XXXXXX")"
trap 'rm -f "$guest_tool"' EXIT
rnull_build_guest_tool "$guest_tool"
rnull_install_binary_with_deps "$guest_tool"
mv "$ROOTFS_DIR$guest_tool" "$ROOTFS_DIR/usr/bin/rnull-io-tool"
rm -f "$ROOTFS_DIR$guest_tool"

rnull_stage_module "$BUILD_DIR" "$TARGET_MODULE_REL" "$TARGET_MODULE_BASENAME"
if [[ -f "$BUILD_DIR/fs/configfs/configfs.ko" ]]; then
    rnull_stage_module "$BUILD_DIR" "fs/configfs/configfs.ko" "configfs.ko"
fi

install -m 0755 "$SCRIPT_DIR/rnull-guest-runner.sh" "$ROOTFS_DIR/usr/libexec/rnull-guest-runner.sh"

cat > "$ROOTFS_DIR/etc/rnull-test.env" <<EOF
RNULL_IMPLEMENTATION=$IMPLEMENTATION
RNULL_MODULE=/lib/modules/$TARGET_MODULE_BASENAME
RNULL_MODULE_NAME=$TARGET_MODULE_NAME
RNULL_SCENARIO=$SCENARIO
RNULL_LIFECYCLE_LOOPS=$LIFECYCLE_LOOPS
EOF

cat > "$ROOTFS_DIR/init" <<'EOF'
#!/bin/sh

set -eu

export PATH=/usr/sbin:/usr/bin:/bin:/sbin

mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev

mkdir -p /sys/kernel/config
if [ -f /lib/modules/configfs.ko ]; then
    insmod /lib/modules/configfs.ko || true
fi
mount -t configfs none /sys/kernel/config || true

if [ -e /proc/sys/kernel/hotplug ]; then
    echo /bin/mdev > /proc/sys/kernel/hotplug
fi
mdev -s

echo "=== rnull automated test init ==="
uname -a

if /usr/libexec/rnull-guest-runner.sh; then
    rc=0
else
    rc=$?
fi

echo "=== rnull automated test exit rc=$rc ==="
sync
poweroff -f || reboot -f || exit "$rc"
EOF

chmod 0755 "$ROOTFS_DIR/init"

mkdir -p "$(dirname "$ROOTFS_IMAGE")"
(
    cd "$ROOTFS_DIR"
    find . -print0 \
        | cpio --null -ov --format=newc 2>/dev/null \
        | gzip -9 > "$ROOTFS_IMAGE"
)

echo "Prepared rnull test rootfs at $ROOTFS_DIR"
echo "Built rnull test initramfs at $ROOTFS_IMAGE"
