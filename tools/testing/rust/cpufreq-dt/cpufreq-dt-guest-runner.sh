#!/bin/sh
# SPDX-License-Identifier: GPL-2.0

set -eu

PATH=/usr/sbin:/usr/bin:/bin:/sbin
RESULT_DIR=/tmp/cpufreq-dt-results
CPUFREQ_ROOT=/sys/devices/system/cpu/cpufreq

mkdir -p "$RESULT_DIR"
. /etc/cpufreq-dt-test.env

log() {
    echo "CPUFREQ_DT_TEST: $*"
}

emit() {
    echo "CPUFREQ_DT_RESULT: $*"
}

emit_file_value() {
    key="$1"
    file="$2"

    if [ -f "$file" ]; then
        emit "$key=$(tr '\n' ' ' <"$file" | sed 's/[[:space:]]\+/ /g; s/[[:space:]]$//')"
    fi
}

emit_file_excerpt() {
    key="$1"
    file="$2"
    lines="${3:-5}"

    if [ -f "$file" ]; then
        emit "$key=$(head -n "$lines" "$file" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g; s/[[:space:]]$//')"
    fi
}

load_one_module() {
    name="$1"
    path="$2"
    log_file="$RESULT_DIR/$name.insmod.txt"

    if [ -z "$path" ]; then
        emit "module.$name.skipped=1"
        return 0
    fi

    if insmod "$path" >"$log_file" 2>&1; then
        emit "module.$name.loaded=1"
        return 0
    fi

    emit "module.$name.loaded=0"
    emit_file_excerpt "module.$name.err" "$log_file" 20
    return 1
}

save_dmesg_views() {
    dmesg >"$RESULT_DIR/dmesg.txt" 2>&1 || true
    grep -i "cpufreq" "$RESULT_DIR/dmesg.txt" >"$RESULT_DIR/dmesg.cpufreq.txt" 2>/dev/null || true
    grep -i "cpufreq-dt" "$RESULT_DIR/dmesg.txt" >"$RESULT_DIR/dmesg.cpufreq-dt.txt" 2>/dev/null || true

    emit_file_excerpt "dmesg.cpufreq.head" "$RESULT_DIR/dmesg.cpufreq.txt" 20
    emit_file_excerpt "dmesg.cpufreq_dt.head" "$RESULT_DIR/dmesg.cpufreq-dt.txt" 20
}

scan_dmesg_for_anomalies() {
    anomaly_pattern="BUG:|WARNING:|Oops:|KASAN:|KFENCE:|KCSAN:|UBSAN:|use-after-free|double[- ]free|lockdep:|possible recursive locking detected|suspicious RCU usage|refcount_t:|kmemleak: [0-9]+ new suspected memory leaks|unreferenced object|bad unlock balance"

    if grep -E "$anomaly_pattern" "$RESULT_DIR/dmesg.txt" >"$RESULT_DIR/dmesg.anomalies.txt"; then
        emit "dmesg.anomaly=1"
    else
        emit "dmesg.anomaly=0"
    fi

    emit_file_excerpt "dmesg.anomaly.head" "$RESULT_DIR/dmesg.anomalies.txt" 20
}

wait_for_policy_root() {
    i=0
    while [ "$i" -lt 30 ]; do
        for policy in "$CPUFREQ_ROOT"/policy*; do
            if [ -d "$policy" ]; then
                return 0
            fi
        done
        sleep 1
        i=$((i + 1))
    done

    return 1
}

observe_file() {
    key="$1"
    file="$2"

    if [ -f "$file" ]; then
        emit_file_value "$key" "$file"
        return 0
    fi

    emit "$key=<missing>"
    return 1
}

attempt_userspace_roundtrip() {
    policy_name="$1"
    policy_dir="$2"
    original_governor="$(cat "$policy_dir/scaling_governor" 2>/dev/null || true)"
    available_governors="$(cat "$policy_dir/scaling_available_governors" 2>/dev/null || true)"
    available_freqs="$(cat "$policy_dir/scaling_available_frequencies" 2>/dev/null || true)"

    if echo "$available_governors" | grep -qw userspace; then
        emit "policy.userspace_available.$policy_name=1"
    else
        emit "policy.userspace_available.$policy_name=0"
        return 0
    fi

    if ! echo userspace >"$policy_dir/scaling_governor" 2>"$RESULT_DIR/$policy_name.userspace-governor.err"; then
        emit "policy.userspace_governor_switch.$policy_name=failed"
        emit_file_excerpt \
            "policy.userspace_governor_switch_err.$policy_name" \
            "$RESULT_DIR/$policy_name.userspace-governor.err" \
            10
        return 0
    fi

    emit "policy.userspace_governor_switch.$policy_name=ok"

    target_freq="$(echo "$available_freqs" | awk '{ print $1 }')"
    if [ -z "$target_freq" ] || [ ! -w "$policy_dir/scaling_setspeed" ]; then
        emit "policy.userspace_setspeed.$policy_name=unavailable"
    elif echo "$target_freq" >"$policy_dir/scaling_setspeed" 2>"$RESULT_DIR/$policy_name.setspeed.err"; then
        sleep 1
        emit "policy.userspace_setspeed.$policy_name=ok"
        emit "policy.userspace_target.$policy_name=$target_freq"
        if [ -f "$policy_dir/scaling_cur_freq" ]; then
            emit_file_value "policy.userspace_readback.$policy_name" "$policy_dir/scaling_cur_freq"
        elif [ -f "$policy_dir/cpuinfo_cur_freq" ]; then
            emit_file_value "policy.userspace_readback.$policy_name" "$policy_dir/cpuinfo_cur_freq"
        else
            emit "policy.userspace_readback.$policy_name=<missing>"
        fi
    else
        emit "policy.userspace_setspeed.$policy_name=failed"
        emit_file_excerpt "policy.userspace_setspeed_err.$policy_name" "$RESULT_DIR/$policy_name.setspeed.err" 10
    fi

    if [ -n "$original_governor" ]; then
        echo "$original_governor" >"$policy_dir/scaling_governor" 2>/dev/null || true
    fi
}

observe_policy() {
    policy_dir="$1"
    policy_name="$(basename "$policy_dir")"

    emit "policy.present.$policy_name=1"
    observe_file "policy.scaling_driver.$policy_name" "$policy_dir/scaling_driver" || true
    observe_file "policy.scaling_governor.$policy_name" "$policy_dir/scaling_governor" || true
    observe_file "policy.scaling_available_governors.$policy_name" "$policy_dir/scaling_available_governors" || true
    observe_file "policy.scaling_available_frequencies.$policy_name" "$policy_dir/scaling_available_frequencies" || true
    observe_file "policy.scaling_cur_freq.$policy_name" "$policy_dir/scaling_cur_freq" || true
    observe_file "policy.cpuinfo_cur_freq.$policy_name" "$policy_dir/cpuinfo_cur_freq" || true
    observe_file "policy.related_cpus.$policy_name" "$policy_dir/related_cpus" || true
    observe_file "policy.affected_cpus.$policy_name" "$policy_dir/affected_cpus" || true
    observe_file "policy.cpuinfo_min_freq.$policy_name" "$policy_dir/cpuinfo_min_freq" || true
    observe_file "policy.cpuinfo_max_freq.$policy_name" "$policy_dir/cpuinfo_max_freq" || true
    observe_file "policy.scaling_min_freq.$policy_name" "$policy_dir/scaling_min_freq" || true
    observe_file "policy.scaling_max_freq.$policy_name" "$policy_dir/scaling_max_freq" || true

    attempt_userspace_roundtrip "$policy_name" "$policy_dir"
}

main() {
    policy_count=0

    emit "env.name=$CPUFREQ_DT_ENV"
    emit "implementation=$CPUFREQ_DT_IMPLEMENTATION"

    load_one_module provider "$CPUFREQ_DT_PROVIDER_MODULE" || return 10
    load_one_module governor "$CPUFREQ_DT_GOV_MODULE" || return 11
    load_one_module driver "$CPUFREQ_DT_DRIVER_MODULE" || return 12

    sleep 2
    save_dmesg_views

    if ! wait_for_policy_root; then
        emit "policy.root_visible=0"
        scan_dmesg_for_anomalies
        return 13
    fi

    emit "policy.root_visible=1"

    for policy_dir in "$CPUFREQ_ROOT"/policy*; do
        [ -d "$policy_dir" ] || continue
        policy_count=$((policy_count + 1))
        observe_policy "$policy_dir"
    done

    emit "policy.count=$policy_count"
    scan_dmesg_for_anomalies

    if [ "$policy_count" -eq 0 ]; then
        return 14
    fi

    return 0
}

main "$@"
