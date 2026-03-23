#!/bin/sh
# SPDX-License-Identifier: GPL-2.0

set -eu

PATH=/usr/sbin:/usr/bin:/bin:/sbin
RESULT_DIR=/tmp/rnull-results
CONFIGFS_ROOT=/sys/kernel/config/nullb
IO_TOOL=/usr/bin/rnull-io-tool
mkdir -p "$RESULT_DIR"

. /etc/rnull-test.env

BLOCKDEV_BIN=/usr/sbin/blockdev
if [ ! -x "$BLOCKDEV_BIN" ]; then
    BLOCKDEV_BIN=/usr/bin/blockdev
fi

log() {
    echo "RNULL_TEST: $*"
}

emit() {
    echo "RNULL_RESULT: $*"
}

wait_for_path() {
    path="$1"
    want_present="$2"
    timeout_secs="${3:-20}"
    wait_i=0

    log "wait_for_path path=$path present=$want_present timeout=$timeout_secs"

    while [ "$wait_i" -lt "$timeout_secs" ]; do
        if [ "$want_present" = "1" ] && [ -e "$path" ]; then
            log "wait_for_path path=$path satisfied=1 after=${wait_i}s"
            return 0
        fi
        if [ "$want_present" = "0" ] && [ ! -e "$path" ]; then
            log "wait_for_path path=$path satisfied=1 after=${wait_i}s"
            return 0
        fi
        sleep 1
        wait_i=$((wait_i + 1))
    done

    log "timeout waiting for path=$path present=$want_present"
    return 1
}

load_target_module() {
    if grep -q "^$RNULL_MODULE_NAME " /proc/modules; then
        return 0
    fi

    insmod "$RNULL_MODULE"
    emit "module.loaded=$RNULL_MODULE_NAME"
}

cleanup_configfs_items() {
    if [ ! -d "$CONFIGFS_ROOT" ]; then
        return 0
    fi

    for item in "$CONFIGFS_ROOT"/*; do
        [ -d "$item" ] || continue
        if [ -f "$item/power" ]; then
            echo 0 >"$item/power" 2>/dev/null || true
        fi
        rmdir "$item" 2>/dev/null || true
    done
}

unload_target_module() {
    cleanup_configfs_items
    if grep -q "^$RNULL_MODULE_NAME " /proc/modules; then
        rmmod "$RNULL_MODULE_NAME"
        emit "module.unloaded=$RNULL_MODULE_NAME"
    fi
}

cleanup() {
    cleanup_configfs_items || true
    unload_target_module || true
}

trap cleanup EXIT

create_cfg_device() {
    name="$1"
    mkdir "$CONFIGFS_ROOT/$name"
}

set_attr_ok() {
    name="$1"
    attr="$2"
    value="$3"
    printf '%s\n' "$value" >"$CONFIGFS_ROOT/$name/$attr"
}

set_attr_fail() {
    name="$1"
    attr="$2"
    value="$3"
    key="$4"

    if printf '%s\n' "$value" >"$CONFIGFS_ROOT/$name/$attr" 2>/dev/null; then
        emit "$key=unexpected_success"
        return 1
    fi

    emit "$key=expected_failure"
    return 0
}

get_attr() {
    name="$1"
    attr="$2"
    cat "$CONFIGFS_ROOT/$name/$attr"
}

power_on_device() {
    name="$1"
    set_attr_ok "$name" power 1
    wait_for_path "/dev/$name" 1 30
}

power_off_device() {
    name="$1"
    if [ -f "$CONFIGFS_ROOT/$name/power" ]; then
        echo 0 >"$CONFIGFS_ROOT/$name/power" 2>/dev/null || true
    fi
    wait_for_path "/dev/$name" 0 30 || true
}

remove_cfg_device() {
    name="$1"
    power_off_device "$name"
    rmdir "$CONFIGFS_ROOT/$name"
}

scan_dmesg() {
    tag="$1"
    anomaly_pattern="BUG:|WARNING:|Oops:|KASAN:|KFENCE:|KCSAN:|UBSAN:|use-after-free|double[- ]free|lockdep:|possible recursive locking detected|suspicious RCU usage|refcount_t:|kmemleak: [0-9]+ new suspected memory leaks|unreferenced object|DEBUG_OBJECTS|bad unlock balance"

    dmesg >"$RESULT_DIR/$tag.dmesg.txt" 2>&1 || true
    if grep -E "$anomaly_pattern" "$RESULT_DIR/$tag.dmesg.txt" >"$RESULT_DIR/$tag.dmesg.anomalies.txt"; then
        emit "dmesg_anomaly.$tag=1"
        head -n 5 "$RESULT_DIR/$tag.dmesg.anomalies.txt" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g; s/[[:space:]]$//' \
            | sed "s/^/RNULL_RESULT: dmesg_anomaly_head.$tag=/" 
    else
        emit "dmesg_anomaly.$tag=0"
    fi
}

timed_write_ns() {
    dev="$1"
    offset="$2"
    bytes="$3"
    seed="$4"
    hipri="${5:-0}"

    "$IO_TOOL" timed-write "$dev" "$offset" "$bytes" "$seed" "$hipri" \
        | sed -n 's/^elapsed_ns=//p'
}

run_baseline() {
    wait_for_path "$CONFIGFS_ROOT" 1 15
    wait_for_path /dev/nullb0 1 30
    emit "baseline.default_device=/dev/nullb0"
    emit "baseline.size_bytes=$("$BLOCKDEV_BIN" --getsize64 /dev/nullb0)"
    emit "baseline.logical_block_size=$(cat /sys/block/nullb0/queue/logical_block_size)"
    emit "baseline.rotational=$(cat /sys/block/nullb0/queue/rotational)"

    unload_target_module
    wait_for_path /dev/nullb0 0 30

    load_target_module
    wait_for_path "$CONFIGFS_ROOT" 1 15
    wait_for_path /dev/nullb0 1 30
    emit "baseline.reload=ok"
    scan_dmesg baseline
}

run_lifecycle() {
    lifecycle_i=1
    while [ "$lifecycle_i" -le "$RNULL_LIFECYCLE_LOOPS" ]; do
        name="cycle$lifecycle_i"
        log "lifecycle loop=$lifecycle_i step=create"
        create_cfg_device "$name"
        log "lifecycle loop=$lifecycle_i step=configure"
        set_attr_ok "$name" size 64
        set_attr_ok "$name" blocksize 4096
        set_attr_ok "$name" memory_backed 1
        set_attr_ok "$name" completion_nsec 2000000
        set_attr_ok "$name" irqmode 2
        log "lifecycle loop=$lifecycle_i step=power_on"
        power_on_device "$name"
        log "lifecycle loop=$lifecycle_i step=io"
        "$IO_TOOL" write-pattern "/dev/$name" 0 4096 17 >/dev/null
        log "lifecycle loop=$lifecycle_i step=power_off"
        power_off_device "$name"
        log "lifecycle loop=$lifecycle_i step=remove"
        remove_cfg_device "$name"
        lifecycle_i=$((lifecycle_i + 1))
    done

    emit "lifecycle.loops_completed=$RNULL_LIFECYCLE_LOOPS"
    scan_dmesg lifecycle
}

run_matrix_core() {
    log "matrix-core step=create.matrix0"
    create_cfg_device matrix0
    log "matrix-core step=configure.matrix0"
    set_attr_ok matrix0 size 64
    set_attr_ok matrix0 completion_nsec 1000000
    set_attr_ok matrix0 submit_queues 2
    set_attr_ok matrix0 poll_queues 0
    set_attr_ok matrix0 home_node 0
    set_attr_ok matrix0 queue_mode 0
    set_attr_ok matrix0 blocksize 4096
    set_attr_ok matrix0 max_sectors 128
    set_attr_ok matrix0 irqmode 2
    set_attr_ok matrix0 hw_queue_depth 64
    set_attr_ok matrix0 memory_backed 1
    set_attr_ok matrix0 discard 1
    set_attr_ok matrix0 cache_size 1
    set_attr_ok matrix0 mbps 8
    set_attr_ok matrix0 no_sched 1
    set_attr_ok matrix0 shared_tag_bitmap 1
    set_attr_ok matrix0 fua 1
    set_attr_ok matrix0 virt_boundary 1

    log "matrix-core step=power_on.matrix0"
    power_on_device matrix0
    emit "matrix0.queue_mode_after_power=$(get_attr matrix0 queue_mode | tr -d '\n')"
    emit "matrix0.blocking_after_power=$(get_attr matrix0 blocking | tr -d '\n')"
    emit "matrix0.shared_tag_bitmap=$(get_attr matrix0 shared_tag_bitmap | tr -d '\n')"

    set_attr_fail matrix0 blocksize 8192 matrix0.blocksize_reject
    set_attr_fail matrix0 memory_backed 0 matrix0.memory_backed_reject
    set_attr_fail matrix0 cache_size 2 matrix0.cache_size_reject

    if printf '3\n' >"$CONFIGFS_ROOT/matrix0/submit_queues"; then
        emit "matrix0.submit_queues_update=ok"
    else
        emit "matrix0.submit_queues_update=failed"
        return 1
    fi

    if printf '1\n' >"$CONFIGFS_ROOT/matrix0/poll_queues"; then
        emit "matrix0.poll_queues_update=ok"
    else
        emit "matrix0.poll_queues_update=failed"
        return 1
    fi

    log "matrix-core step=remove.matrix0"
    remove_cfg_device matrix0

    log "matrix-core step=create.matrix_rq"
    create_cfg_device matrix_rq
    set_attr_ok matrix_rq queue_mode 1
    if printf '1\n' >"$CONFIGFS_ROOT/matrix_rq/power" 2>/dev/null; then
        emit "matrix_rq.power_reject=unexpected_success"
        return 1
    fi
    emit "matrix_rq.power_reject=expected_failure"
    log "matrix-core step=remove.matrix_rq"
    remove_cfg_device matrix_rq

    log "matrix-core step=create.matrix_shared"
    create_cfg_device matrix_shared
    set_attr_ok matrix_shared size 64
    set_attr_ok matrix_shared blocksize 4096
    set_attr_ok matrix_shared memory_backed 1
    set_attr_ok matrix_shared submit_queues 2
    set_attr_ok matrix_shared poll_queues 0
    set_attr_ok matrix_shared shared_tags 1
    set_attr_ok matrix_shared shared_tag_bitmap 1
    log "matrix-core step=power_on.matrix_shared"
    power_on_device matrix_shared
    if printf '3\n' >"$CONFIGFS_ROOT/matrix_shared/submit_queues"; then
        emit "matrix_shared.submit_queues_update=ok"
    else
        emit "matrix_shared.submit_queues_update=failed"
        return 1
    fi
    if printf '1\n' >"$CONFIGFS_ROOT/matrix_shared/poll_queues"; then
        emit "matrix_shared.poll_queues_update=ok"
    else
        emit "matrix_shared.poll_queues_update=failed"
        return 1
    fi
    log "matrix-core step=remove.matrix_shared"
    remove_cfg_device matrix_shared

    scan_dmesg matrix_core
}

run_io_core() {
    log "io-core step=create.io0"
    create_cfg_device io0
    log "io-core step=configure.io0"
    set_attr_ok io0 size 64
    set_attr_ok io0 blocksize 4096
    set_attr_ok io0 memory_backed 1
    set_attr_ok io0 discard 1
    set_attr_ok io0 cache_size 1
    set_attr_ok io0 irqmode 1
    log "io-core step=power_on.io0"
    power_on_device io0

    log "io-core step=exercise.io0"
    "$IO_TOOL" write-pattern /dev/io0 0 8192 17 >/dev/null
    "$IO_TOOL" read-verify /dev/io0 0 8192 17 >/dev/null
    "$IO_TOOL" flush /dev/io0 >/dev/null
    "$IO_TOOL" discard /dev/io0 0 4096 >/dev/null
    "$IO_TOOL" read-zero /dev/io0 0 4096 >/dev/null
    emit "io0.basic_io=ok"

    set_attr_ok io0 badblocks +8-15
    if "$IO_TOOL" write-pattern /dev/io0 4096 4096 33 >/dev/null 2>&1; then
        emit "io0.badblocks_write_fail=unexpected_success"
        return 1
    fi
    emit "io0.badblocks_write_fail=expected_failure"
    set_attr_ok io0 badblocks -8-15
    "$IO_TOOL" write-pattern /dev/io0 4096 4096 33 >/dev/null
    "$IO_TOOL" read-verify /dev/io0 4096 4096 33 >/dev/null
    emit "io0.badblocks_clear=ok"
    log "io-core step=remove.io0"
    remove_cfg_device io0

    irq_mode=0
    while [ "$irq_mode" -le 2 ]; do
        name="ioirq$irq_mode"
        log "io-core step=create.$name"
        create_cfg_device "$name"
        set_attr_ok "$name" size 32
        set_attr_ok "$name" blocksize 4096
        set_attr_ok "$name" memory_backed 1
        set_attr_ok "$name" irqmode "$irq_mode"
        if [ "$irq_mode" -eq 2 ]; then
            set_attr_ok "$name" completion_nsec 2000000
        fi
        log "io-core step=power_on.$name"
        power_on_device "$name"
        "$IO_TOOL" write-pattern "/dev/$name" 0 4096 $((65 + irq_mode)) >/dev/null
        "$IO_TOOL" read-verify "/dev/$name" 0 4096 $((65 + irq_mode)) >/dev/null
        emit "$name.io=ok"
        log "io-core step=remove.$name"
        remove_cfg_device "$name"
        irq_mode=$((irq_mode + 1))
    done

    log "io-core step=create.iopoll"
    create_cfg_device iopoll
    set_attr_ok iopoll size 32
    set_attr_ok iopoll blocksize 4096
    set_attr_ok iopoll memory_backed 1
    set_attr_ok iopoll submit_queues 1
    set_attr_ok iopoll poll_queues 1
    log "io-core step=power_on.iopoll"
    power_on_device iopoll
    "$IO_TOOL" write-pattern /dev/iopoll 0 4096 99 1 >/dev/null
    "$IO_TOOL" read-verify /dev/iopoll 0 4096 99 1 >/dev/null
    emit "iopoll.hipri=ok"
    log "io-core step=remove.iopoll"
    remove_cfg_device iopoll

    log "io-core step=create.fast0"
    create_cfg_device fast0
    set_attr_ok fast0 size 64
    set_attr_ok fast0 blocksize 4096
    set_attr_ok fast0 memory_backed 1
    log "io-core step=power_on.fast0"
    power_on_device fast0
    fast_ns="$(timed_write_ns /dev/fast0 0 65536 51 0)"
    emit "fast0.elapsed_ns=$fast_ns"
    log "io-core step=remove.fast0"
    remove_cfg_device fast0

    log "io-core step=create.slow0"
    create_cfg_device slow0
    set_attr_ok slow0 size 64
    set_attr_ok slow0 blocksize 4096
    set_attr_ok slow0 memory_backed 1
    set_attr_ok slow0 mbps 4
    log "io-core step=power_on.slow0"
    power_on_device slow0
    slow_ns="$(timed_write_ns /dev/slow0 0 65536 52 0)"
    emit "slow0.elapsed_ns=$slow_ns"
    if [ "$slow_ns" -le "$fast_ns" ]; then
        emit "slow0.throttle=runtime_gap"
    else
        emit "slow0.throttle=ok"
    fi
    log "io-core step=remove.slow0"
    remove_cfg_device slow0

    scan_dmesg io_core
}

emit "implementation=$RNULL_IMPLEMENTATION"
emit "scenario=$RNULL_SCENARIO"

load_target_module

case "$RNULL_SCENARIO" in
    baseline)
        run_baseline
        ;;
    lifecycle)
        run_lifecycle
        ;;
    matrix-core)
        run_matrix_core
        ;;
    io-core)
        run_io_core
        ;;
    *)
        log "unknown scenario: $RNULL_SCENARIO"
        exit 1
        ;;
esac

emit "status=ok"
