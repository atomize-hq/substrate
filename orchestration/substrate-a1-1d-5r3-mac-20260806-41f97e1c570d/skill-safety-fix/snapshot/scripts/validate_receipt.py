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


def require_optional_oid(value: Any, name: str) -> str | None:
    if value is None:
        return None
    return require_oid(value, name)


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


def require_paths(value: Any, name: str) -> list[str]:
    if not isinstance(value, list) or not value:
        fail(f"{name} must be a non-empty array")
    if not all(
        isinstance(path, str) and is_canonical_repo_relative_path(path)
        for path in value
    ):
        fail(f"{name} entries must be canonical repository-relative POSIX paths")
    if value != sorted(set(value)):
        fail(f"{name} must be sorted and unique")
    return value


def validate_common_success(data: dict[str, Any], status: str) -> None:
    require_digest(data.get("subject_fingerprint"), "subject_fingerprint")

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
    if status == "LANDED_CLEAN":
        require_integer_zero(verification.get("ahead"), "verification.ahead")
        require_integer_zero(verification.get("behind"), "verification.behind")
    else:
        for field in (
            "local_ref_verified",
            "two_commit_topology_verified",
            "no_merge_performed",
            "no_push_performed",
            "remote_unchanged",
        ):
            require_bool(verification.get(field), f"verification.{field}", True)


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
        if "landed" in data or "closed" in data:
            fail("blocked receipt must not contain landed or closed")
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

    if status not in {"LANDED_CLEAN", "CLOSED_CLEAN"}:
        fail(f"unsupported receipt status: {status}")

    commit: str
    if status == "LANDED_CLEAN":
        if "closed" in data or "publication" in data:
            fail("LANDED_CLEAN must not contain closed or publication")
        landed = require_object(data.get("landed"), "landed")
        commit = require_oid(landed.get("commit"), "landed.commit")
        require_oid(landed.get("tree"), "landed.tree")
        target_ref = require_string(landed.get("target_ref"), "landed.target_ref")
        if not is_canonical_branch_ref(target_ref):
            fail("landed.target_ref must be a canonical refs/heads/ branch ref")
        if require_oid(landed.get("live_remote"), "landed.live_remote") != commit:
            fail("landed.live_remote must equal landed.commit")
        require_paths(data.get("changed_files"), "changed_files")
    else:
        if "landed" in data:
            fail("CLOSED_CLEAN must not contain landed")
        closed = require_object(data.get("closed"), "closed")
        worktree = require_string(closed.get("worktree"), "closed.worktree")
        if not is_absolute_platform_path(worktree):
            fail("closed.worktree must be an absolute platform path")
        branch_ref = require_string(closed.get("branch_ref"), "closed.branch_ref")
        if not is_canonical_branch_ref(branch_ref):
            fail("closed.branch_ref must be a canonical refs/heads/ branch ref")
        require_oid(closed.get("work_commit"), "closed.work_commit")
        require_oid(closed.get("work_tree"), "closed.work_tree")
        commit = require_oid(closed.get("control_commit"), "closed.control_commit")
        require_oid(closed.get("control_tree"), "closed.control_tree")
        if require_oid(closed.get("local_ref"), "closed.local_ref") != commit:
            fail("closed.local_ref must equal closed.control_commit")
        work_files = require_paths(
            closed.get("work_changed_files"),
            "closed.work_changed_files",
        )
        control_files = require_paths(
            closed.get("control_changed_files"),
            "closed.control_changed_files",
        )
        changed_files = require_paths(data.get("changed_files"), "changed_files")
        if changed_files != sorted(set(work_files + control_files)):
            fail(
                "changed_files must equal the sorted union of work and control "
                "changed files"
            )

        publication = require_object(data.get("publication"), "publication")
        if publication.get("mode") != "local_only":
            fail("publication.mode must equal local_only for CLOSED_CLEAN")
        require_bool(
            publication.get("merge_performed"),
            "publication.merge_performed",
            False,
        )
        require_bool(
            publication.get("push_performed"),
            "publication.push_performed",
            False,
        )
        require_bool(
            publication.get("remote_mutation"),
            "publication.remote_mutation",
            False,
        )
        observation = require_object(
            publication.get("remote_observation"),
            "publication.remote_observation",
        )
        remote_name = observation.get("name")
        if remote_name is not None:
            require_string(remote_name, "publication.remote_observation.name")
        remote_ref = require_string(
            observation.get("target_ref"),
            "publication.remote_observation.target_ref",
        )
        if not is_canonical_branch_ref(remote_ref):
            fail(
                "publication.remote_observation.target_ref must be a canonical "
                "refs/heads/ branch ref"
            )
        before = require_optional_oid(
            observation.get("before"),
            "publication.remote_observation.before",
        )
        after = require_optional_oid(
            observation.get("after"),
            "publication.remote_observation.after",
        )
        if before != after:
            fail("publication remote before and after identities must match")
        if remote_name is None and (before is not None or after is not None):
            fail("null remote name requires null before and after identities")

    validate_common_success(data, status)

    next_increment = require_string(data.get("next_increment"), "next_increment")
    if args.expected_next is None:
        fail("--expected-next is required for a successful receipt")
    if next_increment != args.expected_next:
        fail("next_increment does not match the authoritative expected successor")
    print(f"VALID receipt {data['increment']} status={status} commit={commit}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
