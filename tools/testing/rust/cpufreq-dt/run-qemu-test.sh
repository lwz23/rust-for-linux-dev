#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage: run-qemu-test.sh --build-dir <dir> --rootfs-image <path> --log-file <path> --env <sabrelite|mcimx7d-sabre> [options]

Options:
  --timeout-seconds <n>   Default: 600
  --memory-mb <n>         Default: 1024
EOF
}

BUILD_DIR=""
ROOTFS_IMAGE=""
LOG_FILE=""
RUNTIME_ENV=""
TIMEOUT_SECONDS="600"
MEMORY_MB="1024"

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
        --env)
            RUNTIME_ENV="$2"
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

if [[ -z "$BUILD_DIR" || -z "$ROOTFS_IMAGE" || -z "$LOG_FILE" || -z "$RUNTIME_ENV" ]]; then
    usage >&2
    exit 1
fi

case "$RUNTIME_ENV" in
    sabrelite)
        MACHINE="sabrelite"
        DTB_REL="arch/arm/boot/dts/nxp/imx/imx6q-sabrelite.dtb"
        SMP_CPUS="4"
        CONSOLE="ttymxc1"
        SERIAL_ARGS=(-serial null -serial stdio)
        ;;
    mcimx7d-sabre)
        MACHINE="mcimx7d-sabre"
        DTB_REL="arch/arm/boot/dts/nxp/imx/imx7d-sdb.dtb"
        SMP_CPUS="2"
        CONSOLE="ttymxc0"
        SERIAL_ARGS=(-serial stdio)
        ;;
    *)
        echo "unknown runtime env: $RUNTIME_ENV" >&2
        exit 1
        ;;
esac

KERNEL_IMAGE="$BUILD_DIR/arch/arm/boot/zImage"
DTB_IMAGE="$BUILD_DIR/$DTB_REL"

if [[ ! -f "$KERNEL_IMAGE" ]]; then
    echo "kernel image not found: $KERNEL_IMAGE" >&2
    exit 2
fi

if [[ ! -f "$DTB_IMAGE" ]]; then
    echo "dtb not found: $DTB_IMAGE" >&2
    exit 3
fi

mkdir -p "$(dirname "$LOG_FILE")"

QEMU_ARGS=(
    -display none
    -monitor none
    "${SERIAL_ARGS[@]}"
    -no-reboot
    -M "$MACHINE"
    -m "$MEMORY_MB"
    -smp "$SMP_CPUS"
    -kernel "$KERNEL_IMAGE"
    -dtb "$DTB_IMAGE"
    -initrd "$ROOTFS_IMAGE"
    -append "console=$CONSOLE,115200 rdinit=/init printk.devkmsg=on loglevel=8 panic=-1"
)

QEMU_RC=""
QEMU_FORCED_STOP=0
QEMU_PID=""

: >"$LOG_FILE"
qemu-system-arm "${QEMU_ARGS[@]}" >"$LOG_FILE" 2>&1 &
QEMU_PID=$!

for ((elapsed = 0; elapsed < TIMEOUT_SECONDS; elapsed++)); do
    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        set +e
        wait "$QEMU_PID"
        QEMU_RC=$?
        set -e
        break
    fi

    if grep -q 'CPUFREQ_DT_GUEST_RC=' "$LOG_FILE"; then
        sleep 2
        if kill -0 "$QEMU_PID" 2>/dev/null; then
            QEMU_FORCED_STOP=1
            kill "$QEMU_PID" 2>/dev/null || true
            set +e
            wait "$QEMU_PID"
            QEMU_RC=$?
            set -e
        else
            set +e
            wait "$QEMU_PID"
            QEMU_RC=$?
            set -e
        fi
        break
    fi

    sleep 1
done

if [[ -z "$QEMU_RC" ]]; then
    if kill -0 "$QEMU_PID" 2>/dev/null; then
        kill "$QEMU_PID" 2>/dev/null || true
        sleep 1
        kill -9 "$QEMU_PID" 2>/dev/null || true
        set +e
        wait "$QEMU_PID"
        QEMU_RC=$?
        set -e
    else
        set +e
        wait "$QEMU_PID"
        QEMU_RC=$?
        set -e
    fi
    echo "QEMU_EXIT_CODE=$QEMU_RC" >>"$LOG_FILE"
    echo "QEMU_FORCED_STOP=$QEMU_FORCED_STOP" >>"$LOG_FILE"
    echo "qemu timed out after ${TIMEOUT_SECONDS}s" >&2
    exit 124
fi

echo "QEMU_EXIT_CODE=$QEMU_RC" >>"$LOG_FILE"
echo "QEMU_FORCED_STOP=$QEMU_FORCED_STOP" >>"$LOG_FILE"

if [[ $QEMU_RC -ne 0 && $QEMU_RC -ne 143 && $QEMU_RC -ne 137 ]]; then
    echo "qemu exited with rc=$QEMU_RC" >&2
    exit "$QEMU_RC"
fi

GUEST_RC="$(sed -n 's/.*CPUFREQ_DT_GUEST_RC=//p' "$LOG_FILE" | tail -n 1 | tr -d '\r')"
if [[ -z "$GUEST_RC" ]]; then
    echo "guest runner exit marker missing from $LOG_FILE" >&2
    exit 125
fi

if [[ "$GUEST_RC" -ne 0 ]]; then
    echo "guest runner exited with rc=$GUEST_RC" >&2
    exit "$GUEST_RC"
fi

echo "QEMU log written to $LOG_FILE"
