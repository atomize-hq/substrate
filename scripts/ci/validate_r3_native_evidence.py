#!/usr/bin/env python3

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
SHA256 = re.compile(r"^sha256:[0-9a-f]{64}$")
ACTION_TOKEN = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
ARCHITECTURE_TOKEN = re.compile(r"^[A-Za-z0-9._-]+$")
RESULT_STATUSES = {
    "EVIDENCE_CLEAN",
    "BLOCKED_PLATFORM_HANDOFF_REQUIRED",
    "BLOCKED_NATIVE_EVIDENCE",
}
PLATFORM_OSES = {"linux", "macos", "windows"}
KNOWN_ACTIONS = {
    "publish-manifest",
    "issue-receipt",
    "publish-head",
    "update-shared-claims",
    "intent-publication-probe",
    "bootstrap-publisher",
    "guest-pairing",
    "retire-test-publisher",
    "restore-baseline",
    "service-socket-smoke",
    "product-smoke",
    "install-repeat-retry",
    "world-smoke",
    "delete",
    "replace",
    "kill",
    "stop",
    "unregister",
    "ambient-home-mutation",
    "retained-sudo-credential",
    "restore-by-reset",
    "shared-state-removal",
    "import-instance",
}
TOP_LEVEL_FIELDS = {
    "schema_owner",
    "schema_version",
    "evidence_id",
    "source",
    "platform",
    "product_project_id",
    "tool_versions",
    "allowed_actions",
    "prohibited_actions",
    "commands",
    "artifact_manifest",
    "restoration_manifest",
    "intent_publication_capability",
    "correlation",
    "result",
    "gated_successor",
}


class ValidationError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def require_string(value: Any, field: str) -> str:
    require(isinstance(value, str) and value != "", f"{field} must be a non-empty string")
    return value


def require_hex(value: Any, field: str, pattern: re.Pattern[str]) -> str:
    value = require_string(value, field)
    require(pattern.fullmatch(value) is not None, f"{field} has invalid hexadecimal shape")
    return value


def require_unique_string_list(value: Any, field: str) -> list[str]:
    require(isinstance(value, list) and value, f"{field} must be a non-empty list")
    require(
        all(isinstance(item, str) and item for item in value),
        f"{field} must contain non-empty strings",
    )
    require(len(value) == len(set(value)), f"{field} must not contain duplicates")
    return list(value)


def require_string_list(value: Any, field: str) -> list[str]:
    require(isinstance(value, list) and value, f"{field} must be a non-empty list")
    require(
        all(isinstance(item, str) and item for item in value),
        f"{field} must contain non-empty strings",
    )
    return list(value)


def require_digest_object(value: Any, field: str) -> None:
    require(isinstance(value, dict), f"{field} must be an object")
    require(set(value) == {"sha256", "entries"}, f"{field} fields do not match the closed shape")
    require_hex(value["sha256"], f"{field}.sha256", SHA256)
    entries = require_unique_string_list(value["entries"], f"{field}.entries")
    require(
        all(entry.endswith(".json") for entry in entries),
        f"{field}.entries must contain JSON artifact paths",
    )


def expected_platform_os(expected_evidence_id: str) -> str | None:
    upper = expected_evidence_id.upper()
    if "LINUX" in upper:
        return "linux"
    if "MAC" in upper:
        return "macos"
    if "WIN" in upper or "WINDOWS" in upper:
        return "windows"
    return None


def require_platform(value: Any, expected_evidence_id: str) -> None:
    require(isinstance(value, dict), "platform must be an object")
    require(set(value) == {"os", "architecture"}, "platform fields do not match the closed shape")
    os_name = require_string(value["os"], "platform.os")
    require(os_name in PLATFORM_OSES, "platform.os must be a supported platform token")
    architecture = require_string(value["architecture"], "platform.architecture")
    require(
        ARCHITECTURE_TOKEN.fullmatch(architecture) is not None,
        "platform.architecture must be a canonical architecture token",
    )
    expected_os = expected_platform_os(expected_evidence_id)
    if expected_os is not None:
        require(
            os_name == expected_os,
            "platform.os does not match the expected evidence platform",
        )


def require_action_set(value: Any, field: str) -> list[str]:
    actions = require_unique_string_list(value, field)
    for action in actions:
        require(
            ACTION_TOKEN.fullmatch(action) is not None,
            f"{field} contains a non-canonical action token",
        )
        require(action in KNOWN_ACTIONS, f"{field} contains an unknown action token")
    return actions


def require_source(value: Any) -> dict[str, Any]:
    require(isinstance(value, dict), "source must be an object")
    require(
        set(value) == {"commit", "tree", "ref", "live_remote"},
        "source fields do not match the closed shape",
    )
    require_hex(value["commit"], "source.commit", HEX40)
    require_hex(value["tree"], "source.tree", HEX40)
    require_string(value["ref"], "source.ref")
    require_hex(value["live_remote"], "source.live_remote", HEX40)
    require(
        value["live_remote"] == value["commit"],
        "source.live_remote must equal source.commit",
    )
    return value


def require_correlation(value: Any) -> None:
    require(isinstance(value, dict), "correlation must be an object")
    require(
        set(value)
        == {
            "dispatch_nonce",
            "orchestration_id",
            "source_task_thread_id",
            "source_task_host_id",
            "evidence_task_thread_id",
            "evidence_task_host_id",
            "return_thread_id",
            "return_host_id",
        },
        "correlation fields do not match the closed shape",
    )
    for field in sorted(value):
        require_string(value[field], f"correlation.{field}")


def require_commands(value: Any) -> None:
    require(isinstance(value, list) and value, "commands must be a non-empty list")
    expected_index = 1
    for command in value:
        require(isinstance(command, dict), "commands entries must be objects")
        require(
            set(command) == {"index", "argv", "status", "stdout_sha256", "stderr_sha256"},
            "command fields do not match the closed shape",
        )
        require(
            type(command["index"]) is int and command["index"] == expected_index,
            "commands indices must start at 1 and increase without gaps",
        )
        expected_index += 1
        require_string_list(command["argv"], f"commands[{command['index']}].argv")
        require(
            type(command["status"]) is int,
            f"commands[{command['index']}].status must be an integer",
        )
        require_hex(
            command["stdout_sha256"],
            f"commands[{command['index']}].stdout_sha256",
            SHA256,
        )
        require_hex(
            command["stderr_sha256"],
            f"commands[{command['index']}].stderr_sha256",
            SHA256,
        )


def require_result(value: Any) -> None:
    require(isinstance(value, dict), "result must be an object")
    require(
        set(value) == {"status", "summary", "restoration_exact"},
        "result fields do not match the closed shape",
    )
    require(
        value["status"] in RESULT_STATUSES,
        "result.status must be an allowed evidence terminal status",
    )
    require_string(value["summary"], "result.summary")
    require(
        type(value["restoration_exact"]) is bool,
        "result.restoration_exact must be a boolean",
    )


def require_intent_capability(value: Any) -> None:
    if value is None:
        return
    require(isinstance(value, dict), "intent_publication_capability must be null or an object")
    require(
        set(value)
        == {
            "state_root_identity",
            "filesystem_identity",
            "same_filesystem",
            "otmpfile_supported",
            "linkat_empty_path_supported",
            "effective_root",
            "probe_commands",
            "probe_artifact_sha256",
        },
        "intent_publication_capability fields do not match the closed shape",
    )
    require_string(value["state_root_identity"], "intent_publication_capability.state_root_identity")
    require_string(value["filesystem_identity"], "intent_publication_capability.filesystem_identity")
    require(type(value["same_filesystem"]) is bool, "intent_publication_capability.same_filesystem must be a boolean")
    require(type(value["otmpfile_supported"]) is bool, "intent_publication_capability.otmpfile_supported must be a boolean")
    require(
        type(value["linkat_empty_path_supported"]) is bool,
        "intent_publication_capability.linkat_empty_path_supported must be a boolean",
    )
    require(type(value["effective_root"]) is bool, "intent_publication_capability.effective_root must be a boolean")
    require_commands(value["probe_commands"])
    require_hex(
        value["probe_artifact_sha256"],
        "intent_publication_capability.probe_artifact_sha256",
        SHA256,
    )


def validate_artifact(
    artifact: Any,
    *,
    expected_evidence_id: str,
    expected_source_commit: str,
    expected_source_tree: str,
    expected_source_ref: str,
    expected_gated_successor: str,
) -> None:
    require(isinstance(artifact, dict), "artifact must be a JSON object")
    require(
        set(artifact) == TOP_LEVEL_FIELDS,
        "artifact fields do not match the closed v1 shape",
    )
    require(
        artifact["schema_owner"] == "substrate.r3-native-evidence",
        "schema_owner must equal substrate.r3-native-evidence",
    )
    require(
        type(artifact["schema_version"]) is int and artifact["schema_version"] == 1,
        "schema_version must equal 1",
    )
    require(
        artifact["evidence_id"] == expected_evidence_id,
        "evidence_id does not match the expected value",
    )
    source = require_source(artifact["source"])
    require(
        source["commit"] == expected_source_commit,
        "source.commit does not match the expected value",
    )
    require(
        source["tree"] == expected_source_tree,
        "source.tree does not match the expected value",
    )
    require(source["ref"] == expected_source_ref, "source.ref does not match the expected value")
    require(
        artifact["gated_successor"] == expected_gated_successor,
        "gated_successor does not match the expected value",
    )
    require(
        artifact["product_project_id"] == "2ccb802f-301c-4af4-9bd5-51d22808f0a2",
        "product_project_id must equal the fixed R3 project UUID",
    )
    require_platform(artifact["platform"], expected_evidence_id)
    allowed_actions = require_action_set(artifact["allowed_actions"], "allowed_actions")
    prohibited_actions = require_action_set(artifact["prohibited_actions"], "prohibited_actions")
    require(
        set(allowed_actions).isdisjoint(prohibited_actions),
        "allowed_actions and prohibited_actions must be disjoint",
    )
    require_commands(artifact["commands"])
    require_digest_object(artifact["artifact_manifest"], "artifact_manifest")
    require_digest_object(artifact["restoration_manifest"], "restoration_manifest")
    require(
        isinstance(artifact["tool_versions"], dict) and artifact["tool_versions"],
        "tool_versions must be a non-empty object",
    )
    for key, value in artifact["tool_versions"].items():
        require_string(key, "tool_versions key")
        require_string(value, f"tool_versions.{key}")
    require_correlation(artifact["correlation"])
    require_result(artifact["result"])
    require_intent_capability(artifact["intent_publication_capability"])


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate a substrate R3 native evidence artifact")
    parser.add_argument("artifact", type=Path)
    parser.add_argument("--expected-evidence-id", required=True)
    parser.add_argument("--expected-source-commit", required=True)
    parser.add_argument("--expected-source-tree", required=True)
    parser.add_argument("--expected-source-ref", required=True)
    parser.add_argument("--expected-gated-successor", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        artifact = json.loads(args.artifact.read_text(encoding="utf-8"))
        validate_artifact(
            artifact,
            expected_evidence_id=args.expected_evidence_id,
            expected_source_commit=args.expected_source_commit,
            expected_source_tree=args.expected_source_tree,
            expected_source_ref=args.expected_source_ref,
            expected_gated_successor=args.expected_gated_successor,
        )
    except (OSError, json.JSONDecodeError, ValidationError) as error:
        print(f"INVALID: {args.artifact}: {error}", file=sys.stderr)
        return 1
    print(f"VALID: {args.artifact}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
