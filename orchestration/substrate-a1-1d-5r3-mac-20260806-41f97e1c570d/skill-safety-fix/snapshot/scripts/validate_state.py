#!/usr/bin/env python3
"""Validate structural invariants for a top-level orchestration state file."""

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
STATE_STATUSES = {
    "INITIALIZING",
    "READY",
    "DISPATCHING",
    "RUNNING",
    "RECEIPT_RECEIVED",
    "VERIFYING",
    "EVIDENCE_RUNNING",
    "EVIDENCE_RECEIPTS_RECEIVED",
    "EVIDENCE_VERIFYING",
    "BLOCKED",
    "COMPLETE",
}
PLATFORMS = {"linux", "macos", "windows"}
PUBLICATION_MODES = {"remote", "local"}


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


def validate_evidence_source(
    value: Any,
    name: str,
    target_ref: str,
) -> dict[str, Any]:
    source = require_object(value, name)
    commit = require_oid(source.get("commit"), f"{name}.commit")
    require_oid(source.get("tree"), f"{name}.tree")
    if source.get("target_ref") != target_ref:
        fail(f"{name}.target_ref must match state target_ref")
    mode = source.get("publication_mode", "remote")
    if mode == "remote":
        live_remote = source.get("live_remote")
        if live_remote is not None and require_oid(
            live_remote,
            f"{name}.live_remote",
        ) != commit:
            fail(f"{name}.live_remote must equal commit")
    elif mode == "local":
        if require_oid(source.get("local_ref"), f"{name}.local_ref") != commit:
            fail(f"{name}.local_ref must equal commit")
        worktree = require_string(source.get("worktree"), f"{name}.worktree")
        if not is_absolute_platform_path(worktree):
            fail(f"{name}.worktree must be absolute")
    else:
        fail(f"{name}.publication_mode must be remote or local")
    return source


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("state", type=Path)
    args = parser.parse_args()
    try:
        data = require_object(load_json(args.state), "state")
    except ValueError as exc:
        fail(f"invalid state JSON: {exc}")

    if data.get("protocol") != "codex.top-level-orchestration-state.v1":
        fail("unsupported state protocol")
    require_string(data.get("orchestration_id"), "orchestration_id")
    status = require_string(data.get("status"), "status")
    if status not in STATE_STATUSES:
        fail(f"unsupported status: {status}")
    dispatch_authorized = data.get("dispatch_authorized")
    if not isinstance(dispatch_authorized, bool):
        fail("dispatch_authorized must be a boolean")
    if status == "INITIALIZING" and dispatch_authorized:
        fail("INITIALIZING must not authorize dispatch")
    if status in {
        "DISPATCHING",
        "RUNNING",
        "RECEIPT_RECEIVED",
        "VERIFYING",
        "EVIDENCE_RUNNING",
        "EVIDENCE_RECEIPTS_RECEIVED",
        "EVIDENCE_VERIFYING",
    } and not dispatch_authorized:
        fail(f"{status} requires dispatch_authorized=true")
    publication_mode = data.get("publication_mode", "remote")
    if publication_mode not in PUBLICATION_MODES:
        fail("publication_mode must be remote or local")
    remote = data.get("remote")
    if publication_mode == "remote":
        require_string(remote, "remote")
    elif remote is not None:
        require_string(remote, "remote")
    target_ref = require_string(data.get("target_ref"), "target_ref")
    if not is_canonical_branch_ref(target_ref):
        fail("target_ref must be a canonical refs/heads/ branch ref")
    remote_observation = data.get("remote_observation")
    if publication_mode == "remote":
        if remote_observation is not None:
            fail("remote publication must use null remote_observation")
    else:
        observation = require_object(remote_observation, "remote_observation")
        observed_name = observation.get("name")
        if observed_name is not None:
            require_string(observed_name, "remote_observation.name")
        observed_ref = require_string(
            observation.get("target_ref"),
            "remote_observation.target_ref",
        )
        if not is_canonical_branch_ref(observed_ref):
            fail("remote_observation.target_ref must be a canonical branch ref")
        if observed_ref != target_ref:
            fail("remote_observation.target_ref must match state target_ref")
        observed_before = require_optional_oid(
            observation.get("before"),
            "remote_observation.before",
        )
        if remote is None:
            if observed_name is not None or observed_before is not None:
                fail("local state without a remote requires null observation name and identity")
        elif observed_name != remote:
            fail("remote_observation.name must match state remote")
    require_oid(data.get("required_ancestor"), "required_ancestor")

    sequence = data.get("sequence")
    if not isinstance(sequence, list) or not sequence or not all(
        isinstance(item, str) and item for item in sequence
    ):
        fail("sequence must be a non-empty array of strings")
    if len(sequence) != len(set(sequence)):
        fail("sequence entries must be unique")

    required_evidence = data.get("required_evidence")
    if not isinstance(required_evidence, list):
        fail("required_evidence must be an array")
    evidence_requirements: dict[str, dict[str, str]] = {}
    for index, entry in enumerate(required_evidence):
        obj = require_object(entry, f"required_evidence[{index}]")
        evidence_id = require_string(
            obj.get("evidence_id"),
            f"required_evidence[{index}].evidence_id",
        )
        if evidence_id in evidence_requirements:
            fail("required evidence IDs must be unique")
        platform = require_string(
            obj.get("platform"),
            f"required_evidence[{index}].platform",
        )
        if platform not in PLATFORMS:
            fail(f"required_evidence[{index}].platform is unsupported")
        after_increment = require_string(
            obj.get("after_increment"),
            f"required_evidence[{index}].after_increment",
        )
        before_increment = require_string(
            obj.get("before_increment"),
            f"required_evidence[{index}].before_increment",
        )
        if after_increment not in sequence or before_increment not in sequence:
            fail(f"required_evidence[{index}] increments must exist in sequence")
        if sequence.index(after_increment) + 1 != sequence.index(before_increment):
            fail(
                f"required_evidence[{index}] must gate the increment immediately "
                "after its source checkpoint"
            )
        evidence_requirements[evidence_id] = {
            "platform": platform,
            "after_increment": after_increment,
            "before_increment": before_increment,
        }

    cursor = data.get("cursor")
    if not isinstance(cursor, int) or isinstance(cursor, bool):
        fail("cursor must be an integer")
    if cursor < 0 or cursor > len(sequence):
        fail("cursor is outside sequence bounds")

    base = require_object(data.get("expected_base"), "expected_base")
    require_oid(base.get("commit"), "expected_base.commit")
    require_oid(base.get("tree"), "expected_base.tree")

    meta = require_object(data.get("meta"), "meta")
    for field in ("thread_id", "host_id"):
        if meta.get(field) is not None and not isinstance(meta[field], str):
            fail(f"meta.{field} must be null or a string")
    if status != "INITIALIZING":
        require_string(meta.get("thread_id"), "meta.thread_id")
        require_string(meta.get("host_id"), "meta.host_id")

    active = data.get("active_dispatch")
    if status in {"DISPATCHING", "RUNNING", "RECEIPT_RECEIVED", "VERIFYING"}:
        active_obj = require_object(active, "active_dispatch")
        active_increment = require_string(
            active_obj.get("increment"),
            "active_dispatch.increment",
        )
        if cursor >= len(sequence) or active_increment != sequence[cursor]:
            fail("active_dispatch.increment must equal sequence[cursor]")
        require_string(active_obj.get("thread_id"), "active_dispatch.thread_id")
        require_string(active_obj.get("host_id"), "active_dispatch.host_id")
        nonce = require_string(active_obj.get("dispatch_nonce"), "active_dispatch.dispatch_nonce")
        if not re.fullmatch(r"[0-9a-f]{32,}", nonce):
            fail("active_dispatch.dispatch_nonce must be at least 32 lowercase hex characters")
        active_base = require_object(active_obj.get("expected_base"), "active_dispatch.expected_base")
        if require_oid(active_base.get("commit"), "active_dispatch.expected_base.commit") != base["commit"]:
            fail("active_dispatch expected commit must match state expected_base")
        if require_oid(active_base.get("tree"), "active_dispatch.expected_base.tree") != base["tree"]:
            fail("active_dispatch expected tree must match state expected_base")
        if active_obj.get("target_ref") != target_ref:
            fail("active_dispatch.target_ref must match state target_ref")
        active_mode = active_obj.get("publication_mode", publication_mode)
        if active_mode != publication_mode:
            fail("active_dispatch.publication_mode must match state publication_mode")
    elif active is not None:
        fail(f"active_dispatch must be null while status is {status}")

    evidence_dispatches = data.get("active_evidence_dispatches")
    if not isinstance(evidence_dispatches, list):
        fail("active_evidence_dispatches must be an array")
    evidence_ids: list[str] = []
    evidence_nonces: list[str] = []
    active_evidence_objects: list[dict[str, Any]] = []
    for index, entry in enumerate(evidence_dispatches):
        obj = require_object(entry, f"active_evidence_dispatches[{index}]")
        active_evidence_objects.append(obj)
        evidence_id = require_string(
            obj.get("evidence_id"),
            f"active_evidence_dispatches[{index}].evidence_id",
        )
        evidence_ids.append(evidence_id)
        if evidence_id not in evidence_requirements:
            fail(f"active_evidence_dispatches[{index}] is not required")
        platform = require_string(
            obj.get("platform"),
            f"active_evidence_dispatches[{index}].platform",
        )
        if platform not in PLATFORMS:
            fail(f"active_evidence_dispatches[{index}].platform is unsupported")
        if platform != evidence_requirements[evidence_id]["platform"]:
            fail(f"active_evidence_dispatches[{index}].platform mismatches requirement")
        require_string(
            obj.get("project_id"),
            f"active_evidence_dispatches[{index}].project_id",
        )
        project_path = require_string(
            obj.get("project_path"),
            f"active_evidence_dispatches[{index}].project_path",
        )
        if not is_absolute_platform_path(project_path):
            fail(f"active_evidence_dispatches[{index}].project_path must be absolute")
        require_string(obj.get("thread_id"), f"active_evidence_dispatches[{index}].thread_id")
        require_string(obj.get("host_id"), f"active_evidence_dispatches[{index}].host_id")
        nonce = require_string(
            obj.get("dispatch_nonce"),
            f"active_evidence_dispatches[{index}].dispatch_nonce",
        )
        if not re.fullmatch(r"[0-9a-f]{32,}", nonce):
            fail(
                f"active_evidence_dispatches[{index}].dispatch_nonce must be at least "
                "32 lowercase hex characters"
            )
        evidence_nonces.append(nonce)
        source = validate_evidence_source(
            obj.get("source"),
            f"active_evidence_dispatches[{index}].source",
            target_ref,
        )
    if len(evidence_ids) != len(set(evidence_ids)):
        fail("active evidence IDs must be unique")
    if len(evidence_nonces) != len(set(evidence_nonces)):
        fail("active evidence nonces must be unique")
    if status in {"EVIDENCE_RUNNING", "EVIDENCE_RECEIPTS_RECEIVED", "EVIDENCE_VERIFYING"}:
        if not evidence_dispatches:
            fail(f"{status} requires active_evidence_dispatches")
    elif evidence_dispatches:
        fail(f"active_evidence_dispatches must be empty while status is {status}")

    verified_evidence = data.get("verified_evidence")
    if not isinstance(verified_evidence, list):
        fail("verified_evidence must be an array")
    verified_ids: list[str] = []
    verified_evidence_objects: list[dict[str, Any]] = []
    for index, entry in enumerate(verified_evidence):
        obj = require_object(entry, f"verified_evidence[{index}]")
        verified_evidence_objects.append(obj)
        evidence_id = require_string(
            obj.get("evidence_id"),
            f"verified_evidence[{index}].evidence_id",
        )
        verified_ids.append(evidence_id)
        if evidence_id not in evidence_requirements:
            fail(f"verified_evidence[{index}] is not required")
        platform = require_string(
            obj.get("platform"),
            f"verified_evidence[{index}].platform",
        )
        if platform not in PLATFORMS:
            fail(f"verified_evidence[{index}].platform is unsupported")
        if platform != evidence_requirements[evidence_id]["platform"]:
            fail(f"verified_evidence[{index}].platform mismatches requirement")
        require_string(obj.get("project_id"), f"verified_evidence[{index}].project_id")
        project_path = require_string(
            obj.get("project_path"),
            f"verified_evidence[{index}].project_path",
        )
        if not is_absolute_platform_path(project_path):
            fail(f"verified_evidence[{index}].project_path must be absolute")
        receipt_path = require_string(
            obj.get("receipt_path"),
            f"verified_evidence[{index}].receipt_path",
        )
        if not is_canonical_repo_relative_path(receipt_path):
            fail(f"verified_evidence[{index}].receipt_path must be repository-relative")
        require_string(obj.get("thread_id"), f"verified_evidence[{index}].thread_id")
        require_string(obj.get("host_id"), f"verified_evidence[{index}].host_id")
        nonce = require_string(
            obj.get("dispatch_nonce"),
            f"verified_evidence[{index}].dispatch_nonce",
        )
        if not re.fullmatch(r"[0-9a-f]{32,}", nonce):
            fail(
                f"verified_evidence[{index}].dispatch_nonce must be at least "
                "32 lowercase hex characters"
            )
        digest = require_string(
            obj.get("receipt_sha256"),
            f"verified_evidence[{index}].receipt_sha256",
        )
        if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
            fail(f"verified_evidence[{index}].receipt_sha256 must be a canonical SHA-256")
        source = validate_evidence_source(
            obj.get("source"),
            f"verified_evidence[{index}].source",
            target_ref,
        )
    if len(verified_ids) != len(set(verified_ids)):
        fail("verified evidence IDs must be unique")
    if status == "INITIALIZING" and verified_evidence:
        fail("INITIALIZING state must not contain verified evidence")

    completed = data.get("completed")
    if not isinstance(completed, list):
        fail("completed must be an array")
    if cursor != len(completed):
        fail("cursor must equal completed entry count")
    expected_completed = sequence[:cursor]
    actual_completed = []
    for index, entry in enumerate(completed):
        obj = require_object(entry, f"completed[{index}]")
        actual_completed.append(require_string(obj.get("increment"), f"completed[{index}].increment"))
        require_oid(obj.get("commit"), f"completed[{index}].commit")
        require_oid(obj.get("tree"), f"completed[{index}].tree")
        receipt_status = obj.get("receipt_status")
        if receipt_status is not None:
            if publication_mode == "remote" and receipt_status != "LANDED_CLEAN":
                fail(
                    f"completed[{index}].receipt_status must be LANDED_CLEAN "
                    "for remote publication"
                )
            if publication_mode == "local" and receipt_status != "CLOSED_CLEAN":
                fail(
                    f"completed[{index}].receipt_status must be CLOSED_CLEAN "
                    "for local publication"
                )
        receipt_path = require_string(obj.get("receipt_path"), f"completed[{index}].receipt_path")
        if not is_canonical_repo_relative_path(receipt_path):
            fail(f"completed[{index}].receipt_path must be repository-relative")
    if actual_completed != expected_completed:
        fail("completed entries must match the sequence prefix")
    if completed:
        last_completed = completed[-1]
        if last_completed["commit"] != base["commit"] or last_completed["tree"] != base["tree"]:
            fail("expected_base must equal the last completed commit and tree")

    completed_by_increment = {
        entry["increment"]: entry
        for entry in completed
    }
    for evidence_id, obj in [
        *zip(evidence_ids, active_evidence_objects),
        *zip(verified_ids, verified_evidence_objects),
    ]:
        checkpoint = evidence_requirements[evidence_id]["after_increment"]
        checkpoint_entry = completed_by_increment.get(checkpoint)
        if checkpoint_entry is None:
            fail(f"{evidence_id} requires completed source checkpoint {checkpoint}")
        source = obj["source"]
        if (
            source["commit"] != checkpoint_entry["commit"]
            or source["tree"] != checkpoint_entry["tree"]
        ):
            fail(f"{evidence_id} source must match completed checkpoint {checkpoint}")

    required_for_current: set[str] = set()
    if cursor < len(sequence):
        required_for_current = {
            evidence_id
            for evidence_id, requirement in evidence_requirements.items()
            if requirement["before_increment"] == sequence[cursor]
        }
    verified_set = set(verified_ids)
    if status in {"EVIDENCE_RUNNING", "EVIDENCE_RECEIPTS_RECEIVED", "EVIDENCE_VERIFYING"}:
        if not required_for_current:
            fail(f"{status} requires an evidence-gated current increment")
        if set(evidence_ids) != required_for_current:
            fail("active evidence dispatches must exactly match the current evidence gate")
    if status in {"DISPATCHING", "RUNNING", "RECEIPT_RECEIVED", "VERIFYING"}:
        if not required_for_current.issubset(verified_set):
            fail("current increment may not run before all required evidence verifies")

    if status == "COMPLETE" and cursor != len(sequence):
        fail("COMPLETE requires every sequence entry")
    if cursor == len(sequence) and status != "COMPLETE":
        fail("a fully consumed sequence must use COMPLETE status")
    if status == "COMPLETE" and verified_set != set(evidence_requirements):
        fail("COMPLETE requires exactly all declared evidence to be verified")

    print(f"VALID state {data['orchestration_id']} status={status} cursor={cursor}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
