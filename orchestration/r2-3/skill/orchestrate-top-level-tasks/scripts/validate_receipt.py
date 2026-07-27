#!/usr/bin/env python3
"""Validate structural invariants for a top-level task terminal receipt."""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import Any

from protocol_json import (
    is_absolute_platform_path,
    is_canonical_branch_ref,
    is_canonical_repo_relative_path,
    load_json,
)


OID_RE = re.compile(r"^[0-9a-f]{40}$")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
BLOCKED_STATUSES = {
    "BLOCKED_CONTRADICTION",
    "BLOCKED_SCOPE_EXPANSION",
    "BLOCKED_REVIEW",
    "BLOCKED_NATIVE_EVIDENCE",
    "BLOCKED_PLATFORM_HANDOFF_REQUIRED",
    "BLOCKED_TASK_NOT_TERMINAL",
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


def require_bool(value: Any, name: str, expected: bool | None = None) -> bool:
    if not isinstance(value, bool):
        fail(f"{name} must be a boolean")
    if expected is not None and value is not expected:
        fail(f"{name} must be {str(expected).lower()}")
    return value


def require_integer_zero(value: Any, name: str) -> None:
    if type(value) is not int or value != 0:
        fail(f"{name} must be integer zero")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("receipt", type=Path)
    parser.add_argument(
        "--expected-next",
        help="Require the receipt's next_increment to match this authoritative value.",
    )
    args = parser.parse_args()
    try:
        data = require_object(load_json(args.receipt), "receipt")
    except ValueError as exc:
        fail(f"invalid receipt JSON: {exc}")

    if data.get("protocol") != "codex.top-level-task-receipt.v1":
        fail("unsupported receipt protocol")
    require_string(data.get("orchestration_id"), "orchestration_id")
    nonce = require_string(data.get("dispatch_nonce"), "dispatch_nonce")
    if not re.fullmatch(r"[0-9a-f]{32,}", nonce):
        fail("dispatch_nonce must be at least 32 lowercase hexadecimal characters")
    require_string(data.get("increment"), "increment")
    status = require_string(data.get("status"), "status")
    increment_task = require_object(data.get("increment_task"), "increment_task")
    require_string(increment_task.get("thread_id"), "increment_task.thread_id")
    require_string(increment_task.get("host_id"), "increment_task.host_id")

    base = require_object(data.get("expected_base"), "expected_base")
    require_oid(base.get("commit"), "expected_base.commit")
    require_oid(base.get("tree"), "expected_base.tree")

    if status in BLOCKED_STATUSES:
        if "landed" in data:
            fail("blocked receipt must not contain landed")
        blocker = require_object(data.get("blocker"), "blocker")
        require_string(blocker.get("summary"), "blocker.summary")
        require_string(blocker.get("details"), "blocker.details")
        require_string(blocker.get("required_authority"), "blocker.required_authority")
        if status == "BLOCKED_PLATFORM_HANDOFF_REQUIRED":
            require_string(blocker.get("handoff_prompt"), "blocker.handoff_prompt")
        elif blocker.get("handoff_prompt") is not None:
            require_string(blocker.get("handoff_prompt"), "blocker.handoff_prompt")
        print(f"VALID receipt {data['increment']} status={status}")
        return 0

    if status != "LANDED_CLEAN":
        fail(f"unsupported receipt status: {status}")

    landed = require_object(data.get("landed"), "landed")
    commit = require_oid(landed.get("commit"), "landed.commit")
    require_oid(landed.get("tree"), "landed.tree")
    target_ref = require_string(landed.get("target_ref"), "landed.target_ref")
    if not is_canonical_branch_ref(target_ref):
        fail("landed.target_ref must be a canonical refs/heads/ branch ref")
    if require_oid(landed.get("live_remote"), "landed.live_remote") != commit:
        fail("landed.live_remote must equal landed.commit")

    require_digest(data.get("subject_fingerprint"), "subject_fingerprint")
    changed_files = data.get("changed_files")
    if not isinstance(changed_files, list) or not changed_files:
        fail("changed_files must be a non-empty array")
    if not all(
        isinstance(path, str) and is_canonical_repo_relative_path(path)
        for path in changed_files
    ):
        fail("changed_files entries must be canonical repository-relative POSIX paths")
    if changed_files != sorted(set(changed_files)):
        fail("changed_files must be sorted and unique")

    review = require_object(data.get("review"), "review")
    require_string(review.get("packet_id"), "review.packet_id")
    record_path = require_string(review.get("record_path"), "review.record_path")
    if not is_absolute_platform_path(record_path):
        fail("review.record_path must be an absolute POSIX, drive, or UNC path")
    require_digest(review.get("record_sha256"), "review.record_sha256")
    require_bool(review.get("validated"), "review.validated", True)
    if review.get("terminal") != "CLEAN":
        fail("review.terminal must be CLEAN")
    require_integer_zero(review.get("p1"), "review.p1")
    require_integer_zero(review.get("p2"), "review.p2")
    require_string(review.get("p3_p4_disposition"), "review.p3_p4_disposition")

    verification = require_object(data.get("verification"), "verification")
    for field in (
        "required_checks_passed",
        "allowlist_passed",
        "change_detection_passed",
        "checkout_clean",
    ):
        require_bool(verification.get(field), f"verification.{field}", True)
    require_integer_zero(verification.get("ahead"), "verification.ahead")
    require_integer_zero(verification.get("behind"), "verification.behind")

    next_increment = require_string(data.get("next_increment"), "next_increment")
    if args.expected_next is None:
        fail("--expected-next is required for a LANDED_CLEAN receipt")
    if next_increment != args.expected_next:
        fail("next_increment does not match the authoritative expected successor")
    print(f"VALID receipt {data['increment']} status=LANDED_CLEAN commit={commit}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
