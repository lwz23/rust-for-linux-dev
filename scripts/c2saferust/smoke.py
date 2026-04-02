#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

import intake


REPO_ROOT = Path(__file__).resolve().parents[2]
LOCAL_LLVM_ROOT = Path("/tmp/llvm-15-local/usr")


def _repo_root(repo_root: str | Path | None) -> Path:
    return Path(repo_root).resolve() if repo_root else REPO_ROOT


def _artifact_output(module_id: str, filename: str, output: str | Path | None = None) -> Path:
    if output is not None:
        return Path(output)
    return Path(intake._artifact_path_for(module_id, filename))


def _derive_link_kind(translation_plan: dict) -> str:
    registration_kind = translation_plan["module_shell"]["registration_kind"]
    match = re.search(r'c_str!\("([^"]+)"\)', registration_kind)
    return match.group(1) if match else translation_plan["module_id"]


def _detect_build_env() -> tuple[dict[str, str], dict]:
    env = dict(os.environ)
    profile = {
        "source": "inherited-environment",
        "make_llvm": None,
        "path_prefix": None,
        "libclang_path": env.get("LIBCLANG_PATH"),
        "ld_library_path_prefix": None,
    }

    clang15 = LOCAL_LLVM_ROOT / "bin" / "clang-15"
    libclang_dir = LOCAL_LLVM_ROOT / "lib" / "x86_64-linux-gnu"
    if clang15.exists():
        env["PATH"] = f"{LOCAL_LLVM_ROOT / 'bin'}:{env.get('PATH', '')}"
        env.setdefault("LIBCLANG_PATH", str(libclang_dir))
        current_ld = env.get("LD_LIBRARY_PATH", "")
        env["LD_LIBRARY_PATH"] = (
            f"{libclang_dir}:{current_ld}" if current_ld else str(libclang_dir)
        )
        profile.update(
            {
                "source": "auto-detected-/tmp/llvm-15-local",
                "make_llvm": "-15",
                "path_prefix": str(LOCAL_LLVM_ROOT / "bin"),
                "libclang_path": env["LIBCLANG_PATH"],
                "ld_library_path_prefix": str(libclang_dir),
            }
        )
    return env, profile


def _is_configured_build_dir(path: Path) -> bool:
    return (path / ".config").exists()


def _resolve_build_dir(repo_root: Path, module_id: str, build_dir: str | Path | None) -> tuple[Path, str]:
    if build_dir is not None:
        return Path(build_dir), "user-specified"
    candidates = [
        (Path(f"/tmp/{module_id}-build"), "auto-detected-module-build-dir"),
        (Path(f"/tmp/c2saferust-{module_id}-build"), "default-c2saferust-build-dir"),
        (repo_root, "in-tree-build"),
    ]
    for candidate, source in candidates:
        if _is_configured_build_dir(candidate):
            return candidate, source
    return Path(f"/tmp/c2saferust-{module_id}-build"), "default-c2saferust-build-dir"


def _ensure_kernel_image(
    repo_root: Path,
    build_dir: Path,
    *,
    make_llvm: str | None = None,
) -> tuple[Path, dict]:
    image = build_dir / "arch" / "x86" / "boot" / "bzImage"
    env, env_profile = _detect_build_env()
    llvm_arg = make_llvm if make_llvm is not None else env_profile["make_llvm"]

    if image.exists():
        return image, {
            "status": "already_built",
            "image": str(image),
            "env_profile": env_profile,
            "command": None,
            "exit_code": 0,
        }

    command = ["make", f"O={build_dir}"]
    if llvm_arg is not None:
        command.append(f"LLVM={llvm_arg}")
    command.extend([f"-j{os.cpu_count() or 1}", "bzImage"])

    completed = subprocess.run(
        command,
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    result = {
        "status": "built" if completed.returncode == 0 else "failed",
        "image": str(image),
        "env_profile": env_profile,
        "command": " ".join(command),
        "exit_code": completed.returncode,
        "stdout_tail": completed.stdout.splitlines()[-60:],
        "stderr_tail": completed.stderr.splitlines()[-60:],
    }
    if completed.returncode != 0 or not image.exists():
        raise RuntimeError("Failed to build `bzImage` for smoke run.")
    return image, result


def _write_init_script(root: Path, link_kind: str, device_name: str) -> None:
    init_text = f"""#!/bin/sh
export PATH=/bin:/sbin
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mkdir -p /dev/pts /tmp /run
mknod /dev/console c 5 1 2>/dev/null || true
mknod /dev/null c 1 3 2>/dev/null || true
echo "[smoke] begin link smoke"
echo "[smoke] link_kind={link_kind}"
echo "[smoke] device_name={device_name}"
ip link show >/tmp/ip-link-before.txt 2>&1
ip link add {device_name} type {link_kind} >/tmp/ip-link-add.txt 2>&1
add_rc=$?
echo "[smoke] add_rc=$add_rc"
if [ $add_rc -eq 0 ]; then
    ip link set {device_name} up >/tmp/ip-link-up.txt 2>&1
    up_rc=$?
    echo "[smoke] up_rc=$up_rc"
    ip link show {device_name} >/tmp/ip-link-show.txt 2>&1
    show_rc=$?
    echo "[smoke] show_rc=$show_rc"
    ip link set {device_name} down >/tmp/ip-link-down.txt 2>&1
    down_rc=$?
    echo "[smoke] down_rc=$down_rc"
    ip link del {device_name} >/tmp/ip-link-del.txt 2>&1
    del_rc=$?
    echo "[smoke] del_rc=$del_rc"
else
    up_rc=125
    show_rc=125
    down_rc=125
    del_rc=125
fi
echo "[smoke] before:"
cat /tmp/ip-link-before.txt
echo "[smoke] add:"
cat /tmp/ip-link-add.txt
echo "[smoke] show:"
[ -f /tmp/ip-link-show.txt ] && cat /tmp/ip-link-show.txt || true
echo "[smoke] dmesg-tail:"
dmesg | tail -n 120
if [ $add_rc -eq 0 ] && [ $up_rc -eq 0 ] && [ $show_rc -eq 0 ] && [ $down_rc -eq 0 ] && [ $del_rc -eq 0 ]; then
    echo "[smoke] RESULT=PASS"
else
    echo "[smoke] RESULT=FAIL"
fi
poweroff -f
"""
    (root / "init").write_text(init_text)
    (root / "init").chmod(0o755)


def _build_initramfs(link_kind: str, device_name: str) -> tuple[Path, dict]:
    busybox = shutil.which("busybox")
    cpio = shutil.which("cpio")
    if busybox is None or cpio is None:
        raise RuntimeError("`busybox` and `cpio` are required for QEMU smoke.")

    root = Path(tempfile.mkdtemp(prefix="c2saferust-smoke-rootfs-"))
    for relative in ["bin", "sbin", "proc", "sys", "dev", "etc", "tmp", "run"]:
        (root / relative).mkdir(parents=True, exist_ok=True)

    shutil.copy2(busybox, root / "bin" / "busybox")
    for app in [
        "sh",
        "mount",
        "mkdir",
        "mknod",
        "cat",
        "echo",
        "uname",
        "ip",
        "poweroff",
        "reboot",
        "sleep",
        "dmesg",
        "ls",
        "grep",
        "sed",
        "awk",
        "ifconfig",
        "tail",
    ]:
        (root / "bin" / app).symlink_to("/bin/busybox")
    (root / "sbin" / "modprobe").symlink_to("/bin/busybox")
    (root / "sbin" / "insmod").symlink_to("/bin/busybox")

    _write_init_script(root, link_kind, device_name)

    initramfs_path = Path(tempfile.mkstemp(prefix="c2saferust-smoke-", suffix=".cpio")[1])
    completed = subprocess.run(
        f"cd {root} && find . -print0 | cpio --null -o --format=newc > {initramfs_path}",
        shell=True,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError("Failed to build smoke initramfs.")

    return initramfs_path, {
        "rootfs_dir": str(root),
        "initramfs_path": str(initramfs_path),
        "cpio_stdout_tail": completed.stdout.splitlines()[-20:],
        "cpio_stderr_tail": completed.stderr.splitlines()[-20:],
    }


def _run_qemu(kernel_image: Path, initramfs: Path, qemu_log: Path, *, timeout_sec: int) -> dict:
    qemu = shutil.which("qemu-system-x86_64")
    if qemu is None:
        raise RuntimeError("`qemu-system-x86_64` is required for smoke.")

    command = [
        qemu,
        "-nodefaults",
        "-no-reboot",
        "-nographic",
        "-machine",
        "q35,accel=tcg",
        "-m",
        "1024",
        "-smp",
        "2",
        "-kernel",
        str(kernel_image),
        "-initrd",
        str(initramfs),
        "-append",
        "console=ttyS0 rdinit=/init nokaslr panic=-1",
        "-serial",
        "stdio",
    ]

    completed = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        timeout=timeout_sec,
    )
    qemu_log.write_text(completed.stdout + ("\n[stderr]\n" + completed.stderr if completed.stderr else ""))
    return {
        "command": command,
        "exit_code": completed.returncode,
        "stdout_tail": completed.stdout.splitlines()[-120:],
        "stderr_tail": completed.stderr.splitlines()[-80:],
    }


def _extract_smoke_summary(qemu_log_text: str) -> dict:
    def find_rc(name: str) -> int | None:
        match = re.search(rf"\[smoke\] {re.escape(name)}=(-?\d+)", qemu_log_text)
        return int(match.group(1)) if match else None

    return {
        "result": "PASS" if "[smoke] RESULT=PASS" in qemu_log_text else "FAIL",
        "add_rc": find_rc("add_rc"),
        "up_rc": find_rc("up_rc"),
        "show_rc": find_rc("show_rc"),
        "down_rc": find_rc("down_rc"),
        "del_rc": find_rc("del_rc"),
    }


def run_qemu_link_smoke(
    module_path: str | Path,
    *,
    repo_root: str | Path | None = None,
    output: str | Path | None = None,
    qemu_log_output: str | Path | None = None,
    build_dir: str | Path | None = None,
    make_llvm: str | None = None,
    timeout_sec: int = 120,
) -> dict:
    repo = _repo_root(repo_root)
    translation_plan = intake.build_translation_plan(module_path, repo_root=repo)
    module_id = translation_plan["module_id"]
    link_kind = _derive_link_kind(translation_plan)
    device_name = f"{module_id}0"

    resolved_build_dir, build_dir_source = _resolve_build_dir(repo, module_id, build_dir)
    kernel_image, build_result = _ensure_kernel_image(
        repo,
        resolved_build_dir,
        make_llvm=make_llvm,
    )
    initramfs, initramfs_result = _build_initramfs(link_kind, device_name)

    qemu_log_path = (
        Path(qemu_log_output)
        if qemu_log_output is not None
        else _artifact_output(module_id, "qemu-smoke.log")
    )
    qemu_log_path.parent.mkdir(parents=True, exist_ok=True)
    qemu_result = _run_qemu(kernel_image, initramfs, qemu_log_path, timeout_sec=timeout_sec)
    qemu_text = qemu_log_path.read_text()
    smoke_summary = _extract_smoke_summary(qemu_text)

    payload = {
        "schema_version": 1,
        "artifact_type": "toolchain-run-record",
        "module_id": module_id,
        "module_c_path": translation_plan["module_c_path"],
        "scenario": "qemu-ip-link-smoke",
        "pass": smoke_summary["result"] == "PASS",
        "ready_for_oracle": smoke_summary["result"] == "PASS",
        "link_kind": link_kind,
        "device_name": device_name,
        "build": {
            "build_dir": str(resolved_build_dir),
            "build_dir_source": build_dir_source,
            "kernel_image": str(kernel_image),
            "kernel_build": build_result,
        },
        "initramfs": initramfs_result,
        "qemu": {
            "log_path": str(qemu_log_path),
            **qemu_result,
        },
        "smoke_summary": smoke_summary,
        "steps": [
            {
                "step_id": "build-kernel-image",
                "status": "pass",
                "details": [str(kernel_image)],
            },
            {
                "step_id": "build-initramfs",
                "status": "pass",
                "details": [str(initramfs)],
            },
            {
                "step_id": "run-qemu-link-smoke",
                "status": "pass" if smoke_summary["result"] == "PASS" else "fail",
                "details": [f"RESULT={smoke_summary['result']}"],
            },
        ],
    }

    output_path = _artifact_output(module_id, "toolchain-run-record.json", output)
    intake.write_json(output_path, payload)
    payload["output_path"] = str(output_path)
    return payload
