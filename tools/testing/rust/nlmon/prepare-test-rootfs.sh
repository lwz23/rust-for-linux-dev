#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: prepare-test-rootfs.sh --build-dir <dir> --implementation <c|rust> [options]

Options:
  --rootfs-dir <dir>          Staging directory (default: /home/lwz/rfl-dev/rootfs/nlmon-test-stage)
  --rootfs-image <path>       Output initramfs image (default: /home/lwz/rfl-dev/rootfs/initramfs-nlmon-test.cpio.gz)
  --scenario <name>           baseline|lifecycle|matrix|stress|all (default: baseline)
  --lifecycle-loops <n>       Default: 100
  --generator-rounds <n>      Default: 50
  --stress-seconds <n>        Default: 1800
EOF
}

BUILD_DIR=""
IMPLEMENTATION=""
ROOTFS_DIR=""
ROOTFS_IMAGE=""
SCENARIO="baseline"
LIFECYCLE_LOOPS="100"
GENERATOR_ROUNDS="50"
STRESS_SECONDS="1800"

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
        --generator-rounds)
            GENERATOR_ROUNDS="$2"
            shift 2
            ;;
        --stress-seconds)
            STRESS_SECONDS="$2"
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
    ROOTFS_DIR="$(nlmon_default_rootfs_dir)"
fi

if [[ -z "$ROOTFS_IMAGE" ]]; then
    ROOTFS_IMAGE="$(nlmon_default_rootfs_image)"
fi

case "$IMPLEMENTATION" in
    c) NLMON_MODULE_BASENAME="nlmon.ko" ;;
    rust) NLMON_MODULE_BASENAME="nlmon_rust.ko" ;;
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
    "$ROOTFS_DIR/usr/libexec" \
    "$ROOTFS_DIR/usr/sbin"

nlmon_stage_busybox_applets

for bin_path in \
    /usr/bin/ip \
    /usr/bin/tcpdump \
    /usr/bin/sha256sum \
    /usr/bin/cmp \
    /usr/bin/awk \
    /usr/bin/sed \
    /usr/bin/grep
do
    nlmon_install_binary_with_deps "$bin_path"
done

if [[ -x /usr/sbin/ethtool ]]; then
    nlmon_install_binary_with_deps /usr/sbin/ethtool
    HAVE_ETHTOOL=1
else
    HAVE_ETHTOOL=0
    echo "warning: /usr/sbin/ethtool is unavailable on the host; guest observations will record this downgrade" >&2
fi

if [[ -d /etc/iproute2 ]]; then
    mkdir -p "$ROOTFS_DIR/etc/iproute2"
    cp -a /etc/iproute2/. "$ROOTFS_DIR/etc/iproute2/"
fi

nlmon_stage_module "$BUILD_DIR" drivers/net/dummy.ko dummy.ko
nlmon_stage_module "$BUILD_DIR" net/llc/llc.ko llc.ko
nlmon_stage_module "$BUILD_DIR" net/802/stp.ko stp.ko
nlmon_stage_module "$BUILD_DIR" net/bridge/bridge.ko bridge.ko
nlmon_stage_module "$BUILD_DIR" drivers/net/veth.ko veth.ko
nlmon_stage_module "$BUILD_DIR" "drivers/net/${NLMON_MODULE_BASENAME}" "$NLMON_MODULE_BASENAME"

install -m 0755 "$SCRIPT_DIR/nlmon-guest-runner.sh" "$ROOTFS_DIR/usr/libexec/nlmon-guest-runner.sh"

cat > "$ROOTFS_DIR/etc/nlmon-test.env" <<EOF
NLMON_IMPLEMENTATION=$IMPLEMENTATION
NLMON_MODULE=/lib/modules/$NLMON_MODULE_BASENAME
NLMON_SCENARIO=$SCENARIO
NLMON_LIFECYCLE_LOOPS=$LIFECYCLE_LOOPS
NLMON_GENERATOR_ROUNDS=$GENERATOR_ROUNDS
NLMON_STRESS_SECONDS=$STRESS_SECONDS
NLMON_HAVE_ETHTOOL=$HAVE_ETHTOOL
EOF

cat > "$ROOTFS_DIR/init" <<'EOF'
#!/bin/sh

set -eu

export PATH=/usr/sbin:/usr/bin:/bin:/sbin

mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev

if [ -e /proc/sys/kernel/hotplug ]; then
    echo /bin/mdev > /proc/sys/kernel/hotplug
fi
mdev -s

echo "=== nlmon automated test init ==="
uname -a

if /usr/libexec/nlmon-guest-runner.sh; then
    rc=0
else
    rc=$?
fi

echo "=== nlmon automated test exit rc=$rc ==="
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

echo "Prepared nlmon test rootfs at $ROOTFS_DIR"
echo "Built nlmon test initramfs at $ROOTFS_IMAGE"
