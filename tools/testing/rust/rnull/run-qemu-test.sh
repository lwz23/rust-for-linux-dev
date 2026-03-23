#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: run-qemu-test.sh --build-dir <dir> --rootfs-image <path> --log-file <path> [options]

Options:
  --timeout-seconds <n>   Default: 900
  --memory-mb <n>         Default: 4096
  --cpus <n>              Default: 4
EOF
}

BUILD_DIR=""
ROOTFS_IMAGE=""
LOG_FILE=""
TIMEOUT_SECONDS="900"
MEMORY_MB="4096"
CPUS="4"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --build-dir)
            BUILD_DIR="$2"
            shift 2
            ;;
        --rootfs-image)
            ROOTFS_IMAGE="$2"
            shift 2
            ;;
        --log-file)
            LOG_FILE="$2"
            shift 2
            ;;
        --timeout-seconds)
            TIMEOUT_SECONDS="$2"
            shift 2
            ;;
        --memory-mb)
            MEMORY_MB="$2"
            shift 2
            ;;
        --cpus)
            CPUS="$2"
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

if [[ -z "$BUILD_DIR" || -z "$ROOTFS_IMAGE" || -z "$LOG_FILE" ]]; then
    usage >&2
    exit 1
fi

mkdir -p "$(dirname "$LOG_FILE")"

QEMU_ARGS=(
    -nographic
    -m "$MEMORY_MB"
    -smp "$CPUS"
    -kernel "$BUILD_DIR/arch/x86/boot/bzImage"
    -initrd "$ROOTFS_IMAGE"
    -append "console=ttyS0 rdinit=/init printk.devkmsg=on loglevel=7 panic=-1"
    -no-reboot
)

if [[ -r /dev/kvm ]]; then
    QEMU_ARGS=(-enable-kvm "${QEMU_ARGS[@]}")
fi

timeout "${TIMEOUT_SECONDS}s" qemu-system-x86_64 "${QEMU_ARGS[@]}" >"$LOG_FILE" 2>&1

echo "QEMU log written to $LOG_FILE"
