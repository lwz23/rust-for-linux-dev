#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source /home/lwz/rfl-dev/env.sh

CPUFREQ_DT_RESULTS_ROOT="/home/lwz/rfl-dev/worktrees/cpufreq-dt-blind-rerun/test-results/cpufreq-dt"

cpufreq_dt_default_rootfs_dir() {
    echo "$RFL_DEV_ROOT/rootfs/cpufreq-dt-test-stage"
}

cpufreq_dt_default_rootfs_image() {
    echo "$RFL_DEV_ROOT/rootfs/initramfs-cpufreq-dt-test.cpio.gz"
}

cpufreq_dt_results_root() {
    echo "$CPUFREQ_DT_RESULTS_ROOT"
}

cpufreq_dt_driver_module_basename() {
    case "${1:-}" in
        c) echo "cpufreq-dt.ko" ;;
        rust) echo "rcpufreq_dt.ko" ;;
        *)
            echo "unknown implementation: ${1:-}" >&2
            return 1
            ;;
    esac
}

cpufreq_dt_provider_module_relpath() {
    case "${1:-}" in
        mcimx7d-sabre) echo "drivers/cpufreq/imx-cpufreq-dt.ko" ;;
        sabrelite) echo "" ;;
        *)
            echo "unknown runtime env: ${1:-}" >&2
            return 1
            ;;
    esac
}

cpufreq_dt_copy_path_with_parents() {
    local src="$1"
    local dst="$ROOTFS_DIR$src"
    local resolved_src

    resolved_src="$(readlink -f "$src")"

    mkdir -p "$(dirname "$dst")"
    cp -a "$resolved_src" "$dst"
}

cpufreq_dt_install_binary_with_deps() {
    local bin_path="$1"

    if [[ ! -x "$bin_path" ]]; then
        echo "binary not found or not executable: $bin_path" >&2
        return 1
    fi

    cpufreq_dt_copy_path_with_parents "$bin_path"

    while read -r dep; do
        [[ -n "$dep" ]] || continue
        cpufreq_dt_copy_path_with_parents "$dep"
    done < <(
        ldd "$bin_path" \
            | awk '
                $3 ~ /^\// { print $3 }
                $1 ~ /^\// { print $1 }
            ' \
            | sort -u
    )
}

cpufreq_dt_stage_module() {
    local build_dir="$1"
    local rel_path="$2"
    local dst_name="$3"
    local src="$build_dir/$rel_path"

    if [[ ! -f "$src" ]]; then
        echo "required module not found: $src" >&2
        return 1
    fi

    mkdir -p "$ROOTFS_DIR/lib/modules"
    install -m 0644 "$src" "$ROOTFS_DIR/lib/modules/$dst_name"
}

cpufreq_dt_build_guest_init() {
    local output_path="$1"
    local guest_src="$SCRIPT_DIR/cpufreq-dt-guest-init.c"

    if [[ ! -f "$guest_src" ]]; then
        echo "guest init source not found: $guest_src" >&2
        return 1
    fi

    mkdir -p "$(dirname "$output_path")"
    clang \
        --target=arm-linux-gnueabihf \
        -fuse-ld=lld \
        -fno-pie \
        -ffreestanding \
        -fno-stack-protector \
        -nostdlib \
        -static \
        -O2 \
        -Wl,-e,_start \
        "$guest_src" \
        -o "$output_path"
}
