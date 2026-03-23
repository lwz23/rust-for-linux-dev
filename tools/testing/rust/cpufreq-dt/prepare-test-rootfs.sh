#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: prepare-test-rootfs.sh --build-dir <dir> --implementation <c|rust> --env <sabrelite|mcimx7d-sabre> [options]

Options:
  --rootfs-dir <dir>     Staging directory
  --rootfs-image <path>  Output initramfs image
EOF
}

BUILD_DIR=""
IMPLEMENTATION=""
RUNTIME_ENV=""
ROOTFS_DIR=""
ROOTFS_IMAGE=""

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
        --env)
            RUNTIME_ENV="$2"
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

if [[ -z "$BUILD_DIR" || -z "$IMPLEMENTATION" || -z "$RUNTIME_ENV" ]]; then
    usage >&2
    exit 1
fi

if [[ -z "$ROOTFS_DIR" ]]; then
    ROOTFS_DIR="$(cpufreq_dt_default_rootfs_dir)"
fi

if [[ -z "$ROOTFS_IMAGE" ]]; then
    ROOTFS_IMAGE="$(cpufreq_dt_default_rootfs_image)"
fi

DRIVER_MODULE_BASENAME="$(cpufreq_dt_driver_module_basename "$IMPLEMENTATION")"
DRIVER_MODULE_RELPATH="drivers/cpufreq/$DRIVER_MODULE_BASENAME"
GOV_MODULE_RELPATH="drivers/cpufreq/cpufreq_userspace.ko"
PROVIDER_MODULE_RELPATH="$(cpufreq_dt_provider_module_relpath "$RUNTIME_ENV")"

rm -rf "$ROOTFS_DIR"
mkdir -p \
    "$ROOTFS_DIR/dev" \
    "$ROOTFS_DIR/etc" \
    "$ROOTFS_DIR/lib/modules" \
    "$ROOTFS_DIR/proc" \
    "$ROOTFS_DIR/run" \
    "$ROOTFS_DIR/sys" \
    "$ROOTFS_DIR/tmp"

cat >"$ROOTFS_DIR/etc/passwd" <<'EOF'
root:x:0:0:root:/root:/bin/sh
EOF

cat >"$ROOTFS_DIR/etc/group" <<'EOF'
root:x:0:
EOF

cpufreq_dt_stage_module "$BUILD_DIR" "$DRIVER_MODULE_RELPATH" "$DRIVER_MODULE_BASENAME"
cpufreq_dt_stage_module "$BUILD_DIR" "$GOV_MODULE_RELPATH" "cpufreq_userspace.ko"

if [[ -n "$PROVIDER_MODULE_RELPATH" ]]; then
    cpufreq_dt_stage_module \
        "$BUILD_DIR" \
        "$PROVIDER_MODULE_RELPATH" \
        "$(basename "$PROVIDER_MODULE_RELPATH")"
    PROVIDER_MODULE="/lib/modules/$(basename "$PROVIDER_MODULE_RELPATH")"
else
    PROVIDER_MODULE=""
fi

cat >"$ROOTFS_DIR/etc/cpufreq-dt-test.env" <<EOF
CPUFREQ_DT_IMPLEMENTATION=$IMPLEMENTATION
CPUFREQ_DT_ENV=$RUNTIME_ENV
CPUFREQ_DT_DRIVER_MODULE=/lib/modules/$DRIVER_MODULE_BASENAME
CPUFREQ_DT_GOV_MODULE=/lib/modules/cpufreq_userspace.ko
CPUFREQ_DT_PROVIDER_MODULE=$PROVIDER_MODULE
CPUFREQ_DT_EXPECTED_SCALING_DRIVER=cpufreq-dt
EOF

cpufreq_dt_build_guest_init "$ROOTFS_DIR/init"

mkdir -p "$(dirname "$ROOTFS_IMAGE")"
(
    cd "$ROOTFS_DIR"
    find . -print0 \
        | cpio --null -ov --format=newc 2>/dev/null \
        | gzip -9 >"$ROOTFS_IMAGE"
)

echo "Prepared cpufreq-dt test rootfs at $ROOTFS_DIR"
echo "Built cpufreq-dt test initramfs at $ROOTFS_IMAGE"
