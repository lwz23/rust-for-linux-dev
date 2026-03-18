#!/bin/sh
# SPDX-License-Identifier: GPL-2.0

set -eu

PATH=/usr/sbin:/usr/bin:/bin:/sbin
RESULT_DIR=/tmp/nlmon-results
mkdir -p "$RESULT_DIR" /run/netns

. /etc/nlmon-test.env

IP_BIN=/usr/bin/ip
if [ ! -x "$IP_BIN" ]; then
    IP_BIN=/usr/sbin/ip
fi

ip_cmd() {
    "$IP_BIN" "$@"
}

log() {
    echo "NLMON_TEST: $*"
}

emit() {
    echo "NLMON_RESULT: $*"
}

emit_file_value() {
    local key="$1"
    local file="$2"

    if [ -f "$file" ]; then
        emit "$key=$(tr '\n' ' ' <"$file" | sed 's/[[:space:]]\+/ /g; s/[[:space:]]$//')"
    fi
}

emit_file_excerpt() {
    local key="$1"
    local file="$2"
    local lines="${3:-5}"

    if [ -f "$file" ]; then
        emit "$key=$(head -n "$lines" "$file" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g; s/[[:space:]]$//')"
    fi
}

cleanup_names() {
    ip_cmd link del nlmon0 2>/dev/null || true
    ip_cmd link del nlmon_dummy0 2>/dev/null || true
    ip_cmd link del nlmon_veth0 2>/dev/null || true
    ip_cmd link del nlmon_bridge0 2>/dev/null || true
    ip_cmd link del nlmon_br_veth0 2>/dev/null || true
    ip_cmd link del nlmon_br_veth1 2>/dev/null || true
    ip_cmd link del nlmon_ns_veth0 2>/dev/null || true
    ip_cmd netns del nlmonns0 2>/dev/null || true
    ip_cmd rule del pref 1000 2>/dev/null || true
    ip_cmd route del 198.18.0.0/24 table 100 2>/dev/null || true
}

finish() {
    sync
    cleanup_names
}

trap finish EXIT

load_modules() {
    insmod /lib/modules/dummy.ko
    insmod /lib/modules/llc.ko
    insmod /lib/modules/stp.ko
    insmod /lib/modules/bridge.ko
    insmod /lib/modules/veth.ko
    insmod "$NLMON_MODULE"
}

ensure_nlmon_up() {
    ip_cmd link add nlmon0 type nlmon
    ip_cmd link set nlmon0 up
}

start_capture() {
    local tag="$1"
    rm -f "$RESULT_DIR/$tag.pcap"
    tcpdump -U -i nlmon0 -w "$RESULT_DIR/$tag.pcap" >"$RESULT_DIR/$tag.tcpdump.stdout" 2>"$RESULT_DIR/$tag.tcpdump.stderr" &
    TCPDUMP_PID=$!
    sleep 1
}

stop_capture() {
    if [ -n "${TCPDUMP_PID:-}" ]; then
        sleep 1
        kill -INT "$TCPDUMP_PID" 2>/dev/null || true
        wait "$TCPDUMP_PID" 2>/dev/null || true
        TCPDUMP_PID=
    fi
}

observe_iface() {
    local iface="$1"
    local tag="$2"

    if ip_cmd -d link show "$iface" >"$RESULT_DIR/$tag.ip-d.txt" 2>"$RESULT_DIR/$tag.ip-d.err"; then
        emit "observation_mode.$tag=full-iproute2"
    else
        ip_cmd link show "$iface" >"$RESULT_DIR/$tag.ip.txt" 2>&1 || true
        for field in type flags mtu; do
            cat "/sys/class/net/$iface/$field" >"$RESULT_DIR/$tag.$field.txt" 2>/dev/null || true
        done
        emit "observation_mode.$tag=sysfs-fallback"
    fi

    if ip_cmd -s link show "$iface" >"$RESULT_DIR/$tag.ip-s.txt" 2>"$RESULT_DIR/$tag.ip-s.err"; then
        :
    else
        for field in rx_packets rx_bytes tx_packets tx_bytes; do
            cat "/sys/class/net/$iface/statistics/$field" >"$RESULT_DIR/$tag.$field.txt" 2>/dev/null || true
        done
    fi

    if [ "$NLMON_HAVE_ETHTOOL" = "1" ]; then
        /usr/sbin/ethtool "$iface" >"$RESULT_DIR/$tag.ethtool.txt" 2>&1 || true
    else
        echo "ethtool unavailable on host; not staged into guest" >"$RESULT_DIR/$tag.ethtool.txt"
    fi

    for field in type flags mtu rx_packets rx_bytes tx_packets tx_bytes; do
        emit_file_value "$tag.$field" "$RESULT_DIR/$tag.$field.txt"
    done
}

finalize_capture() {
    local tag="$1"
    local decoded_head="$RESULT_DIR/$tag.decoded.head"
    local decoded_lines="$RESULT_DIR/$tag.decoded.lines"

    : >"$decoded_head"
    if ! tcpdump -nn -r "$RESULT_DIR/$tag.pcap" 2>"$RESULT_DIR/$tag.decode.err" \
        | awk -v head_file="$decoded_head" '
            NR <= 20 { print > head_file }
            { count++ }
            END { print count + 0 }
        ' >"$decoded_lines"; then
        echo 0 >"$decoded_lines"
    fi
    sha256sum "$RESULT_DIR/$tag.pcap" >"$RESULT_DIR/$tag.pcap.sha256" 2>/dev/null || true
    wc -c "$RESULT_DIR/$tag.pcap" >"$RESULT_DIR/$tag.pcap.bytes" 2>/dev/null || true
    wc -l "$RESULT_DIR/$tag.tcpdump.stdout" >"$RESULT_DIR/$tag.tcpdump.stdout.lines" 2>/dev/null || true
    wc -l "$RESULT_DIR/$tag.tcpdump.stderr" >"$RESULT_DIR/$tag.tcpdump.stderr.lines" 2>/dev/null || true
    wc -l "$RESULT_DIR/$tag.decode.err" >"$RESULT_DIR/$tag.decode.err.lines" 2>/dev/null || true
    emit_file_value "$tag.pcap.sha256" "$RESULT_DIR/$tag.pcap.sha256"
    emit_file_value "$tag.pcap.bytes" "$RESULT_DIR/$tag.pcap.bytes"
    emit_file_value "$tag.decoded.lines" "$RESULT_DIR/$tag.decoded.lines"
    emit_file_value "$tag.tcpdump.stdout.lines" "$RESULT_DIR/$tag.tcpdump.stdout.lines"
    emit_file_value "$tag.tcpdump.stderr.lines" "$RESULT_DIR/$tag.tcpdump.stderr.lines"
    emit_file_value "$tag.decode.err.lines" "$RESULT_DIR/$tag.decode.err.lines"
    emit_file_excerpt "$tag.decoded.head" "$decoded_head"
    emit_file_excerpt "$tag.tcpdump.stderr.head" "$RESULT_DIR/$tag.tcpdump.stderr"
    emit_file_excerpt "$tag.decode.err.head" "$RESULT_DIR/$tag.decode.err"
    observe_iface nlmon0 "$tag.nlmon0"
}

scan_dmesg() {
    local tag="$1"
    local anomaly_pattern

    anomaly_pattern="BUG:|WARNING:|Oops:|KASAN:|KFENCE:|KCSAN:|UBSAN:|use-after-free|double[- ]free|lockdep:|possible recursive locking detected|suspicious RCU usage|refcount_t:|kmemleak|DEBUG_OBJECTS|bad unlock balance"
    dmesg >"$RESULT_DIR/$tag.dmesg.txt" 2>&1 || true
    if grep -E "$anomaly_pattern" "$RESULT_DIR/$tag.dmesg.txt" >"$RESULT_DIR/$tag.dmesg.anomalies.txt"; then
        emit "dmesg_anomaly.$tag=1"
    else
        emit "dmesg_anomaly.$tag=0"
    fi
    wc -l "$RESULT_DIR/$tag.dmesg.anomalies.txt" >"$RESULT_DIR/$tag.dmesg.anomaly.lines" 2>/dev/null || true
    emit_file_value "dmesg_anomaly_lines.$tag" "$RESULT_DIR/$tag.dmesg.anomaly.lines"
    emit_file_excerpt "dmesg_anomaly_head.$tag" "$RESULT_DIR/$tag.dmesg.anomalies.txt"
}

gen_dummy() {
    log "generator: dummy"
    ip_cmd link add nlmon_dummy0 type dummy
    ip_cmd link set nlmon_dummy0 up
    ip_cmd addr add 192.0.2.1/24 dev nlmon_dummy0
    ip_cmd -s link show nlmon_dummy0 >/dev/null 2>&1 || true
    observe_iface nlmon_dummy0 dummy
    ip_cmd link del nlmon_dummy0 || true
}

gen_veth() {
    log "generator: veth"
    ip_cmd link add nlmon_veth0 type veth peer name nlmon_veth1
    ip_cmd link set nlmon_veth0 up || true
    ip_cmd link set nlmon_veth1 up || true
    ip_cmd addr add 198.51.100.1/24 dev nlmon_veth0 || true
    ip_cmd addr add 198.51.100.2/24 dev nlmon_veth1 || true
    ip_cmd -s link show nlmon_veth0 >/dev/null 2>&1 || true
    ip_cmd link del nlmon_veth0 || true
}

gen_bridge() {
    log "generator: bridge"
    ip_cmd link add nlmon_bridge0 type bridge
    ip_cmd link add nlmon_br_veth0 type veth peer name nlmon_br_veth1
    ip_cmd link set nlmon_bridge0 up || true
    ip_cmd link set nlmon_br_veth0 master nlmon_bridge0 || true
    ip_cmd link set nlmon_br_veth0 up || true
    ip_cmd link set nlmon_br_veth1 up || true
    ip_cmd link del nlmon_br_veth0 || true
    ip_cmd link del nlmon_bridge0 || true
}

gen_route_rule() {
    log "generator: route_rule"
    ip_cmd link set lo up || true
    ip_cmd route add 198.18.0.0/24 dev lo table 100 || true
    ip_cmd rule add pref 1000 from 192.0.2.0/24 table 100 || true
    ip_cmd rule del pref 1000 || true
    ip_cmd route del 198.18.0.0/24 table 100 || true
}

gen_netns() {
    log "generator: netns"
    ip_cmd netns add nlmonns0
    ip_cmd link add nlmon_ns_veth0 type veth peer name nlmon_ns_veth1
    ip_cmd link set nlmon_ns_veth1 netns nlmonns0
    ip_cmd link set nlmon_ns_veth0 up || true
    ip_cmd netns exec nlmonns0 "$IP_BIN" link set lo up || true
    ip_cmd netns exec nlmonns0 "$IP_BIN" link set nlmon_ns_veth1 up || true
    ip_cmd netns del nlmonns0 || true
    ip_cmd link del nlmon_ns_veth0 || true
}

run_all_generators_once() {
    gen_dummy
    gen_veth
    gen_bridge
    gen_route_rule
    gen_netns
}

run_baseline() {
    log "running baseline scenario"
    ensure_nlmon_up
    start_capture baseline
    run_all_generators_once
    stop_capture
    finalize_capture baseline
    scan_dmesg baseline
    ip_cmd link del nlmon0
}

run_lifecycle() {
    log "running lifecycle scenario"
    i=1
    while [ "$i" -le "$NLMON_LIFECYCLE_LOOPS" ]; do
        ensure_nlmon_up
        if [ $((i % 10)) -eq 0 ]; then
            run_all_generators_once
        fi
        ip_cmd link del nlmon0
        i=$((i + 1))
    done
    scan_dmesg lifecycle
}

run_matrix() {
    log "running generator matrix"
    for generator in dummy veth bridge route_rule netns; do
        ensure_nlmon_up
        start_capture "matrix.$generator"
        i=1
        while [ "$i" -le "$NLMON_GENERATOR_ROUNDS" ]; do
            "gen_$generator"
            i=$((i + 1))
        done
        stop_capture
        finalize_capture "matrix.$generator"
        scan_dmesg "matrix.$generator"
        ip_cmd link del nlmon0
    done
}

run_stress() {
    log "running stress scenario"
    ensure_nlmon_up
    start_capture stress
    start_ts="$(date +%s)"
    while :; do
        now="$(date +%s)"
        if [ $((now - start_ts)) -ge "$NLMON_STRESS_SECONDS" ]; then
            break
        fi
        run_all_generators_once
    done
    stop_capture
    finalize_capture stress
    scan_dmesg stress
    ip_cmd link del nlmon0
}

emit "implementation=$NLMON_IMPLEMENTATION"
emit "scenario=$NLMON_SCENARIO"
emit "have_ethtool=$NLMON_HAVE_ETHTOOL"
emit "ip_binary=$IP_BIN"
emit "ip_version=$("$IP_BIN" -V 2>&1 | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g; s/[[:space:]]$//')"

load_modules

case "$NLMON_SCENARIO" in
    baseline) run_baseline ;;
    lifecycle) run_lifecycle ;;
    matrix) run_matrix ;;
    stress) run_stress ;;
    all)
        run_baseline
        run_lifecycle
        run_matrix
        run_stress
        ;;
    *)
        log "unknown scenario: $NLMON_SCENARIO"
        exit 1
        ;;
esac

emit "status=ok"
