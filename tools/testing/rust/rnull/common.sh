#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-2.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source /home/lwz/rfl-dev/env.sh
export KERNEL_SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

rnull_default_build_dir() {
    echo "$RFL_DEV_ROOT/build-rnull-fresh-validation"
}

rnull_default_rootfs_dir() {
    echo "$RFL_DEV_ROOT/rootfs/rnull-test-stage"
}

rnull_default_rootfs_image() {
    echo "$RFL_DEV_ROOT/rootfs/initramfs-rnull-test.cpio.gz"
}

rnull_copy_path_with_parents() {
    local src="$1"
    local dst="$ROOTFS_DIR$src"
    local resolved_src

    resolved_src="$(readlink -f "$src")"

    mkdir -p "$(dirname "$dst")"
    cp -a "$resolved_src" "$dst"
}

rnull_install_binary_with_deps() {
    local bin_path="$1"

    if [[ ! -x "$bin_path" ]]; then
        echo "binary not found or not executable: $bin_path" >&2
        return 1
    fi

    rnull_copy_path_with_parents "$bin_path"

    while read -r dep; do
        [[ -n "$dep" ]] || continue
        rnull_copy_path_with_parents "$dep"
    done < <(
        ldd "$bin_path" \
            | awk '
                $3 ~ /^\// { print $3 }
                $1 ~ /^\// { print $1 }
            ' \
            | sort -u
    )
}

rnull_stage_module() {
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

rnull_stage_busybox_applets() {
    local applet

    install -m 0755 /usr/bin/busybox "$ROOTFS_DIR/bin/busybox"

    for applet in \
        sh mount umount mkdir mknod insmod rmmod modprobe dmesg uname ls cat echo sleep \
        mdev date kill ps head wc cut sort tr rm cp mv chmod touch sync poweroff reboot \
        grep awk sed printf basename dirname dd true false test
    do
        ln -sf /bin/busybox "$ROOTFS_DIR/bin/$applet"
    done

    for applet in insmod rmmod modprobe mdev poweroff reboot
    do
        ln -sf /bin/busybox "$ROOTFS_DIR/sbin/$applet"
    done
}

rnull_build_guest_tool() {
    local output="$1"

    cc -O2 -D_GNU_SOURCE -Wall -Wextra -o "$output" "$SCRIPT_DIR/rnull-io-tool.c"
}
