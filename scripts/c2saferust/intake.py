#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0

from __future__ import annotations

import json
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INCLUDE_RE = re.compile(r'^\s*#include\s+[<"]([^>"]+)[>"]', re.MULTILINE)
MAKEFILE_ENTRY_RE = re.compile(r'obj-\$\(CONFIG_([A-Z0-9_]+)\)\s*\+=\s*([A-Za-z0-9_.-]+)')
INITIALIZER_FIELD_RE = re.compile(r"^\s*\.(\w+)\s*=\s*([^,]+),", re.MULTILINE)
VALIDATE_CHECK_RE = re.compile(r"if\s*\(\s*tb\[(?P<field>[A-Z0-9_]+)\]\s*\)\s*return\s+(?P<retval>[-A-Z0-9_]+);")

HELPER_WRAPPER_CANDIDATES = {
    "dev_kfree_skb": {
        "helper_name": "rust_helper_dev_kfree_skb",
        "reason": "skb release helper is exposed as a macro and should not leak macro expansion into Rust",
        "required_headers": ["linux/skbuff.h"],
        "signature": "void rust_helper_dev_kfree_skb(struct sk_buff *skb)",
        "body": ["\tdev_kfree_skb(skb);"],
        "mvp_blocking": False,
    },
    "dev_lstats_add": {
        "helper_name": "rust_helper_dev_lstats_add",
        "reason": "per-cpu stats helper; bindgen cannot cross the inline boundary directly",
        "required_headers": ["linux/netdevice.h"],
        "signature": "void rust_helper_dev_lstats_add(struct net_device *dev, unsigned int len)",
        "body": ["\tdev_lstats_add(dev, len);"],
        "mvp_blocking": False,
    },
    "netdev_priv": {
        "helper_name": "rust_helper_netdev_priv",
        "reason": "private data access helper; Rust side should not depend on C macro expansion",
        "required_headers": ["linux/netdevice.h"],
        "signature": "void *rust_helper_netdev_priv(const struct net_device *dev)",
        "body": ["\treturn netdev_priv(dev);"],
        "mvp_blocking": True,
    },
}

DIRECT_FFI_CANDIDATES = {
    "dev_lstats_read": "regular function used by stats callback",
    "netlink_add_tap": "regular exported netlink tap registration function",
    "netlink_remove_tap": "regular exported netlink tap unregistration function",
    "rtnl_link_register": "regular exported rtnl link registration function",
    "rtnl_link_unregister": "regular exported rtnl link unregistration function",
}

ABSTRACTION_REQUIREMENTS = {
    "rtnl": {
        "patterns": ("struct rtnl_link_ops", "rtnl_link_register", "rtnl_link_unregister"),
        "reason": "nlmon is an rtnl link type and needs Rust-side registration and callback bridging",
    },
    "netdevice": {
        "patterns": ("struct net_device_ops", "struct net_device", "struct ethtool_ops"),
        "reason": "nlmon configures net_device state, netdev ops, ethtool ops, flags, and MTU",
    },
    "netlink_tap": {
        "patterns": ("struct netlink_tap", "netlink_add_tap", "netlink_remove_tap"),
        "reason": "nlmon open/close lifecycle is defined around netlink tap registration",
    },
    "stats": {
        "patterns": ("dev_lstats_add", "dev_lstats_read", "rtnl_link_stats64"),
        "reason": "nlmon uses lstats helpers and get_stats64 callback wiring",
    },
    "skbuff": {
        "patterns": ("struct sk_buff", "dev_kfree_skb"),
        "reason": "nlmon xmit path consumes skb objects and therefore needs a Rust skb wrapper story",
    },
}

BINDINGS_HEADER_DECISIONS = {
    "linux/if_arp.h": {
        "action": "add_to_bindings",
        "reason": "Expose `ARPHRD_NETLINK` for `nlmon_setup` device type configuration.",
        "required_symbols": ["ARPHRD_NETLINK"],
        "mvp_blocking": True,
    },
    "linux/kernel.h": {
        "action": "handled_by_rust_abstraction",
        "reason": "The Rust side should avoid depending on generic C helper macros from `linux/kernel.h`.",
        "required_symbols": [],
        "mvp_blocking": False,
    },
    "linux/module.h": {
        "action": "handled_by_rust_abstraction",
        "reason": "Use `kernel::ThisModule`/`module!` instead of bindgen exposure for `THIS_MODULE` and module lifecycle macros.",
        "required_symbols": ["THIS_MODULE"],
        "mvp_blocking": False,
    },
    "linux/netdevice.h": {
        "action": "add_to_bindings",
        "reason": "Expose `net_device`, `net_device_ops`, `dev_lstats_read`, flags, and per-cpu stat enums needed by nlmon planning.",
        "required_symbols": [
            "struct net_device",
            "struct net_device_ops",
            "struct rtnl_link_stats64",
            "dev_lstats_read",
            "IFF_NO_QUEUE",
            "NETDEV_PCPU_STAT_LSTATS",
        ],
        "mvp_blocking": True,
    },
    "linux/netlink.h": {
        "action": "add_to_bindings",
        "reason": "Expose `netlink_tap`, tap registration functions, and `NLMSG_GOODSIZE` used by nlmon lifecycle/setup planning.",
        "required_symbols": [
            "struct netlink_tap",
            "netlink_add_tap",
            "netlink_remove_tap",
            "NLMSG_GOODSIZE",
        ],
        "mvp_blocking": True,
    },
    "net/net_namespace.h": {
        "action": "defer_to_abstraction",
        "reason": "Namespace-specific link handling is outside the first `ip link add nlmon0 type nlmon` MVP.",
        "required_symbols": ["struct net"],
        "mvp_blocking": False,
    },
    "net/rtnetlink.h": {
        "action": "add_to_bindings",
        "reason": "Expose `rtnl_link_ops`, `rtnl_link_register`, `rtnl_link_unregister`, and `netlink_ext_ack` for the link-type abstraction plan.",
        "required_symbols": [
            "struct rtnl_link_ops",
            "rtnl_link_register",
            "rtnl_link_unregister",
            "struct netlink_ext_ack",
        ],
        "mvp_blocking": True,
    },
}

KEYWORD_CALLS = {"if", "return", "sizeof", "while", "for", "switch"}

MVP_NET_MODULES = {
    "rtnl": "rust/kernel/net/rtnl.rs",
    "netdevice": "rust/kernel/net/netdevice.rs",
    "netlink_tap": "rust/kernel/net/netlink_tap.rs",
    "stats": "rust/kernel/net/stats.rs",
    "skbuff": "rust/kernel/net/skbuff.rs",
}

POST_MVP_NET_MODULES = {}


def _repo_root(repo_root: str | Path | None) -> Path:
    return Path(repo_root).resolve() if repo_root else REPO_ROOT


def _path_from_repo(repo_root: Path, path_like: str | Path) -> Path:
    path = Path(path_like)
    return path if path.is_absolute() else (repo_root / path)


def _rel(repo_root: Path, path: Path) -> str:
    try:
        return str(path.resolve().relative_to(repo_root.resolve()))
    except ValueError:
        return str(path.resolve())


def stable_json(payload: dict) -> str:
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def write_json(path: str | Path, payload: dict) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(stable_json(payload))


def _load_text(path: Path) -> str:
    return path.read_text()


def _normalize_whitespace(text: str) -> str:
    return " ".join(text.split())


def _compact_whitespace(text: str) -> str:
    return "".join(text.split())


def _strip_rust_noncode(text: str) -> str:
    result: list[str] = []
    index = 0
    block_comment_depth = 0
    in_line_comment = False
    in_string = False
    in_char = False
    escape = False

    while index < len(text):
        char = text[index]
        nxt = text[index + 1] if index + 1 < len(text) else ""

        if in_line_comment:
            if char == "\n":
                in_line_comment = False
                result.append(char)
            else:
                result.append(" ")
            index += 1
            continue

        if block_comment_depth:
            if char == "/" and nxt == "*":
                block_comment_depth += 1
                result.extend("  ")
                index += 2
            elif char == "*" and nxt == "/":
                block_comment_depth -= 1
                result.extend("  ")
                index += 2
            else:
                result.append("\n" if char == "\n" else " ")
                index += 1
            continue

        if in_string:
            if escape:
                escape = False
                result.append(" ")
            elif char == "\\":
                escape = True
                result.append(" ")
            elif char == '"':
                in_string = False
                result.append(" ")
            else:
                result.append("\n" if char == "\n" else " ")
            index += 1
            continue

        if in_char:
            if escape:
                escape = False
                result.append(" ")
            elif char == "\\":
                escape = True
                result.append(" ")
            elif char == "'":
                in_char = False
                result.append(" ")
            else:
                result.append("\n" if char == "\n" else " ")
            index += 1
            continue

        if char == "/" and nxt == "/":
            in_line_comment = True
            result.extend("  ")
            index += 2
            continue

        if char == "/" and nxt == "*":
            block_comment_depth = 1
            result.extend("  ")
            index += 2
            continue

        if char == '"':
            in_string = True
            result.append(" ")
            index += 1
            continue

        if char == "'":
            in_char = True
            result.append(" ")
            index += 1
            continue

        result.append(char)
        index += 1

    return "".join(result)


def _find_rust_unsafe_sites(repo_root: Path, relative_paths: list[str]) -> list[dict]:
    sites: list[dict] = []
    for relative_path in relative_paths:
        path = _path_from_repo(repo_root, relative_path)
        if not path.exists():
            continue
        stripped = _strip_rust_noncode(_load_text(path))
        for line_number, line in enumerate(stripped.splitlines(), start=1):
            if "unsafe" not in line:
                continue
            if re.search(r"\bunsafe\s+impl\b", line):
                kind = "unsafe-impl"
            elif re.search(r"\bunsafe\s+fn\b", line):
                kind = "unsafe-fn"
            elif re.search(r"\bunsafe\s+trait\b", line):
                kind = "unsafe-trait"
            elif re.search(r"\bunsafe\s*\{", line):
                kind = "unsafe-block"
            else:
                kind = "unsafe-token"

            sites.append(
                {
                    "file": relative_path,
                    "line": line_number,
                    "unsafe_kind": kind,
                    "source_excerpt": line.strip(),
                }
            )
    return sites


def _guess_obligation_ids(relative_path: str, excerpt: str) -> list[str]:
    obligation_ids: list[str] = []

    if relative_path.endswith("rust/kernel/net/rtnl.rs"):
        if any(token in excerpt for token in ("rtnl_link_register", "rtnl_link_unregister", "rtnl_link_ops")):
            obligation_ids.append("rtnl-registration-lifetime")
        if any(token in excerpt for token in ("Device::from_raw", "ExtAck::from_raw", "unsafe fn from_raw")):
            obligation_ids.append("callback-pointer-aliasing")
    elif relative_path.endswith("rust/kernel/net/netdevice.rs"):
        if any(token in excerpt for token in ("private_ptr", "netdev_priv")):
            obligation_ids.append("netdev-private-layout")
        if any(token in excerpt for token in ("from_raw", "private_ptr", "Pin::new_unchecked")):
            obligation_ids.append("callback-pointer-aliasing")
        if any(token in excerpt for token in ("from_raw_ref", "SkBuff::from_raw_owned", "TxOutcome::Busy")):
            obligation_ids.append("skb-consumed-once")
    elif relative_path.endswith("rust/kernel/net/netlink_tap.rs"):
        if any(token in excerpt for token in ("netlink_add_tap", "module", ".dev")):
            obligation_ids.append("tap-module-pointer-validity")
        if any(token in excerpt for token in ("netlink_remove_tap", "registered", "zeroed")):
            obligation_ids.append("tap-open-stop-balance")
    elif relative_path.endswith("rust/kernel/net/stats.rs"):
        if any(token in excerpt for token in ("rust_helper_dev_lstats_add", "dev_lstats_add")):
            obligation_ids.append("callback-pointer-aliasing")
    elif relative_path.endswith("rust/kernel/net/skbuff.rs"):
        if any(
            token in excerpt
            for token in (
                "from_raw_owned",
                "into_raw",
                "dev_kfree_skb",
                "rust_helper_dev_kfree_skb",
                "NonNull::new_unchecked",
                "self.ptr()).len",
            )
        ):
            obligation_ids.append("skb-consumed-once")

    return obligation_ids


def _extract_includes(source_text: str) -> list[str]:
    seen = []
    for header in INCLUDE_RE.findall(source_text):
        if header not in seen:
            seen.append(header)
    return seen


def _collect_rust_net_modules(repo_root: Path) -> list[str]:
    net_dir = repo_root / "rust" / "kernel" / "net"
    if not net_dir.exists():
        return []
    return sorted(str(path.relative_to(repo_root)) for path in net_dir.rglob("*.rs"))


def _artifact_dir_for(module_id: str) -> str:
    return f"Documentation/rust/c2saferust/{module_id}"


def _artifact_path_for(module_id: str, filename: str) -> str:
    return f"{_artifact_dir_for(module_id)}/{filename}"


def _line_number_of_exact_line(text: str, exact_line: str | None) -> int | None:
    if exact_line is None:
        return None
    for line_number, line in enumerate(text.splitlines(), start=1):
        if line == exact_line:
            return line_number
    return None


def _find_kconfig_block_end_line(text: str, symbol: str) -> int | None:
    lines = text.splitlines()
    start_line = _line_number_of_exact_line(text, f"config {symbol}")
    if start_line is None:
        return None

    end_line = start_line
    for line_number in range(start_line + 1, len(lines) + 1):
        line = lines[line_number - 1]
        if line and not line.startswith((" ", "\t")) and re.match(
            r"(config|menuconfig|source|if|endif|comment)\b", line
        ):
            break
        end_line = line_number
    return end_line


def _extract_brace_body(text: str, open_brace_index: int) -> str:
    depth = 0
    for index in range(open_brace_index, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[open_brace_index + 1 : index]
    raise ValueError("Unbalanced braces while extracting C block")


def _extract_function_body(source_text: str, function_name: str) -> str | None:
    match = re.search(
        rf"\b{re.escape(function_name)}\s*\([^;]*?\)\s*\{{",
        source_text,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        return None
    return _extract_brace_body(source_text, match.end() - 1)


def _find_initializer_name(source_text: str, struct_name: str) -> str | None:
    match = re.search(
        rf"static\s+(?:const\s+)?struct\s+{re.escape(struct_name)}\s+([A-Za-z0-9_]+)[^=]*=\s*\{{",
        source_text,
        re.MULTILINE,
    )
    return match.group(1) if match else None


def _extract_initializer_body(source_text: str, initializer_name: str) -> str | None:
    match = re.search(
        rf"\b{re.escape(initializer_name)}\b[^=]*=\s*\{{",
        source_text,
        re.MULTILINE,
    )
    if not match:
        return None
    return _extract_brace_body(source_text, match.end() - 1)


def _extract_initializer_fields(initializer_body: str | None) -> list[dict]:
    if not initializer_body:
        return []
    return [
        {
            "field": match.group(1),
            "value": match.group(2).strip(),
        }
        for match in INITIALIZER_FIELD_RE.finditer(initializer_body)
    ]


def _initializer_field_map(initializer_body: str | None) -> dict[str, str]:
    return {
        entry["field"]: entry["value"]
        for entry in _extract_initializer_fields(initializer_body)
    }


def _extract_pointer_field_assignments(body: str | None, prefix: str) -> list[dict]:
    if not body:
        return []
    return [
        {
            "field": match.group(1),
            "operator": match.group(2),
            "value": match.group(3).strip(),
        }
        for match in re.finditer(
            rf"{re.escape(prefix)}([A-Za-z0-9_]+)\s*(\|=|=)\s*([^;]+);",
            body,
        )
    ]


def _extract_call_sites(body: str | None) -> list[str]:
    if not body:
        return []
    seen: list[str] = []
    for match in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(", body):
        candidate = match.group(1)
        if candidate in KEYWORD_CALLS or candidate == "this_cpu_ptr":
            continue
        if candidate not in seen:
            seen.append(candidate)
    return seen


def _extract_validate_checks(body: str | None) -> list[dict]:
    if not body:
        return []
    return [
        {
            "field": match.group("field"),
            "return": match.group("retval"),
        }
        for match in VALIDATE_CHECK_RE.finditer(body)
    ]


def _extract_private_struct_fields(source_text: str, struct_name: str) -> list[dict]:
    match = re.search(
        rf"struct\s+{re.escape(struct_name)}\s*\{{(?P<body>.*?)\}};",
        source_text,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        return []

    fields = []
    for line in match.group("body").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("/*"):
            continue
        field_match = re.match(r"(.+?)\s+([A-Za-z_][A-Za-z0-9_]*)\s*;$", stripped)
        if field_match:
            fields.append(
                {
                    "type": field_match.group(1).strip(),
                    "name": field_match.group(2),
                }
            )
    return fields


def _helper_symbols_used(source_text: str) -> list[str]:
    return [
        symbol
        for symbol in HELPER_WRAPPER_CANDIDATES
        if re.search(rf"\b{re.escape(symbol)}\s*\(", source_text)
    ]


def _binding_header_plan(header: str) -> dict:
    return BINDINGS_HEADER_DECISIONS.get(
        header,
        {
            "action": "add_to_bindings",
            "reason": "Direct source include is missing from `bindings_helper.h`; no explicit override rule exists.",
            "required_symbols": [],
            "mvp_blocking": False,
        },
    )


def _build_binding_patch_units(bindings_helper_text: str, headers: list[dict], target_path: str) -> list[dict]:
    existing_headers = _extract_includes(bindings_helper_text)
    searchable_headers = [
        header
        for header in existing_headers
        if not header.startswith("../../") and not header.startswith("trace/")
    ]
    grouped: dict[tuple[str | None, str | None], list[dict]] = {}

    for header_entry in headers:
        header = header_entry["header"]
        smaller = [candidate for candidate in searchable_headers if candidate < header]
        larger = [candidate for candidate in searchable_headers if candidate > header]
        insert_after = max(smaller) if smaller else None
        insert_before = min(larger) if larger else None
        grouped.setdefault((insert_after, insert_before), []).append(header_entry)

    patch_units = []
    for (insert_after, insert_before), block_entries in sorted(
        grouped.items(), key=lambda item: [(entry["header"]) for entry in item[1]]
    ):
        insert_lines = [f"#include <{entry['header']}>" for entry in sorted(block_entries, key=lambda item: item["header"])]
        anchor_line = (
            _line_number_of_exact_line(bindings_helper_text, f"#include <{insert_after}>")
            if insert_after
            else _line_number_of_exact_line(bindings_helper_text, f"#include <{insert_before}>")
        )
        patch_units.append(
            {
                "target_path": target_path,
                "operation": "insert_after" if insert_after else "insert_before",
                "anchor": {
                    "header": insert_after or insert_before,
                    "line": anchor_line,
                },
                "insert_lines": insert_lines,
                "headers": [
                    {
                        "header": entry["header"],
                        "reason": entry["reason"],
                        "required_symbols": entry["required_symbols"],
                        "mvp_blocking": entry["mvp_blocking"],
                    }
                    for entry in sorted(block_entries, key=lambda item: item["header"])
                ],
            }
        )
    return patch_units


def _helper_wrapper_specs(symbols: list[str]) -> list[dict]:
    return [
        {
            "symbol": symbol,
            **HELPER_WRAPPER_CANDIDATES[symbol],
        }
        for symbol in sorted(symbols)
    ]


def _build_helper_file_content(wrapper_specs: list[dict]) -> list[str]:
    include_headers = sorted({header for spec in wrapper_specs for header in spec["required_headers"]})
    content = ["// SPDX-License-Identifier: GPL-2.0", ""]
    content.extend(f"#include <{header}>" for header in include_headers)
    content.append("")
    for index, spec in enumerate(wrapper_specs):
        content.append(f"__rust_helper {spec['signature']}")
        content.append("{")
        content.extend(spec["body"])
        content.append("}")
        if index != len(wrapper_specs) - 1:
            content.append("")
    return content


def _helper_include_anchor(helpers_aggregate_text: str, helper_file_name: str) -> tuple[str | None, str | None]:
    include_names = re.findall(r'^#include "([^"]+)"', helpers_aggregate_text, re.MULTILINE)
    previous_candidates = [name for name in include_names if name < helper_file_name]
    next_candidates = [name for name in include_names if name > helper_file_name]
    return (max(previous_candidates) if previous_candidates else None, min(next_candidates) if next_candidates else None)


def _file_exists(repo_root: Path, relpath: str) -> bool:
    return (repo_root / relpath).exists()


def _read_if_exists(path: Path) -> str:
    return path.read_text() if path.exists() else ""


def _implemented_net_areas(repo_root: Path) -> set[str]:
    net_root = _read_if_exists(repo_root / "rust" / "kernel" / "net.rs")
    implemented = set()

    for area, relpath in {**MVP_NET_MODULES, **POST_MVP_NET_MODULES}.items():
        if _file_exists(repo_root, relpath) and f"pub mod {area};" in net_root:
            implemented.add(area)

    return implemented


def _net_scope_assessment(modules: list[str], implemented_areas: set[str]) -> str:
    implemented_mvp = [area for area in MVP_NET_MODULES if area in implemented_areas]
    if set(MVP_NET_MODULES).issubset(implemented_areas):
        return (
            "The tree now contains the full smoke-path link-type abstractions "
            "(`rtnl`, `netdevice`, `netlink_tap`, `stats`, `skbuff`); "
            "`get_stats64` and `ethtool` remain deferred."
        )
    if modules == ["rust/kernel/net/phy.rs", "rust/kernel/net/phy/reg.rs"]:
        return "The tree currently exposes only `net::phy`; nlmon needs fresh abstractions for link-type devices."
    return (
        "The tree exposes partial Rust net support; nlmon still needs the remaining MVP link-type "
        f"abstractions after {', '.join(implemented_mvp) or 'phy'}."
    )


def build_kbuild_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    module_dir = module.parent
    makefile = module_dir / "Makefile"
    kconfig = module_dir / "Kconfig"

    module_object = module.with_suffix(".o").name
    makefile_text = _load_text(makefile)
    kconfig_text = _load_text(kconfig)

    source_config_symbol = module.stem.upper().replace("-", "_")
    current_makefile_line = None
    for match in MAKEFILE_ENTRY_RE.finditer(makefile_text):
        if match.group(2) == module_object:
            source_config_symbol = match.group(1)
            current_makefile_line = match.group(0)
            break

    rust_symbol = f"{source_config_symbol}_RUST"
    has_rust_switch = (
        f"CONFIG_{rust_symbol}" in makefile_text
        or re.search(rf"^config {rust_symbol}$", kconfig_text, re.MULTILINE) is not None
    )

    suggested_kconfig = [
        f"config {rust_symbol}",
        f'\tbool "Rust implementation of {module.stem}"',
        f"\tdepends on RUST && {source_config_symbol}",
        "\thelp",
        f"\t  Builds the Rust implementation of {module.stem} ({module.stem}_rust.ko)",
        f"\t  instead of the original C implementation ({module.stem}.ko).",
    ]
    suggested_makefile = [
        f"ifdef CONFIG_{rust_symbol}",
        f"  obj-$(CONFIG_{source_config_symbol}) += {module.stem}_rust.o",
        "else",
        f"  obj-$(CONFIG_{source_config_symbol}) += {module.stem}.o",
        "endif",
    ]
    next_action = (
        "Kbuild switch already exists; keep downstream patch/translation artifacts aligned."
        if has_rust_switch
        else "Apply the Kbuild switch before generating nlmon Rust driver patches."
    )

    return {
        "schema_version": 1,
        "artifact_type": "kbuild-plan",
        "module_id": module.stem,
        "module_c_path": _rel(repo, module),
        "module_dir": _rel(repo, module_dir),
        "kconfig_path": _rel(repo, kconfig),
        "makefile_path": _rel(repo, makefile),
        "source_config_symbol": source_config_symbol,
        "suggested_rust_config_symbol": rust_symbol,
        "suggested_rust_object": f"{module.stem}_rust.o",
        "current_state": {
            "current_makefile_line": current_makefile_line,
            "current_kconfig_present": re.search(
                rf"^config {source_config_symbol}$", kconfig_text, re.MULTILINE
            )
            is not None,
            "current_rust_switch_present": has_rust_switch,
        },
        "reference_pattern": {
            "kconfig_path": "drivers/net/phy/Kconfig",
            "makefile_path": "drivers/net/phy/Makefile",
            "rust_driver_path": "drivers/net/phy/ax88796b_rust.rs",
        },
        "suggested_kconfig_snippet": suggested_kconfig,
        "suggested_makefile_snippet": suggested_makefile,
        "next_action": next_action,
    }


def build_binding_gap_audit(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    source_text = _load_text(module)
    bindings_helper = repo / "rust" / "bindings" / "bindings_helper.h"
    bindings_helper_text = _load_text(bindings_helper)

    includes = _extract_includes(source_text)
    helper_includes = set(_extract_includes(bindings_helper_text))
    missing_headers = [header for header in includes if header not in helper_includes]
    existing_headers = [header for header in includes if header in helper_includes]

    direct_ffi_symbols = [
        symbol for symbol in DIRECT_FFI_CANDIDATES if re.search(rf"\b{symbol}\s*\(", source_text)
    ]
    helper_gap_symbols = _helper_symbols_used(source_text)

    abstraction_gaps = []
    for area, metadata in ABSTRACTION_REQUIREMENTS.items():
        if any(pattern in source_text for pattern in metadata["patterns"]):
            abstraction_gaps.append(
                {
                    "area": area,
                    "reason": metadata["reason"],
                }
            )

    return {
        "schema_version": 1,
        "artifact_type": "binding-gap-audit",
        "module_id": module.stem,
        "module_c_path": _rel(repo, module),
        "bindings_helper_path": _rel(repo, bindings_helper),
        "included_headers": includes,
        "headers_already_covered": existing_headers,
        "candidate_missing_binding_headers": missing_headers,
        "direct_ffi_symbols": [
            {
                "symbol": symbol,
                "reason": DIRECT_FFI_CANDIDATES[symbol],
            }
            for symbol in direct_ffi_symbols
        ],
        "helper_gap_symbols": [
            {
                "symbol": symbol,
                "reason": HELPER_WRAPPER_CANDIDATES[symbol]["reason"],
            }
            for symbol in helper_gap_symbols
        ],
        "required_rust_abstractions": abstraction_gaps,
        "current_rust_net_modules": _collect_rust_net_modules(repo),
        "tool_assessment": {
            "requires_new_rust_net_abstractions": bool(abstraction_gaps),
            "stage_2_can_be_driven_statically": True,
        },
    }


def build_helper_audit(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    source_text = _load_text(module)
    helper_dir = repo / "rust" / "helpers"
    helper_files = sorted(path.name for path in helper_dir.glob("*.c"))
    helper_text = "\n".join((helper_dir / name).read_text() for name in helper_files)

    helper_gap_symbols = _helper_symbols_used(source_text)
    existing_net_helpers = [
        symbol
        for symbol in helper_gap_symbols
        if re.search(rf"\b{symbol}\b", helper_text) or re.search(rf"\brust_helper_{symbol}\b", helper_text)
    ]
    missing_helper_wrappers = [symbol for symbol in helper_gap_symbols if symbol not in existing_net_helpers]

    return {
        "schema_version": 1,
        "artifact_type": "helper-audit",
        "module_id": module.stem,
        "module_c_path": _rel(repo, module),
        "helper_dir": _rel(repo, helper_dir),
        "existing_helper_files": helper_files,
        "required_helper_wrappers": [
            {
                "symbol": symbol,
                "reason": HELPER_WRAPPER_CANDIDATES[symbol]["reason"],
            }
            for symbol in helper_gap_symbols
        ],
        "helpers_already_present": existing_net_helpers,
        "missing_helper_wrappers": missing_helper_wrappers,
    }


def build_kbuild_patch_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    kbuild_plan = build_kbuild_plan(module_path, repo_root=repo)
    makefile = _path_from_repo(repo, kbuild_plan["makefile_path"])
    kconfig = _path_from_repo(repo, kbuild_plan["kconfig_path"])
    makefile_text = _load_text(makefile)
    kconfig_text = _load_text(kconfig)

    current_makefile_line = kbuild_plan["current_state"]["current_makefile_line"]
    makefile_line = _line_number_of_exact_line(makefile_text, current_makefile_line)
    kconfig_insert_after_line = _find_kconfig_block_end_line(
        kconfig_text, kbuild_plan["source_config_symbol"]
    )
    patch_required = not kbuild_plan["current_state"]["current_rust_switch_present"]
    patch_units = []
    if patch_required:
        patch_units = [
            {
                "target_path": kbuild_plan["kconfig_path"],
                "operation": "insert_after",
                "anchor": {
                    "symbol": kbuild_plan["source_config_symbol"],
                    "line": kconfig_insert_after_line,
                },
                "insert_lines": kbuild_plan["suggested_kconfig_snippet"],
                "reason": "Introduce a Rust switch without replacing the existing `NLMON` user-facing tristate.",
            },
            {
                "target_path": kbuild_plan["makefile_path"],
                "operation": "replace_exact_line",
                "anchor": {
                    "line": makefile_line,
                    "text": current_makefile_line,
                },
                "replacement_lines": kbuild_plan["suggested_makefile_snippet"],
                "reason": "Select `nlmon_rust.o` only when `CONFIG_NLMON_RUST=y`; otherwise keep the C object.",
            },
        ]

    return {
        "schema_version": 1,
        "artifact_type": "kbuild-patch-plan",
        "module_id": kbuild_plan["module_id"],
        "module_c_path": kbuild_plan["module_c_path"],
        "patch_required": patch_required,
        "status": "pending" if patch_required else "already_applied",
        "inputs": {
            "kbuild_plan": _artifact_path_for(kbuild_plan["module_id"], "kbuild-plan.json"),
        },
        "patch_units": patch_units,
        "validation_checks": [
            {
                "path": kbuild_plan["kconfig_path"],
                "contains": f"config {kbuild_plan['suggested_rust_config_symbol']}",
            },
            {
                "path": kbuild_plan["makefile_path"],
                "contains": f"ifdef CONFIG_{kbuild_plan['suggested_rust_config_symbol']}",
            },
            {
                "path": kbuild_plan["makefile_path"],
                "contains": f"obj-$(CONFIG_{kbuild_plan['source_config_symbol']}) += {kbuild_plan['suggested_rust_object']}",
            },
        ],
    }


def build_bindings_patch_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    audit = build_binding_gap_audit(module_path, repo_root=repo)
    bindings_helper = _path_from_repo(repo, audit["bindings_helper_path"])
    bindings_helper_text = _load_text(bindings_helper)

    binding_additions = []
    deferred_headers = []
    rust_abstraction_headers = []
    for header in audit["candidate_missing_binding_headers"]:
        decision = _binding_header_plan(header)
        entry = {
            "header": header,
            "reason": decision["reason"],
            "required_symbols": decision["required_symbols"],
            "mvp_blocking": decision["mvp_blocking"],
        }
        action = decision["action"]
        if action == "add_to_bindings":
            binding_additions.append(entry)
        elif action == "defer_to_abstraction":
            deferred_headers.append(entry)
        else:
            rust_abstraction_headers.append(entry)

    patch_units = _build_binding_patch_units(
        bindings_helper_text,
        binding_additions,
        audit["bindings_helper_path"],
    )
    patch_required = bool(binding_additions)

    return {
        "schema_version": 1,
        "artifact_type": "bindings-patch-plan",
        "module_id": audit["module_id"],
        "module_c_path": audit["module_c_path"],
        "bindings_helper_path": audit["bindings_helper_path"],
        "patch_required": patch_required,
        "status": "pending" if patch_required else "already_applied",
        "inputs": {
            "binding_gap_audit": _artifact_path_for(audit["module_id"], "binding-gap-audit.json"),
        },
        "style_constraints": [
            "Keep the top include list in `bindings_helper.h` alphabetically ordered.",
            "Prefer exposing only headers needed by the nlmon abstraction/driver plan; do not mirror every C include blindly.",
        ],
        "binding_additions": sorted(binding_additions, key=lambda entry: entry["header"]),
        "deferred_headers": sorted(deferred_headers, key=lambda entry: entry["header"]),
        "rust_abstraction_headers": sorted(rust_abstraction_headers, key=lambda entry: entry["header"]),
        "patch_units": patch_units,
        "validation_checks": [
            {
                "path": audit["bindings_helper_path"],
                "contains": f"#include <{entry['header']}>",
            }
            for entry in sorted(binding_additions, key=lambda item: item["header"])
        ],
    }


def build_helpers_patch_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    helper_audit = build_helper_audit(module_path, repo_root=repo)
    helpers_aggregate = repo / "rust" / "helpers" / "helpers.c"
    aggregate_text = _load_text(helpers_aggregate)
    wrapper_specs = _helper_wrapper_specs(helper_audit["missing_helper_wrappers"])
    helper_file_name = "net.c"
    insert_after, insert_before = _helper_include_anchor(aggregate_text, helper_file_name)
    anchor_line = (
        _line_number_of_exact_line(aggregate_text, f'#include "{insert_after}"')
        if insert_after
        else _line_number_of_exact_line(aggregate_text, f'#include "{insert_before}"')
    )
    patch_units = []
    if wrapper_specs:
        patch_units = [
            {
                "target_path": f"{helper_audit['helper_dir']}/{helper_file_name}",
                "operation": "create_file",
                "reason": "Add the missing network-specific helper wrappers as a dedicated helper shard.",
                "content_lines": _build_helper_file_content(wrapper_specs),
            },
            {
                "target_path": f"{helper_audit['helper_dir']}/helpers.c",
                "operation": "insert_after" if insert_after else "insert_before",
                "anchor": {
                    "file": insert_after or insert_before,
                    "line": anchor_line,
                },
                "insert_lines": [f'#include "{helper_file_name}"'],
                "reason": "Compile the new helper shard through the existing aggregate translation unit while keeping alphabetical order.",
            },
        ]

    return {
        "schema_version": 1,
        "artifact_type": "helpers-patch-plan",
        "module_id": helper_audit["module_id"],
        "module_c_path": helper_audit["module_c_path"],
        "helper_dir": helper_audit["helper_dir"],
        "patch_required": bool(wrapper_specs),
        "status": "pending" if wrapper_specs else "already_applied",
        "inputs": {
            "helper_audit": _artifact_path_for(helper_audit["module_id"], "helper-audit.json"),
        },
        "wrapper_specs": wrapper_specs,
        "no_makefile_change_required": True,
        "patch_units": patch_units,
        "validation_checks": [
            {
                "path": f"{helper_audit['helper_dir']}/{helper_file_name}",
                "contains": spec["helper_name"],
            }
            for spec in wrapper_specs
        ]
        + (
            [
                {
                    "path": f"{helper_audit['helper_dir']}/helpers.c",
                    "contains": f'#include "{helper_file_name}"',
                }
            ]
            if wrapper_specs
            else []
        ),
    }


def build_abstraction_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    source_text = _load_text(module)
    binding_audit = build_binding_gap_audit(module_path, repo_root=repo)
    helper_audit = build_helper_audit(module_path, repo_root=repo)
    bindings_patch_plan = build_bindings_patch_plan(module_path, repo_root=repo)
    implemented_areas = _implemented_net_areas(repo)
    missing_mvp_areas = [area for area in MVP_NET_MODULES if area not in implemented_areas]
    phase_4_prereqs = []
    if missing_mvp_areas:
        phase_4_prereqs.append(
            "Implement/export the remaining MVP link-type abstractions: "
            + ", ".join(missing_mvp_areas)
            + "."
        )
    if bindings_patch_plan["patch_required"]:
        phase_4_prereqs.append(
            "Expose the remaining nlmon bindings headers before driver codegen."
        )
    if helper_audit["missing_helper_wrappers"]:
        phase_4_prereqs.append(
            "Add the remaining helper wrappers before driver codegen: "
            + ", ".join(helper_audit["missing_helper_wrappers"])
            + "."
        )

    module_id = module.stem
    rtnl_initializer_name = _find_initializer_name(source_text, "rtnl_link_ops")
    netdev_initializer_name = _find_initializer_name(source_text, "net_device_ops")
    ethtool_initializer_name = _find_initializer_name(source_text, "ethtool_ops")

    rtnl_body = _extract_initializer_body(source_text, rtnl_initializer_name) if rtnl_initializer_name else None
    netdev_body = _extract_initializer_body(source_text, netdev_initializer_name) if netdev_initializer_name else None
    ethtool_body = _extract_initializer_body(source_text, ethtool_initializer_name) if ethtool_initializer_name else None

    rtnl_fields = _initializer_field_map(rtnl_body)
    netdev_fields = _initializer_field_map(netdev_body)
    ethtool_fields = _initializer_field_map(ethtool_body)

    setup_body = _extract_function_body(source_text, rtnl_fields.get("setup", "")) if rtnl_fields.get("setup") else None
    validate_body = (
        _extract_function_body(source_text, rtnl_fields.get("validate", ""))
        if rtnl_fields.get("validate")
        else None
    )
    open_body = _extract_function_body(source_text, netdev_fields.get("ndo_open", "")) if netdev_fields.get("ndo_open") else None
    stop_body = _extract_function_body(source_text, netdev_fields.get("ndo_stop", "")) if netdev_fields.get("ndo_stop") else None
    xmit_body = (
        _extract_function_body(source_text, netdev_fields.get("ndo_start_xmit", ""))
        if netdev_fields.get("ndo_start_xmit")
        else None
    )
    stats_body = (
        _extract_function_body(source_text, netdev_fields.get("ndo_get_stats64", ""))
        if netdev_fields.get("ndo_get_stats64")
        else None
    )

    required_callbacks = [
        callback
        for callback in [
            rtnl_fields.get("setup"),
            rtnl_fields.get("validate"),
            netdev_fields.get("ndo_open"),
            netdev_fields.get("ndo_stop"),
            netdev_fields.get("ndo_start_xmit"),
        ]
        if callback
    ]
    deferred_callbacks = [
        callback
        for callback in [
            netdev_fields.get("ndo_get_stats64"),
            ethtool_fields.get("get_link"),
        ]
        if callback
    ]

    setup_writes = _extract_pointer_field_assignments(setup_body, "dev->")
    tap_open_assignments = _extract_pointer_field_assignments(open_body, "nlmon->nt.")

    return {
        "schema_version": 1,
        "artifact_type": "abstraction-plan",
        "module_id": module_id,
        "module_c_path": _rel(repo, module),
        "inputs": {
            "kbuild_patch_plan": _artifact_path_for(module_id, "kbuild-patch-plan.json"),
            "bindings_patch_plan": _artifact_path_for(module_id, "bindings-patch-plan.json"),
            "helpers_patch_plan": _artifact_path_for(module_id, "helpers-patch-plan.json"),
        },
        "goal": "Constrain Phase 4 so the first Rust loop reaches the `ip link add/up/down/del nlmon0` smoke closure without unconstrained driver invention.",
        "current_rust_net_scope": {
            "rust_net_root": _rel(repo, repo / "rust" / "kernel" / "net.rs"),
            "modules": binding_audit["current_rust_net_modules"],
            "implemented_areas": sorted(implemented_areas),
            "assessment": _net_scope_assessment(
                binding_audit["current_rust_net_modules"],
                implemented_areas,
            ),
        },
        "source_inventory": {
            "private_struct_fields": _extract_private_struct_fields(source_text, module_id),
            "rtnl_link_ops": {
                "name": rtnl_initializer_name,
                "fields": _extract_initializer_fields(rtnl_body),
            },
            "net_device_ops": {
                "name": netdev_initializer_name,
                "fields": _extract_initializer_fields(netdev_body),
            },
            "ethtool_ops": {
                "name": ethtool_initializer_name,
                "fields": _extract_initializer_fields(ethtool_body),
            },
            "setup_field_writes": setup_writes,
            "validate_checks": _extract_validate_checks(validate_body),
            "open_calls": _extract_call_sites(open_body),
            "open_tap_assignments": tap_open_assignments,
            "stop_calls": _extract_call_sites(stop_body),
            "xmit_calls": _extract_call_sites(xmit_body),
            "stats_calls": _extract_call_sites(stats_body),
        },
        "mvp_scope": {
            "target_command": "ip link add nlmon0 type nlmon && ip link set nlmon0 up && ip link set nlmon0 down && ip link del nlmon0",
            "required_callbacks": required_callbacks,
            "deferred_callbacks": deferred_callbacks,
            "note": "The first Rust loop must already cover the smoke-path lifecycle. `xmit` is required for `ip link set ... up`; `get_stats64` and `ethtool` parity can still land later.",
        },
        "abstraction_areas": [
            {
                "area": "rtnl",
                "priority": "mvp-blocker",
                "status": "implemented" if "rtnl" in implemented_areas else "missing",
                "implementation_path": MVP_NET_MODULES["rtnl"] if "rtnl" in implemented_areas else None,
                "c_evidence": [
                    item
                    for item in [
                        "struct rtnl_link_ops",
                        "rtnl_link_register",
                        "rtnl_link_unregister",
                        rtnl_fields.get("setup"),
                        rtnl_fields.get("validate"),
                    ]
                    if item
                ],
                "required_surface": [
                    "A registration owner that balances `rtnl_link_register`/`rtnl_link_unregister`.",
                    "A typed link-ops description carrying `kind`, `priv_size`, `setup`, and `validate`.",
                    "Callback shims that keep raw `nlattr`/`netlink_ext_ack` handling out of driver code.",
                ],
                "deferred_surface": [
                    "Optional `newlink`/`changelink`/`dellink` hooks.",
                    "xstats and queue-count helpers.",
                ],
                "depends_on": [
                    _artifact_path_for(module_id, "bindings-patch-plan.json"),
                ],
                "preferred_files": [
                    "rust/kernel/net/rtnl.rs",
                    "rust/kernel/net.rs",
                ],
            },
            {
                "area": "netdevice",
                "priority": "mvp-blocker",
                "status": "implemented" if "netdevice" in implemented_areas else "missing",
                "implementation_path": MVP_NET_MODULES["netdevice"] if "netdevice" in implemented_areas else None,
                "c_evidence": [
                    "struct net_device_ops",
                    "struct net_device",
                    "struct ethtool_ops",
                ],
                "required_surface": [
                    "A typed `net_device` wrapper for `setup` field writes and private storage access.",
                    "A `net_device_ops`-style trait covering `open`, `stop`, and an ownership-safe `start_xmit` surface.",
                    "Enough constants/enums to set `type`, `flags`, `priv_flags`, `needs_free_netdev`, and MTU bounds.",
                ],
                "deferred_surface": [
                    "Dedicated `ethtool_ops` wrapper for `get_link`.",
                    "A typed `get_stats64` callback surface.",
                ],
                "depends_on": [
                    _artifact_path_for(module_id, "bindings-patch-plan.json"),
                    _artifact_path_for(module_id, "helpers-patch-plan.json"),
                ],
                "preferred_files": [
                    "rust/kernel/net/netdevice.rs",
                    "rust/kernel/net.rs",
                ],
            },
            {
                "area": "netlink_tap",
                "priority": "mvp-blocker",
                "status": "implemented" if "netlink_tap" in implemented_areas else "missing",
                "implementation_path": MVP_NET_MODULES["netlink_tap"] if "netlink_tap" in implemented_areas else None,
                "c_evidence": [
                    "struct netlink_tap",
                    "netlink_add_tap",
                    "netlink_remove_tap",
                    "THIS_MODULE",
                ],
                "required_surface": [
                    "A private-state-owned tap handle that stores `dev` and `module` before registration.",
                    "Open/stop helper methods that call add/remove in matched lifecycle points.",
                ],
                "deferred_surface": [],
                "depends_on": [
                    _artifact_path_for(module_id, "bindings-patch-plan.json"),
                ],
                "preferred_files": [
                    "rust/kernel/net/netlink_tap.rs",
                    "rust/kernel/net.rs",
                ],
            },
            {
                "area": "stats",
                "priority": "mvp-blocker",
                "status": "implemented" if "stats" in implemented_areas else "missing",
                "implementation_path": MVP_NET_MODULES["stats"] if "stats" in implemented_areas else None,
                "c_evidence": [
                    "dev_lstats_add",
                    "dev_lstats_read",
                    "NETDEV_PCPU_STAT_LSTATS",
                ],
                "required_surface": [
                    "Expose the `NETDEV_PCPU_STAT_LSTATS` configuration constant for `setup` if Phase 4 keeps that field write.",
                    "Provide a helper-backed `dev_lstats_add` wrapper that the driver can call from `start_xmit` without touching helper FFI directly.",
                ],
                "deferred_surface": [
                    "A typed wrapper for `dev_lstats_read` and `rtnl_link_stats64`.",
                ],
                "depends_on": [
                    _artifact_path_for(module_id, "helpers-patch-plan.json"),
                    _artifact_path_for(module_id, "bindings-patch-plan.json"),
                ],
                "preferred_files": [
                    "rust/kernel/net/stats.rs",
                    "rust/kernel/net.rs",
                ],
            },
            {
                "area": "skbuff",
                "priority": "mvp-blocker",
                "status": "implemented" if "skbuff" in implemented_areas else "missing",
                "implementation_path": MVP_NET_MODULES["skbuff"] if "skbuff" in implemented_areas else None,
                "c_evidence": [
                    "struct sk_buff",
                    "dev_kfree_skb",
                ],
                "required_surface": [
                    "A move-only skb owner for `ndo_start_xmit` that frees on drop and can hand ownership back to the core on `NETDEV_TX_BUSY`.",
                ],
                "deferred_surface": [
                    "Packet mutation helpers beyond `len()`.",
                ],
                "depends_on": [
                    _artifact_path_for(module_id, "helpers-patch-plan.json"),
                ],
                "preferred_files": [
                    "rust/kernel/net/skbuff.rs",
                    "rust/kernel/net.rs",
                ],
            },
        ],
        "phase_4_readiness": {
            "ready_for_minimal_driver_codegen": not phase_4_prereqs,
            "required_before_driver_codegen": phase_4_prereqs,
            "driver_generation_rules": [
                "Driver code may borrow style from `drivers/net/phy/ax88796b_rust.rs` / `rust/kernel/net/phy.rs`, but only for module/abstraction layering.",
                "Driver code must not call raw `bindings::rtnl_link_register`, `bindings::netlink_add_tap`, or helper exports directly.",
                "Any Phase 4 expansion beyond the `mvp_scope.deferred_callbacks` list requires updating this artifact first.",
            ],
            "helper_gaps": helper_audit["missing_helper_wrappers"],
        },
    }


def build_translation_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    module_id = module.stem
    abstraction_plan = build_abstraction_plan(module_path, repo_root=repo)
    unsafe_plan = build_unsafe_obligations(module_path, repo_root=repo)
    kbuild_patch_plan = build_kbuild_patch_plan(module_path, repo_root=repo)
    bindings_patch_plan = build_bindings_patch_plan(module_path, repo_root=repo)
    helpers_patch_plan = build_helpers_patch_plan(module_path, repo_root=repo)
    inventory = abstraction_plan["source_inventory"]

    readiness_blockers = []
    if kbuild_patch_plan["patch_required"]:
        readiness_blockers.append("Apply the Kbuild switch so `nlmon_rust.o` can replace `nlmon.o`.")
    if bindings_patch_plan["patch_required"]:
        readiness_blockers.append("Apply the bindings header additions before generating driver code.")
    if helpers_patch_plan["patch_required"]:
        readiness_blockers.append("Apply the helper wrapper patch before generating driver code.")
    readiness_blockers.extend(abstraction_plan["phase_4_readiness"]["required_before_driver_codegen"])

    setup_translations = []
    setup_translation_map = {
        "type": {
            "status": "required",
            "rust_surface": "dev.set_type(netdevice::device_type::NETLINK)",
            "note": "Keep device type selection inside the typed `netdevice::Device` wrapper.",
        },
        "priv_flags": {
            "status": "required",
            "rust_surface": "dev.add_priv_flag(netdevice::priv_flags::NO_QUEUE)",
            "note": "Preserve the `IFF_NO_QUEUE` setup bit without raw field access.",
        },
        "lltx": {
            "status": "required",
            "rust_surface": "dev.set_lltx(true)",
            "note": "Keep lockless TX configuration in the wrapper layer.",
        },
        "netdev_ops": {
            "status": "handled_by_abstraction",
            "rust_surface": "implicit via `rtnl::Registration::<NlmonDriver>::setup_callback`",
            "note": "Do not assign `dev->netdev_ops` in driver code; the rtnl abstraction wires it before `setup`.",
        },
        "ethtool_ops": {
            "status": "deferred",
            "rust_surface": "omit in MVP",
            "note": "Keep `ethtool_ops` out of the first loop together with `always_on`.",
        },
        "needs_free_netdev": {
            "status": "required",
            "rust_surface": "dev.set_needs_free_netdev(true)",
            "note": "Keep free-on-destroy policy inside the wrapper.",
        },
        "features": {
            "status": "required",
            "rust_surface": "dev.set_features(netdevice::features::SG | netdevice::features::FRAGLIST | netdevice::features::HIGHDMA)",
            "note": "Keep the feature bit expression inside the typed netdevice abstraction surface.",
        },
        "flags": {
            "status": "required",
            "rust_surface": "dev.set_flags(netdevice::flags::NO_ARP)",
            "note": "Preserve the `IFF_NOARP` setup bit through the typed wrapper.",
        },
        "pcpu_stat_type": {
            "status": "mvp-allowed",
            "rust_surface": "dev.set_pcpu_stat_type(netdevice::pcpu_stat_type::LSTATS)",
            "note": "Keep the setup write for parity, but do not enable `ndo_get_stats64` yet.",
        },
        "mtu": {
            "status": "required",
            "rust_surface": "dev.set_mtu(netdevice::mtu::nlmsg_goodsize())",
            "note": "Keep the softlimit MTU assignment explicit in `setup`, but hide bindgen-dependent constants behind the abstraction.",
        },
        "min_mtu": {
            "status": "required",
            "rust_surface": "dev.set_min_mtu(netdevice::mtu::NLMSGHDR)",
            "note": "Expose the `nlmsghdr` size through the abstraction rather than direct bindgen usage in driver code.",
        },
    }

    for write in inventory["setup_field_writes"]:
        mapped = setup_translation_map.get(write["field"])
        entry = {
            "c_field": write["field"],
            "c_operator": write["operator"],
            "c_value": write["value"],
        }
        if mapped:
            entry.update(mapped)
        else:
            entry.update(
                {
                    "status": "manual-review",
                    "rust_surface": None,
                    "note": "No translation rule is defined yet; update the artifact before generating this field write.",
                }
            )
        setup_translations.append(entry)

    validate_translation = []
    for check in inventory["validate_checks"]:
        if check["field"] == "IFLA_ADDRESS" and check["return"] == "-EINVAL":
            validate_translation.append(
                {
                    "status": "required",
                    "c_rule": "if (tb[IFLA_ADDRESS]) return -EINVAL;",
                    "rust_surface": "if ctx.has_link_attr(rtnl::LinkAttr::ADDRESS) { return Err(EINVAL); }",
                    "note": "Use `ValidateContext`/typed `LinkAttr` instead of raw `tb[]` indexing.",
                }
            )
        else:
            validate_translation.append(
                {
                    "status": "manual-review",
                    "c_rule": f"tb[{check['field']}] => {check['return']}",
                    "rust_surface": None,
                    "note": "Unhandled validate rule; update the artifact before code generation.",
                }
            )

    callback_mapping = [
        {
            "status": "required",
            "c_symbol": next(
                (field["value"] for field in inventory["rtnl_link_ops"]["fields"] if field["field"] == "setup"),
                "nlmon_setup",
            ),
            "rust_surface": "impl rtnl::Driver for NlmonDriver::setup",
            "body_plan": [
                "Only use the typed `netdevice::Device` setters listed in `setup_translation`.",
                "Do not assign `netdev_ops` directly; the rtnl abstraction does that automatically.",
            ],
        },
        {
            "status": "required",
            "c_symbol": next(
                (field["value"] for field in inventory["rtnl_link_ops"]["fields"] if field["field"] == "validate"),
                "nlmon_validate",
            ),
            "rust_surface": "impl rtnl::Driver for NlmonDriver::validate",
            "body_plan": [
                "Express the current MVP rule through `ValidateContext` only.",
                "Return `Err(EINVAL)` when `IFLA_ADDRESS` is present.",
            ],
        },
        {
            "status": "required",
            "c_symbol": next(
                (field["value"] for field in inventory["net_device_ops"]["fields"] if field["field"] == "ndo_open"),
                "nlmon_open",
            ),
            "rust_surface": "impl netdevice::Operations for NlmonDriver::open",
            "body_plan": [
                "Access private state through the typed `private` argument only.",
                "Call `private.tap.add(dev, &THIS_MODULE)?`.",
            ],
        },
        {
            "status": "required",
            "c_symbol": next(
                (field["value"] for field in inventory["net_device_ops"]["fields"] if field["field"] == "ndo_stop"),
                "nlmon_close",
            ),
            "rust_surface": "impl netdevice::Operations for NlmonDriver::stop",
            "body_plan": [
                "Access private state through the typed `private` argument only.",
                "Call `private.tap.remove()?`.",
            ],
        },
        {
            "status": "required",
            "c_symbol": next(
                (field["value"] for field in inventory["net_device_ops"]["fields"] if field["field"] == "ndo_start_xmit"),
                "nlmon_xmit",
            ),
            "rust_surface": "impl netdevice::Operations for NlmonDriver::start_xmit",
            "body_plan": [
                "Receive the packet as owned `skbuff::SkBuff` and the device as shared `&netdevice::Device` only.",
                "Call `stats::dev_lstats_add(dev, skb.len())` before returning.",
                "Return `netdevice::TxOutcome::Ok`; let the skb ownership discharge through the abstraction rather than explicit free calls in driver code.",
            ],
        },
    ]

    for deferred_symbol in abstraction_plan["mvp_scope"]["deferred_callbacks"]:
        callback_mapping.append(
            {
                "status": "deferred",
                "c_symbol": deferred_symbol,
                "rust_surface": None,
                "body_plan": [
                    "Keep this callback out of `drivers/net/nlmon_rust.rs` for the first loop.",
                ],
            }
        )

    return {
        "schema_version": 1,
        "artifact_type": "translation-plan",
        "module_id": module_id,
        "module_c_path": _rel(repo, module),
        "driver_rust_path": _rel(repo, module.parent / f"{module_id}_rust.rs"),
        "inputs": {
            "kbuild_patch_plan": _artifact_path_for(module_id, "kbuild-patch-plan.json"),
            "bindings_patch_plan": _artifact_path_for(module_id, "bindings-patch-plan.json"),
            "helpers_patch_plan": _artifact_path_for(module_id, "helpers-patch-plan.json"),
            "abstraction_plan": _artifact_path_for(module_id, "abstraction-plan.json"),
            "unsafe_obligations": _artifact_path_for(module_id, "unsafe-obligations.json"),
        },
        "goal": "Constrain the first `drivers/net/nlmon_rust.rs` to a minimal, artifact-driven smoke loop covering registration, setup, validate, open, stop, and xmit.",
        "readiness": {
            "ready_for_minimal_driver_codegen": not readiness_blockers,
            "blockers": readiness_blockers,
            "upstream_artifact_status": {
                "kbuild_patch_plan": kbuild_patch_plan["status"],
                "bindings_patch_plan": bindings_patch_plan["status"],
                "helpers_patch_plan": helpers_patch_plan["status"],
                "abstraction_plan": (
                    "ready" if abstraction_plan["phase_4_readiness"]["ready_for_minimal_driver_codegen"] else "blocked"
                ),
            },
        },
        "module_shell": {
            "module_macro_required": True,
            "preferred_module_type": "NlmonModule",
            "preferred_driver_type": "NlmonDriver",
            "preferred_private_type": "NlmonPriv",
            "preferred_module_trait": "kernel::InPlaceModule",
            "owned_registration": "rtnl::Registration<NlmonDriver>",
            "registration_kind": "c_str!(\"nlmon\")",
            "module_aliases": ["rtnl-link-nlmon"],
        },
        "private_state": {
            "type_name": "NlmonPriv",
            "layout": "#[repr(C)]",
            "init_policy": {
                "allocation_source": "RTNL/net core zero-initialized private storage",
                "rust_requirement": "The private type must accept the all-zero bit pattern (`Zeroable`).",
            },
            "fields": [
                {
                    "name": "tap",
                    "rust_type": "netlink_tap::Tap",
                    "source_c_field": "nt",
                    "note": "Own the netlink tap in Rust private storage; the MVP flow only uses it in open/stop.",
                }
            ],
        },
        "callback_mapping": callback_mapping,
        "setup_translation": setup_translations,
        "validate_translation": validate_translation,
        "deferred_callbacks": abstraction_plan["mvp_scope"]["deferred_callbacks"],
        "allowed_driver_surfaces": [
            "kernel::net::rtnl::{Registration, Driver, ValidateContext, LinkAttr}",
            "kernel::net::netdevice::{Device, Operations, TxOutcome, device_type, flags, priv_flags, pcpu_stat_type, features, mtu}",
            "kernel::net::netlink_tap::Tap",
            "kernel::net::skbuff::SkBuff",
            "kernel::net::stats",
            "&THIS_MODULE",
        ],
        "forbidden_driver_calls": [
            "bindings::rtnl_link_register",
            "bindings::rtnl_link_unregister",
            "bindings::netlink_add_tap",
            "bindings::netlink_remove_tap",
            "bindings::netdev_priv",
            "bindings::dev_kfree_skb",
            "bindings::dev_lstats_add",
        ],
        "unsafe_policy": {
            "driver_unsafe_expected": False,
            "note": "The first Rust loop should keep raw pointer work and bindgen coupling inside the abstractions named above; driver code must remain free of `unsafe` and direct `bindings::*` usage.",
            "linked_obligations": [entry["id"] for entry in unsafe_plan["obligations"]],
        },
        "driver_policy": {
            "driver_zero_unsafe": True,
            "driver_zero_bindings": True,
            "driver_zero_extern_c": True,
        },
    }


def build_unsafe_obligations(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    abstraction_plan = build_abstraction_plan(module_path, repo_root=repo)
    inventory = abstraction_plan["source_inventory"]
    module_id = abstraction_plan["module_id"]

    return {
        "schema_version": 1,
        "artifact_type": "unsafe-obligations",
        "module_id": module_id,
        "module_c_path": abstraction_plan["module_c_path"],
        "inputs": {
            "abstraction_plan": _artifact_path_for(module_id, "abstraction-plan.json"),
        },
        "obligations": [
            {
                "id": "rtnl-registration-lifetime",
                "phase": "mvp",
                "area": "rtnl",
                "category": "lifetime",
                "evidence": ["rtnl_link_register", "rtnl_link_unregister", "module_init", "module_exit"],
                "obligation": "The Rust registration object must own a stable `rtnl_link_ops` table and unregister exactly once before module teardown.",
                "preferred_discharge": "Hide raw register/unregister behind an RAII-style `Registration` wrapper in `rust/kernel/net/rtnl.rs`.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "netdev-private-layout",
                "phase": "mvp",
                "area": "netdevice",
                "category": "pointer-cast",
                "evidence": ["netdev_priv", ".priv_size = sizeof(struct nlmon)"],
                "obligation": "Casting `net_device` private storage into Rust state requires a proof that allocation size/layout matches the Rust private type.",
                "preferred_discharge": "Couple `priv_size` to the Rust private-state type in the rtnl/netdevice abstraction and funnel access through the helper wrapper.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "netdev-private-zero-init",
                "phase": "mvp",
                "area": "netdevice",
                "category": "initialization",
                "evidence": [
                    "alloc_netdev/RTNL private storage is zero-initialized",
                    ".priv_size = sizeof(struct nlmon)",
                ],
                "obligation": "A Rust private-state type stored in `net_device` private storage must accept the all-zero bit pattern before any callback observes it.",
                "preferred_discharge": "Require the private-state type to be `Zeroable` and keep any non-zero initialization requirement out of the MVP callback set.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "tap-module-pointer-validity",
                "phase": "mvp",
                "area": "netlink_tap",
                "category": "ffi-pointer",
                "evidence": inventory["open_tap_assignments"] + [{"field": "module", "value": "THIS_MODULE"}],
                "obligation": "Before `netlink_add_tap`, Rust must populate `netlink_tap.dev` and `netlink_tap.module` with valid pointers that outlive the registration.",
                "preferred_discharge": "Thread `&'static ThisModule` through the abstraction and write `ThisModule::as_ptr()` inside the wrapper.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "tap-open-stop-balance",
                "phase": "mvp",
                "area": "netlink_tap",
                "category": "state-machine",
                "evidence": inventory["open_calls"] + inventory["stop_calls"],
                "obligation": "Open/stop shims must keep tap registration balanced and avoid double-remove on partially initialized state.",
                "preferred_discharge": "Model tap state explicitly in the wrapper or rely on netdevice lifecycle invariants while keeping transitions localized.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "callback-pointer-aliasing",
                "phase": "mvp",
                "area": "rtnl+netdevice",
                "category": "aliasing",
                "evidence": abstraction_plan["mvp_scope"]["required_callbacks"],
                "obligation": "Raw callback shims must create only short-lived Rust references from C pointers and must not assume broader aliasing guarantees than the kernel provides.",
                "preferred_discharge": "Keep all pointer-to-reference conversion inside callback trampolines in the abstraction layer.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "validate-attribute-indexing",
                "phase": "mvp",
                "area": "rtnl",
                "category": "bounds",
                "evidence": inventory["validate_checks"],
                "obligation": "Access to `tb[IFLA_ADDRESS]` must preserve kernel indexing rules and nullability checks when surfaced to Rust.",
                "preferred_discharge": "Provide a safe `NlAttrTable` accessor instead of exposing raw pointer arithmetic to driver code.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "stats-buffer-write",
                "phase": "post-mvp",
                "area": "stats",
                "category": "mutable-ffi-output",
                "evidence": inventory["stats_calls"],
                "obligation": "Any Rust wrapper for `get_stats64` must guarantee that the output stats buffer is writable and initialized exactly as expected by `dev_lstats_read`.",
                "preferred_discharge": "Introduce a typed `Stats64Mut` wrapper before enabling `ndo_get_stats64` in Phase 4.",
                "allowed_unsafe_location": "abstraction-only",
            },
            {
                "id": "skb-consumed-once",
                "phase": "mvp",
                "area": "skbuff",
                "category": "ownership",
                "evidence": inventory["xmit_calls"],
                "obligation": "The xmit path must consume/free each skb exactly once; Rust code must not duplicate ownership across helper and binding calls.",
                "preferred_discharge": "Model packets as move-only `skbuff::SkBuff`; `TxOutcome::Ok` consumes via drop and `TxOutcome::Busy(skb)` returns ownership to the core explicitly.",
                "allowed_unsafe_location": "abstraction-only",
            },
        ],
        "driver_side_rules": [
            "Do not use `unsafe` in driver code, including `unsafe impl`, `unsafe fn`, or `unsafe {}` blocks.",
            "Do not reference `bindings::*` directly from driver code; route constants and types through approved abstractions.",
            "Do not cast `net_device` private storage inside `drivers/net/nlmon_rust.rs`.",
            "Do not add new raw FFI calls in driver code beyond the abstractions named in `abstraction-plan.json`.",
            "Driver `start_xmit` may only use `skbuff::SkBuff`, `stats::dev_lstats_add`, and `netdevice::TxOutcome`; ownership must stay inside those safe surfaces.",
            "Keep `ndo_get_stats64` disabled/deferred until the post-MVP obligations above have an explicit discharge path.",
        ],
    }


def build_safety_policy(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    module_id = module.stem
    abstraction_plan = build_abstraction_plan(module_path, repo_root=repo)
    translation_plan = build_translation_plan(module_path, repo_root=repo)

    allowlisted_files = [
        entry["implementation_path"]
        for entry in abstraction_plan["abstraction_areas"]
        if entry["status"] == "implemented" and entry["implementation_path"]
    ]
    allowlisted_files = [path for path in allowlisted_files if path.startswith("rust/kernel/net/")]

    return {
        "schema_version": 1,
        "artifact_type": "safety-policy",
        "module_id": module_id,
        "module_c_path": _rel(repo, module),
        "driver_rust_path": translation_plan["driver_rust_path"],
        "inputs": {
            "abstraction_plan": _artifact_path_for(module_id, "abstraction-plan.json"),
            "translation_plan": _artifact_path_for(module_id, "translation-plan.json"),
            "unsafe_obligations": _artifact_path_for(module_id, "unsafe-obligations.json"),
            "soundness_discharge": _artifact_path_for(module_id, "soundness-discharge.json"),
        },
        "driver_policy": {
            "zero_unsafe": True,
            "zero_bindings": True,
            "zero_extern_c": True,
            "required_crate_attributes": ["#![forbid(unsafe_code)]"],
            "forbidden_tokens": ["unsafe", "bindings::", 'extern "C"', "*mut ", "*const "],
            "forbidden_calls": translation_plan["forbidden_driver_calls"],
            "allowed_surfaces": [
                "kernel::net::rtnl::{Registration, Driver, ValidateContext, LinkAttr}",
                "kernel::net::netdevice::{Device, Operations, TxOutcome, device_type, flags, priv_flags, pcpu_stat_type, features, mtu}",
                "kernel::net::netlink_tap::Tap",
                "kernel::net::skbuff::SkBuff",
                "kernel::net::stats",
                "kernel::ThisModule",
                "&THIS_MODULE",
            ],
        },
        "abstraction_policy": {
            "allowlisted_files": allowlisted_files,
            "unsafe_trait_impls_are_proof_sites": True,
            "required_soundness_rules": [
                {
                    "id": "typed-link-attr-api",
                    "linked_obligation_ids": ["validate-attribute-indexing"],
                    "file": "rust/kernel/net/rtnl.rs",
                    "must_contain": [
                        "pub struct LinkAttr",
                        "pub struct InfoAttr",
                        "pub fn has_link_attr(&self, attr: LinkAttr) -> bool",
                        "pub fn has_info_attr(&self, attr: InfoAttr) -> bool",
                        "const LINK_ATTR_TABLE_LEN: usize = bindings::__IFLA_MAX as usize;",
                        "const INFO_ATTR_TABLE_LEN: usize = bindings::__IFLA_INFO_MAX as usize;",
                    ],
                    "must_not_contain": [
                        "pub fn has_link_attr(&self, attr: usize)",
                        "pub fn has_data_attr(&self, attr: usize)",
                    ],
                },
                {
                    "id": "pinned-netlink-tap-api",
                    "linked_obligation_ids": ["tap-module-pointer-validity", "tap-open-stop-balance"],
                    "file": "rust/kernel/net/netlink_tap.rs",
                    "must_contain": [
                        "pub fn add(self: Pin<&mut Self>",
                        "pub fn remove(self: Pin<&mut Self>) -> Result",
                    ],
                    "must_not_contain": [
                        "pub fn add(&mut self",
                        "pub fn remove(&mut self)",
                    ],
                },
                {
                    "id": "private-state-no-drop",
                    "linked_obligation_ids": ["netdev-private-zero-init", "netdev-private-layout"],
                    "file": "rust/kernel/net/rtnl.rs",
                    "must_contain": [
                        "build_assert!(!core::mem::needs_drop::<T::Private>())",
                    ],
                    "must_not_contain": [],
                },
                {
                    "id": "shared-xmit-device-api",
                    "linked_obligation_ids": ["callback-pointer-aliasing", "skb-consumed-once"],
                    "file": "rust/kernel/net/netdevice.rs",
                    "must_contain": [
                        "fn start_xmit(skb: skbuff::SkBuff, dev: &Device) -> TxOutcome;",
                        "extern \"C\" fn start_xmit_callback(",
                        "let dev = unsafe { Device::from_raw_ref(dev) };",
                        "let skb = unsafe { skbuff::SkBuff::from_raw_owned(skb) };",
                        "pub(crate) fn as_ptr(&self) -> *mut bindings::net_device {",
                        "TxOutcome::Busy(skb) =>",
                    ],
                    "must_not_contain": [
                        "fn start_xmit(skb: skbuff::SkBuff, dev: &mut Device) -> TxOutcome;",
                        "pub fn as_ptr(&self) -> *mut bindings::net_device {",
                    ],
                },
                {
                    "id": "move-only-skb-api",
                    "linked_obligation_ids": ["skb-consumed-once"],
                    "file": "rust/kernel/net/skbuff.rs",
                    "must_contain": [
                        "pub struct SkBuff",
                        "pub(crate) unsafe fn from_raw_owned",
                        "pub(crate) fn into_raw(mut self) -> *mut bindings::sk_buff",
                        "impl Drop for SkBuff",
                    ],
                    "must_not_contain": [
                        "impl Clone for SkBuff",
                        "impl Copy for SkBuff",
                        "#[derive(Clone",
                        "#[derive(Copy",
                    ],
                },
                {
                    "id": "shared-device-stats-api",
                    "linked_obligation_ids": ["callback-pointer-aliasing"],
                    "file": "rust/kernel/net/stats.rs",
                    "must_contain": [
                        "pub fn dev_lstats_add(dev: &netdevice::Device, len: u32) {",
                        "bindings::dev_lstats_add",
                    ],
                    "must_not_contain": [
                        "pub fn dev_lstats_add(dev: &mut netdevice::Device, len: u32) {",
                    ],
                },
            ],
        },
    }


def build_soundness_discharge(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    module_id = module.stem
    safety_policy = build_safety_policy(module_path, repo_root=repo)
    unsafe_plan = build_unsafe_obligations(module_path, repo_root=repo)
    allowlisted_files = safety_policy["abstraction_policy"]["allowlisted_files"]

    obligation_map = {entry["id"]: entry for entry in unsafe_plan["obligations"]}
    proof_sites = []
    for index, site in enumerate(_find_rust_unsafe_sites(repo, allowlisted_files), start=1):
        linked_obligation_ids = _guess_obligation_ids(site["file"], site["source_excerpt"])
        preferred_discharge = [
            obligation_map[obligation_id]["preferred_discharge"]
            for obligation_id in linked_obligation_ids
            if obligation_id in obligation_map
        ]
        proof_sites.append(
            {
                "id": f"proof-site-{index}",
                "file": site["file"],
                "line": site["line"],
                "unsafe_kind": site["unsafe_kind"],
                "source_excerpt": site["source_excerpt"],
                "linked_obligation_ids": linked_obligation_ids,
                "safe_api_surface": Path(site["file"]).stem,
                "caller_obligations": [
                    "Keep the raw pointer/reference conversion inside the abstraction boundary.",
                ],
                "kernel_preconditions": [
                    "The kernel callback or registration path upholds the pointed-to object lifetime.",
                ],
                "proof_argument": " ".join(preferred_discharge) if preferred_discharge else "No matching unsafe obligation was found.",
                "evidence_refs": [_artifact_path_for(module_id, "unsafe-obligations.json")],
                "status": "discharged" if linked_obligation_ids else "blocked",
            }
        )

    driver_path = _path_from_repo(repo, safety_policy["driver_rust_path"])
    driver_text = _load_text(driver_path) if driver_path.exists() else ""
    stripped_driver = _strip_rust_noncode(driver_text)

    structural_rules = []
    for rule in safety_policy["abstraction_policy"]["required_soundness_rules"]:
        path = _path_from_repo(repo, rule["file"])
        text = _load_text(path) if path.exists() else ""
        normalized_text = _normalize_whitespace(text)
        compact_text = _compact_whitespace(text)
        missing = [
            pattern
            for pattern in rule["must_contain"]
            if pattern not in text
            and _normalize_whitespace(pattern) not in normalized_text
            and _compact_whitespace(pattern) not in compact_text
        ]
        unexpected = [
            pattern
            for pattern in rule["must_not_contain"]
            if pattern in text
            or _normalize_whitespace(pattern) in normalized_text
            or _compact_whitespace(pattern) in compact_text
        ]
        structural_rules.append(
            {
                "id": rule["id"],
                "file": rule["file"],
                "linked_obligation_ids": rule["linked_obligation_ids"],
                "must_contain": rule["must_contain"],
                "must_not_contain": rule["must_not_contain"],
                "status": "discharged" if not missing and not unexpected else "blocked",
                "missing_patterns": missing,
                "unexpected_patterns": unexpected,
            }
        )

    for crate_attr in safety_policy["driver_policy"]["required_crate_attributes"]:
        structural_rules.append(
            {
                "id": "driver-forbid-unsafe-code-attr",
                "file": safety_policy["driver_rust_path"],
                "linked_obligation_ids": [],
                "must_contain": [crate_attr],
                "must_not_contain": [],
                "status": "discharged" if crate_attr in driver_text else "blocked",
                "missing_patterns": [] if crate_attr in driver_text else [crate_attr],
                "unexpected_patterns": [],
            }
        )
    driver_rules = [
        {
            "id": "driver-zero-unsafe",
            "status": "discharged" if not re.search(r"\bunsafe\b", stripped_driver) else "blocked",
            "must_not_contain": ["unsafe"],
        },
        {
            "id": "driver-zero-bindings",
            "status": "discharged" if "bindings::" not in stripped_driver else "blocked",
            "must_not_contain": ["bindings::"],
        },
        {
            "id": "driver-zero-ffi",
            "status": "discharged" if 'extern "C"' not in stripped_driver else "blocked",
            "must_not_contain": ['extern "C"'],
        },
    ]

    return {
        "schema_version": 1,
        "artifact_type": "soundness-discharge",
        "module_id": module_id,
        "module_c_path": _rel(repo, module),
        "driver_rust_path": safety_policy["driver_rust_path"],
        "inputs": {
            "safety_policy": _artifact_path_for(module_id, "safety-policy.json"),
            "unsafe_obligations": _artifact_path_for(module_id, "unsafe-obligations.json"),
        },
        "proof_sites": proof_sites,
        "structural_rules": structural_rules,
        "driver_rules": driver_rules,
    }


def build_agent_workflow_plan(module_path: str | Path, *, repo_root: str | Path | None = None) -> dict:
    repo = _repo_root(repo_root)
    module = _path_from_repo(repo, module_path)
    module_id = module.stem

    abstraction_plan = build_abstraction_plan(module_path, repo_root=repo)
    translation_plan = build_translation_plan(module_path, repo_root=repo)
    safety_policy = build_safety_policy(module_path, repo_root=repo)

    driver_rust_path = translation_plan["driver_rust_path"]
    managed_abstraction_files = safety_policy["abstraction_policy"]["allowlisted_files"]
    managed_files = [driver_rust_path, *managed_abstraction_files]
    driver_object_path = driver_rust_path.removesuffix(".rs") + ".o"

    readiness_blockers = []
    if not translation_plan["readiness"]["ready_for_minimal_driver_codegen"]:
        readiness_blockers.extend(translation_plan["readiness"]["blockers"])

    return {
        "schema_version": 1,
        "artifact_type": "agent-workflow-plan",
        "module_id": module_id,
        "module_c_path": _rel(repo, module),
        "driver_rust_path": driver_rust_path,
        "driver_object_path": driver_object_path,
        "inputs": {
            "abstraction_plan": _artifact_path_for(module_id, "abstraction-plan.json"),
            "translation_plan": _artifact_path_for(module_id, "translation-plan.json"),
            "safety_policy": _artifact_path_for(module_id, "safety-policy.json"),
            "soundness_discharge": _artifact_path_for(module_id, "soundness-discharge.json"),
            "unsafe_obligations": _artifact_path_for(module_id, "unsafe-obligations.json"),
        },
        "preflight": {
            "ready_for_agent_codegen": not readiness_blockers,
            "blockers": readiness_blockers,
            "required_reads": [
                _artifact_path_for(module_id, "abstraction-plan.json"),
                _artifact_path_for(module_id, "translation-plan.json"),
                _artifact_path_for(module_id, "safety-policy.json"),
                _artifact_path_for(module_id, "soundness-discharge.json"),
                _artifact_path_for(module_id, "unsafe-obligations.json"),
            ],
        },
        "generation_scope": {
            "managed_files": managed_files,
            "driver_files": [driver_rust_path],
            "abstraction_files": managed_abstraction_files,
            "deferred_callbacks": translation_plan["deferred_callbacks"],
            "forbidden_driver_calls": translation_plan["forbidden_driver_calls"],
            "allowed_driver_surfaces": translation_plan["allowed_driver_surfaces"],
            "required_driver_crate_attributes": safety_policy["driver_policy"]["required_crate_attributes"],
        },
        "agent_contract": {
            "goal": translation_plan["goal"],
            "must_read_before_edit": [
                _artifact_path_for(module_id, "translation-plan.json"),
                _artifact_path_for(module_id, "safety-policy.json"),
                _artifact_path_for(module_id, "soundness-discharge.json"),
            ],
            "must_not_do": [
                "Do not introduce `unsafe` into driver code.",
                "Do not call `bindings::*` directly from driver code.",
                "Do not implement callbacks listed under `deferred_callbacks` without updating the artifacts first.",
                "Do not edit files outside `generation_scope.managed_files` unless a newer artifact explicitly expands the scope.",
            ],
            "soundness_claim_scope": {
                "mvp_callbacks": abstraction_plan["mvp_scope"]["required_callbacks"],
                "deferred_callbacks": translation_plan["deferred_callbacks"],
                "abstraction_allowlist": managed_abstraction_files,
            },
        },
        "acceptance_gates": [
            {
                "id": "verify-safety",
                "stage": "pre-acceptance",
                "mandatory": True,
                "command": f"python3 scripts/c2saferust/tool_cli.py verify-safety --module-path {module_path} --output {_artifact_path_for(module_id, 'safety-verdict.json')}",
                "pass_condition": "`safety-verdict.json.pass == true`",
            },
            {
                "id": "compile-driver-object",
                "stage": "pre-smoke",
                "mandatory": True,
                "command": f"make O=/tmp/c2saferust-{module_id}-build <LLVM/ENV> {driver_object_path}",
                "pass_condition": f"`make` exits 0 for `{driver_object_path}` after safety gate passes.",
            },
        ],
    }
