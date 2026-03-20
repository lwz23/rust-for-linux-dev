#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import pathlib
import sys

PREFIX = "ax88796b-observation|"
REQUIRED_FIELDS = [
    "scenario",
    "phy_id",
    "ret",
    "speed",
    "duplex",
    "link",
    "autoneg",
    "autoneg_complete",
    "suspended",
    "pause",
    "asym_pause",
    "bmcr_final",
    "reads",
    "writes",
    "trace",
]


def parse_observation_line(line: str) -> dict:
    line = line.strip()
    if not line.startswith(PREFIX):
        raise ValueError("missing observation prefix")

    fields = {}
    for item in line[len(PREFIX):].split("|"):
        if "=" not in item:
            raise ValueError(f"malformed observation field: {item!r}")
        key, value = item.split("=", 1)
        fields[key] = value

    missing = [field for field in REQUIRED_FIELDS if field not in fields]
    if missing:
        raise ValueError(f"missing required fields: {', '.join(missing)}")

    for field in REQUIRED_FIELDS:
        if field in {"scenario", "phy_id", "bmcr_final", "trace"}:
            continue
        fields[field] = int(fields[field], 10)

    return fields


def normalize_log(mode: str, log_path: pathlib.Path, output_path: pathlib.Path) -> int:
    observations = []
    for raw_line in log_path.read_text(encoding="utf-8").splitlines():
        if PREFIX in raw_line:
            observations.append(parse_observation_line(raw_line[raw_line.index(PREFIX):]))

    observations.sort(key=lambda item: item["scenario"])

    if not observations:
        raise ValueError(f"no observations found in {log_path}")

    payload = {
        "schema_version": 1,
        "module_id": "ax88796b",
        "mode": mode,
        "generated_on": dt.date.today().isoformat(),
        "source_log": str(log_path),
        "observation_count": len(observations),
        "observations": observations,
    }
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


def load_runtime_file(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def compare_runs(c_path: pathlib.Path, rust_path: pathlib.Path, output_path: pathlib.Path) -> int:
    c_payload = load_runtime_file(c_path)
    rust_payload = load_runtime_file(rust_path)

    c_map = {item["scenario"]: item for item in c_payload["observations"]}
    rust_map = {item["scenario"]: item for item in rust_payload["observations"]}

    differences = []
    for scenario in sorted(set(c_map) | set(rust_map)):
        if scenario not in c_map:
            differences.append({"scenario": scenario, "field": "scenario", "c": None, "rust": "missing"})
            continue
        if scenario not in rust_map:
            differences.append({"scenario": scenario, "field": "scenario", "c": "missing", "rust": None})
            continue

        for field in REQUIRED_FIELDS:
            if c_map[scenario][field] != rust_map[scenario][field]:
                differences.append(
                    {
                        "scenario": scenario,
                        "field": field,
                        "c": c_map[scenario][field],
                        "rust": rust_map[scenario][field],
                    }
                )

    payload = {
        "schema_version": 1,
        "module_id": "ax88796b",
        "generated_on": dt.date.today().isoformat(),
        "c_runtime": str(c_path),
        "rust_runtime": str(rust_path),
        "status": "match" if not differences else "mismatch",
        "scenario_count": len(c_map),
        "differences": differences,
    }
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0 if not differences else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="cmd", required=True)

    normalize = subparsers.add_parser("normalize")
    normalize.add_argument("mode", choices=("c", "rust"))
    normalize.add_argument("log")
    normalize.add_argument("output")

    compare = subparsers.add_parser("compare")
    compare.add_argument("c_runtime")
    compare.add_argument("rust_runtime")
    compare.add_argument("output")

    args = parser.parse_args()

    try:
        if args.cmd == "normalize":
            return normalize_log(args.mode, pathlib.Path(args.log), pathlib.Path(args.output))
        return compare_runs(
            pathlib.Path(args.c_runtime),
            pathlib.Path(args.rust_runtime),
            pathlib.Path(args.output),
        )
    except Exception as exc:
        print(f"ax88796b-runtime-diff.py: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
