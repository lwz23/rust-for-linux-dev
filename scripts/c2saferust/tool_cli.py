#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0

from __future__ import annotations

import argparse
from pathlib import Path

import intake
import safety
import smoke
import workflow


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="C2SafeRust tool bootstrap CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    intake_kbuild = subparsers.add_parser("intake-kbuild")
    intake_kbuild.add_argument("--module-path", default="drivers/net/nlmon.c")
    intake_kbuild.add_argument("--output", required=True)
    intake_kbuild.add_argument("--repo-root")

    audit_bindings = subparsers.add_parser("audit-bindings")
    audit_bindings.add_argument("--module-path", default="drivers/net/nlmon.c")
    audit_bindings.add_argument("--binding-output", required=True)
    audit_bindings.add_argument("--helper-output", required=True)
    audit_bindings.add_argument("--repo-root")

    plan_patches = subparsers.add_parser("plan-patches")
    plan_patches.add_argument("--module-path", default="drivers/net/nlmon.c")
    plan_patches.add_argument("--kbuild-output", required=True)
    plan_patches.add_argument("--bindings-output", required=True)
    plan_patches.add_argument("--helpers-output", required=True)
    plan_patches.add_argument("--repo-root")

    plan_abstractions = subparsers.add_parser("plan-abstractions")
    plan_abstractions.add_argument("--module-path", default="drivers/net/nlmon.c")
    plan_abstractions.add_argument("--abstraction-output", required=True)
    plan_abstractions.add_argument("--unsafe-output", required=True)
    plan_abstractions.add_argument("--repo-root")

    plan_translation = subparsers.add_parser("plan-translation")
    plan_translation.add_argument("--module-path", default="drivers/net/nlmon.c")
    plan_translation.add_argument("--translation-output", required=True)
    plan_translation.add_argument("--repo-root")

    plan_safety = subparsers.add_parser("plan-safety-policy")
    plan_safety.add_argument("--module-path", default="drivers/net/nlmon.c")
    plan_safety.add_argument("--policy-output", required=True)
    plan_safety.add_argument("--discharge-output", required=True)
    plan_safety.add_argument("--repo-root")

    verify_safety = subparsers.add_parser("verify-safety")
    verify_safety.add_argument("--module-path", default="drivers/net/nlmon.c")
    verify_safety.add_argument("--output", required=True)
    verify_safety.add_argument("--repo-root")

    plan_agent_workflow = subparsers.add_parser("plan-agent-workflow")
    plan_agent_workflow.add_argument("--module-path", default="drivers/net/nlmon.c")
    plan_agent_workflow.add_argument("--output", required=True)
    plan_agent_workflow.add_argument("--repo-root")

    gate_agent_candidate = subparsers.add_parser("gate-agent-candidate")
    gate_agent_candidate.add_argument("--module-path", default="drivers/net/nlmon.c")
    gate_agent_candidate.add_argument("--output", required=True)
    gate_agent_candidate.add_argument("--safety-output")
    gate_agent_candidate.add_argument("--repo-root")
    gate_agent_candidate.add_argument("--build-dir")
    gate_agent_candidate.add_argument("--make-llvm")
    gate_agent_candidate.add_argument("--skip-compile", action="store_true")

    run_smoke_qemu = subparsers.add_parser("run-smoke-qemu")
    run_smoke_qemu.add_argument("--module-path", default="drivers/net/nlmon.c")
    run_smoke_qemu.add_argument("--output", required=True)
    run_smoke_qemu.add_argument("--qemu-log-output")
    run_smoke_qemu.add_argument("--repo-root")
    run_smoke_qemu.add_argument("--build-dir")
    run_smoke_qemu.add_argument("--make-llvm")
    run_smoke_qemu.add_argument("--timeout-sec", type=int, default=120)

    bootstrap = subparsers.add_parser("bootstrap-nlmon")
    bootstrap.add_argument("--module-path", default="drivers/net/nlmon.c")
    bootstrap.add_argument("--output-dir", required=True)
    bootstrap.add_argument("--repo-root")

    return parser.parse_args()


def run_intake_kbuild(args: argparse.Namespace) -> str:
    payload = intake.build_kbuild_plan(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.output, payload)
    return intake.stable_json(payload)


def run_audit_bindings(args: argparse.Namespace) -> str:
    binding_payload = intake.build_binding_gap_audit(args.module_path, repo_root=args.repo_root)
    helper_payload = intake.build_helper_audit(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.binding_output, binding_payload)
    intake.write_json(args.helper_output, helper_payload)
    return intake.stable_json(binding_payload)


def run_plan_patches(args: argparse.Namespace) -> str:
    kbuild_payload = intake.build_kbuild_patch_plan(args.module_path, repo_root=args.repo_root)
    bindings_payload = intake.build_bindings_patch_plan(args.module_path, repo_root=args.repo_root)
    helpers_payload = intake.build_helpers_patch_plan(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.kbuild_output, kbuild_payload)
    intake.write_json(args.bindings_output, bindings_payload)
    intake.write_json(args.helpers_output, helpers_payload)
    return intake.stable_json(kbuild_payload)


def run_plan_abstractions(args: argparse.Namespace) -> str:
    abstraction_payload = intake.build_abstraction_plan(args.module_path, repo_root=args.repo_root)
    unsafe_payload = intake.build_unsafe_obligations(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.abstraction_output, abstraction_payload)
    intake.write_json(args.unsafe_output, unsafe_payload)
    return intake.stable_json(abstraction_payload)


def run_plan_translation(args: argparse.Namespace) -> str:
    payload = intake.build_translation_plan(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.translation_output, payload)
    return intake.stable_json(payload)


def run_plan_safety_policy(args: argparse.Namespace) -> str:
    policy_payload = intake.build_safety_policy(args.module_path, repo_root=args.repo_root)
    discharge_payload = intake.build_soundness_discharge(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.policy_output, policy_payload)
    intake.write_json(args.discharge_output, discharge_payload)
    return intake.stable_json(policy_payload)


def run_verify_safety(args: argparse.Namespace) -> str:
    payload = safety.verify_module_safety(args.module_path, repo_root=args.repo_root)
    safety.write_verdict(args.output, payload)
    return intake.stable_json(payload)


def run_plan_agent_workflow(args: argparse.Namespace) -> str:
    payload = intake.build_agent_workflow_plan(args.module_path, repo_root=args.repo_root)
    intake.write_json(args.output, payload)
    return intake.stable_json(payload)


def run_gate_agent_candidate(args: argparse.Namespace) -> str:
    payload = workflow.gate_agent_candidate(
        args.module_path,
        repo_root=args.repo_root,
        output=args.output,
        safety_output=args.safety_output,
        build_dir=args.build_dir,
        make_llvm=args.make_llvm,
        skip_compile=args.skip_compile,
    )
    return intake.stable_json(payload)


def run_smoke_qemu(args: argparse.Namespace) -> str:
    payload = smoke.run_qemu_link_smoke(
        args.module_path,
        repo_root=args.repo_root,
        output=args.output,
        qemu_log_output=args.qemu_log_output,
        build_dir=args.build_dir,
        make_llvm=args.make_llvm,
        timeout_sec=args.timeout_sec,
    )
    return intake.stable_json(payload)


def run_bootstrap_nlmon(args: argparse.Namespace) -> str:
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    artifacts = {
        "kbuild-plan.json": intake.build_kbuild_plan(args.module_path, repo_root=args.repo_root),
        "binding-gap-audit.json": intake.build_binding_gap_audit(args.module_path, repo_root=args.repo_root),
        "helper-audit.json": intake.build_helper_audit(args.module_path, repo_root=args.repo_root),
        "kbuild-patch-plan.json": intake.build_kbuild_patch_plan(args.module_path, repo_root=args.repo_root),
        "bindings-patch-plan.json": intake.build_bindings_patch_plan(args.module_path, repo_root=args.repo_root),
        "helpers-patch-plan.json": intake.build_helpers_patch_plan(args.module_path, repo_root=args.repo_root),
        "abstraction-plan.json": intake.build_abstraction_plan(args.module_path, repo_root=args.repo_root),
        "unsafe-obligations.json": intake.build_unsafe_obligations(args.module_path, repo_root=args.repo_root),
        "translation-plan.json": intake.build_translation_plan(args.module_path, repo_root=args.repo_root),
        "safety-policy.json": intake.build_safety_policy(args.module_path, repo_root=args.repo_root),
        "soundness-discharge.json": intake.build_soundness_discharge(args.module_path, repo_root=args.repo_root),
        "agent-workflow-plan.json": intake.build_agent_workflow_plan(args.module_path, repo_root=args.repo_root),
    }

    for filename, payload in artifacts.items():
        intake.write_json(output_dir / filename, payload)

    manifest = {
        "schema_version": 1,
        "artifact_type": "bootstrap-manifest",
        "module_id": Path(args.module_path).stem,
        "generated_artifacts": sorted(str(output_dir / filename) for filename in artifacts),
    }
    return intake.stable_json(manifest)


def main() -> int:
    args = parse_args()
    if args.command == "intake-kbuild":
        rendered = run_intake_kbuild(args)
    elif args.command == "audit-bindings":
        rendered = run_audit_bindings(args)
    elif args.command == "plan-patches":
        rendered = run_plan_patches(args)
    elif args.command == "plan-abstractions":
        rendered = run_plan_abstractions(args)
    elif args.command == "plan-translation":
        rendered = run_plan_translation(args)
    elif args.command == "plan-safety-policy":
        rendered = run_plan_safety_policy(args)
    elif args.command == "verify-safety":
        rendered = run_verify_safety(args)
    elif args.command == "plan-agent-workflow":
        rendered = run_plan_agent_workflow(args)
    elif args.command == "gate-agent-candidate":
        rendered = run_gate_agent_candidate(args)
    elif args.command == "run-smoke-qemu":
        rendered = run_smoke_qemu(args)
    else:
        rendered = run_bootstrap_nlmon(args)

    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
