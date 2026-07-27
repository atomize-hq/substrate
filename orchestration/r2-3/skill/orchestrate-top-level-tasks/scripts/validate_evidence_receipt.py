#!/usr/bin/env python3
"""Validate structural invariants for a native/read-only evidence receipt."""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import Any

from protocol_json import (
    is_absolute_platform_path,
    is_canonical_branch_ref,
    load_json,
)


OID_RE = re.compile(r"^[0-9a-f]{40}$")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
BLOCKED_STATUSES = {
    "BLOCKED_NATIVE_EVIDENCE",
    "BLOCKED_PLATFORM_HANDOFF_REQUIRED",
    "BLOCKED_CONTRADICTION",
    "AUTHORITY_REQUIRED",
    "BASE_DRIFT",
}


def fail(message: str) -> None:
    raise SystemExit(message)


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    return value


def require_string(value: Any, name: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{name} must be a non-empty string")
    return value


def require_oid(value: Any, name: str) -> str:
    text = require_string(value, name)
    if not OID_RE.fullmatch(text):
        fail(f"{name} must be 40 lowercase hexadecimal characters")
    return text


def require_digest(value: Any, name: str) -> str:
    text = require_string(value, name)
    if not DIGEST_RE.fullmatch(text):
        fail(f"{name} must be sha256 plus 64 lowercase hexadecimal characters")
    return text


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("receipt", type=Path)
    args = parser.parse_args()
    try:
        data = require_object(load_json(args.receipt), "receipt")
    except ValueError as exc:
        fail(f"invalid evidence receipt JSON: {exc}")

    if data.get("protocol") != "codex.top-level-evidence-receipt.v1":
        fail("unsupported evidence receipt protocol")
    require_string(data.get("orchestration_id"), "orchestration_id")
    nonce = require_string(data.get("dispatch_nonce"), "dispatch_nonce")
    if not re.fullmatch(r"[0-9a-f]{32,}", nonce):
        fail("dispatch_nonce must be at least 32 lowercase hexadecimal characters")
    require_string(data.get("evidence_id"), "evidence_id")
    status = require_string(data.get("status"), "status")
    platform = require_string(data.get("platform"), "platform")
    if platform not in {"linux", "macos", "windows"}:
        fail("platform must be linux, macos, or windows")
    evidence_task = require_object(data.get("evidence_task"), "evidence_task")
    require_string(evidence_task.get("thread_id"), "evidence_task.thread_id")
    require_string(evidence_task.get("host_id"), "evidence_task.host_id")

    source = require_object(data.get("source"), "source")
    commit = require_oid(source.get("commit"), "source.commit")
    require_oid(source.get("tree"), "source.tree")
    target_ref = require_string(source.get("target_ref"), "source.target_ref")
    if not is_canonical_branch_ref(target_ref):
        fail("source.target_ref must be a canonical refs/heads/ branch ref")
    if require_oid(source.get("live_remote"), "source.live_remote") != commit:
        fail("source.live_remote must equal source.commit")

    if status in BLOCKED_STATUSES:
        blocker = require_object(data.get("blocker"), "blocker")
        require_string(blocker.get("summary"), "blocker.summary")
        require_string(blocker.get("details"), "blocker.details")
        require_string(blocker.get("required_authority"), "blocker.required_authority")
        if status == "BLOCKED_PLATFORM_HANDOFF_REQUIRED":
            require_string(blocker.get("handoff_prompt"), "blocker.handoff_prompt")
        elif blocker.get("handoff_prompt") is not None:
            require_string(blocker.get("handoff_prompt"), "blocker.handoff_prompt")
        print(f"VALID evidence receipt {data['evidence_id']} status={status}")
        return 0

    if status != "EVIDENCE_CLEAN":
        fail(f"unsupported evidence receipt status: {status}")

    environment = require_object(data.get("environment"), "environment")
    environment_host_id = require_string(environment.get("host_id"), "environment.host_id")
    if evidence_task["host_id"] != environment_host_id:
        fail("evidence_task.host_id must equal environment.host_id")
    require_string(environment.get("project_id"), "environment.project_id")
    project_path = require_string(environment.get("project_path"), "environment.project_path")
    if not is_absolute_platform_path(project_path):
        fail("environment.project_path must be an absolute POSIX, drive, or UNC path")
    require_string(environment.get("os_version"), "environment.os_version")
    versions = require_object(environment.get("tool_versions"), "environment.tool_versions")
    if not versions or not all(
        isinstance(key, str)
        and key
        and isinstance(value, str)
        and value
        for key, value in versions.items()
    ):
        fail("environment.tool_versions must be a non-empty string map")

    evidence = require_object(data.get("evidence"), "evidence")
    artifact_path = require_string(evidence.get("artifact_path"), "evidence.artifact_path")
    if not is_absolute_platform_path(artifact_path):
        fail("evidence.artifact_path must be an absolute POSIX, drive, or UNC path")
    require_digest(evidence.get("artifact_sha256"), "evidence.artifact_sha256")
    gates = evidence.get("gates")
    if not isinstance(gates, list) or not gates or not all(
        isinstance(gate, str) and gate for gate in gates
    ):
        fail("evidence.gates must be a non-empty array of strings")
    if gates != sorted(set(gates)):
        fail("evidence.gates must be sorted and unique")

    if data.get("prohibited_actions_confirmed") is not True:
        fail("prohibited_actions_confirmed must be true")
    if data.get("checkout_unchanged") is not True:
        fail("checkout_unchanged must be true")

    print(f"VALID evidence receipt {data['evidence_id']} status=EVIDENCE_CLEAN commit={commit}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
