#!/usr/bin/env python3

import argparse
import json
import os
import sys
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Set, Tuple


ALLOWED_TASK_TYPES = {"code", "test", "integration", "ops", "investigation"}
ALLOWED_TASK_STATUSES = {"pending", "in_progress", "completed", "queued", "blocked", "canceled"}
ALLOWED_PLATFORMS = {"linux", "macos", "windows", "wsl"}
ALLOWED_RUNNERS = {"local", "github-actions", "manual"}


@dataclass(frozen=True)
class ValidationError:
    message: str


def _is_str_list(value: Any) -> bool:
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def _read_json(path: str) -> Any:
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def _error(errors: List[ValidationError], message: str) -> None:
    errors.append(ValidationError(message=message))


def _validate_tasks_shape(data: Any, errors: List[ValidationError], path: str) -> Optional[List[Dict[str, Any]]]:
    if not isinstance(data, dict):
        _error(errors, f"{path}: expected top-level JSON object with a `tasks` array, got {type(data).__name__}")
        return None
    if "tasks" not in data:
        _error(errors, f"{path}: missing top-level key `tasks`")
        return None
    tasks = data["tasks"]
    if not isinstance(tasks, list):
        _error(errors, f"{path}: `tasks` must be an array, got {type(tasks).__name__}")
        return None
    if not all(isinstance(task, dict) for task in tasks):
        _error(errors, f"{path}: every entry in `tasks` must be an object")
        return None
    return tasks


def _validate_task_fields(tasks: List[Dict[str, Any]], errors: List[ValidationError], path: str) -> None:
    required = [
        "id",
        "name",
        "type",
        "phase",
        "status",
        "description",
        "references",
        "acceptance_criteria",
        "start_checklist",
        "end_checklist",
        "worktree",
        "integration_task",
        "kickoff_prompt",
        "depends_on",
        "concurrent_with",
    ]

    ids: List[str] = []
    for index, task in enumerate(tasks):
        prefix = f"{path}:tasks[{index}]"

        missing = [key for key in required if key not in task]
        if missing:
            _error(errors, f"{prefix}: missing required keys: {', '.join(missing)}")
            continue

        if not isinstance(task["id"], str) or not task["id"]:
            _error(errors, f"{prefix}.id: must be a non-empty string")
        else:
            ids.append(task["id"])

        if not isinstance(task["name"], str) or not task["name"]:
            _error(errors, f"{prefix}.name: must be a non-empty string")
        if not isinstance(task["phase"], str) or not task["phase"]:
            _error(errors, f"{prefix}.phase: must be a non-empty string")
        if not isinstance(task["description"], str) or not task["description"]:
            _error(errors, f"{prefix}.description: must be a non-empty string")

        task_type = task["type"]
        if task_type not in ALLOWED_TASK_TYPES:
            _error(errors, f"{prefix}.type: must be one of {sorted(ALLOWED_TASK_TYPES)}, got {task_type!r}")

        status = task["status"]
        if status not in ALLOWED_TASK_STATUSES:
            _error(errors, f"{prefix}.status: must be one of {sorted(ALLOWED_TASK_STATUSES)}, got {status!r}")

        if not _is_str_list(task["references"]):
            _error(errors, f"{prefix}.references: must be an array of strings")
        if not _is_str_list(task["acceptance_criteria"]):
            _error(errors, f"{prefix}.acceptance_criteria: must be an array of strings")
        if not _is_str_list(task["start_checklist"]):
            _error(errors, f"{prefix}.start_checklist: must be an array of strings")
        if not _is_str_list(task["end_checklist"]):
            _error(errors, f"{prefix}.end_checklist: must be an array of strings")

        task_type = task["type"]

        worktree_value = task["worktree"]
        if task_type in {"code", "test", "integration"}:
            if not isinstance(worktree_value, str) or not worktree_value:
                _error(errors, f"{prefix}.worktree: must be a non-empty string (recommended: starts with `wt/`)")
        else:
            if worktree_value is not None and (not isinstance(worktree_value, str) or not worktree_value):
                _error(errors, f"{prefix}.worktree: must be null or a non-empty string")

        integration_task_value = task["integration_task"]
        if task_type == "integration":
            if integration_task_value is None:
                pass
            elif not isinstance(integration_task_value, str):
                _error(errors, f"{prefix}.integration_task: must be a string or null for integration tasks")
        elif task_type in {"code", "test"}:
            if not isinstance(integration_task_value, str) or not integration_task_value:
                _error(errors, f"{prefix}.integration_task: must be a non-empty string")
        else:
            if integration_task_value is not None and (not isinstance(integration_task_value, str) or not integration_task_value):
                _error(errors, f"{prefix}.integration_task: must be null or a non-empty string")

        kickoff_prompt_value = task["kickoff_prompt"]
        if task_type in {"code", "test", "integration"}:
            if not isinstance(kickoff_prompt_value, str) or not kickoff_prompt_value:
                _error(errors, f"{prefix}.kickoff_prompt: must be a non-empty string path")
        else:
            if kickoff_prompt_value is not None and (not isinstance(kickoff_prompt_value, str) or not kickoff_prompt_value):
                _error(errors, f"{prefix}.kickoff_prompt: must be null or a non-empty string path")

        if not _is_str_list(task["depends_on"]):
            _error(errors, f"{prefix}.depends_on: must be an array of strings")
        if not _is_str_list(task["concurrent_with"]):
            _error(errors, f"{prefix}.concurrent_with: must be an array of strings")

        platform = task.get("platform")
        if platform is not None and platform not in ALLOWED_PLATFORMS:
            _error(errors, f"{prefix}.platform: must be one of {sorted(ALLOWED_PLATFORMS)}, got {platform!r}")

        runner = task.get("runner")
        if runner is not None and runner not in ALLOWED_RUNNERS:
            _error(errors, f"{prefix}.runner: must be one of {sorted(ALLOWED_RUNNERS)}, got {runner!r}")

    duplicates = {task_id for task_id in ids if ids.count(task_id) > 1}
    if duplicates:
        _error(errors, f"{path}: duplicate task ids: {', '.join(sorted(duplicates))}")


def _validate_references(
    feature_dir: str,
    tasks: List[Dict[str, Any]],
    external_task_ids: Set[str],
    errors: List[ValidationError],
    path: str,
) -> None:
    all_task_ids: Set[str] = {task.get("id") for task in tasks if isinstance(task.get("id"), str)}
    tasks_by_id: Dict[str, Dict[str, Any]] = {task["id"]: task for task in tasks if isinstance(task.get("id"), str)}

    for index, task in enumerate(tasks):
        prefix = f"{path}:tasks[{index}]({task.get('id', '<missing id>')})"

        for dep in task.get("depends_on", []):
            if dep in all_task_ids or dep in external_task_ids:
                continue
            _error(
                errors,
                f"{prefix}.depends_on: unknown task id {dep!r} (if external, add it to tasks.json meta.external_task_ids)",
            )
        for other in task.get("concurrent_with", []):
            if other in all_task_ids or other in external_task_ids:
                continue
            _error(
                errors,
                f"{prefix}.concurrent_with: unknown task id {other!r} (if external, add it to tasks.json meta.external_task_ids)",
            )

        integration_task = task.get("integration_task")
        task_type = task.get("type")
        if task_type == "integration":
            if not isinstance(integration_task, str) or not integration_task:
                continue
            if integration_task != task.get("id"):
                _error(errors, f"{prefix}.integration_task: integration tasks should set integration_task to their own id")
            continue

        if task_type not in {"code", "test"}:
            continue

        if not isinstance(integration_task, str) or not integration_task:
            _error(errors, f"{prefix}.integration_task: must be a non-empty string")
        elif integration_task in tasks_by_id:
            integration_type = tasks_by_id[integration_task].get("type")
            if integration_type != "integration":
                _error(errors, f"{prefix}.integration_task: {integration_task!r} must reference a task with type=integration")
        else:
            _error(errors, f"{prefix}.integration_task: unknown task id {integration_task!r}")

        kickoff_prompt = task.get("kickoff_prompt")
        if isinstance(kickoff_prompt, str):
            if not os.path.exists(kickoff_prompt):
                _error(errors, f"{prefix}.kickoff_prompt: file does not exist: {kickoff_prompt!r}")
            elif os.path.commonpath([os.path.abspath(kickoff_prompt), os.path.abspath(feature_dir)]) != os.path.abspath(
                feature_dir
            ):
                _error(errors, f"{prefix}.kickoff_prompt: must live under feature dir: {kickoff_prompt!r}")


def _validate_smoke_linkage(feature_dir: str, tasks: List[Dict[str, Any]], errors: List[ValidationError], path: str) -> None:
    smoke_dir = os.path.join(feature_dir, "smoke")
    if not os.path.isdir(smoke_dir):
        return

    for index, task in enumerate(tasks):
        if task.get("type") != "integration":
            continue
        txt = "\n".join(task.get("references", []) + task.get("end_checklist", []))
        if "smoke/" not in txt:
            _error(
                errors,
                f"{path}:tasks[{index}]({task.get('id')}): integration task must reference smoke scripts in references/end_checklist",
            )


def validate_tasks_json(feature_dir: str) -> Tuple[List[ValidationError], str]:
    tasks_path = os.path.join(feature_dir, "tasks.json")
    errors: List[ValidationError] = []

    if not os.path.isfile(tasks_path):
        _error(errors, f"{tasks_path}: missing")
        return errors, tasks_path

    try:
        data = _read_json(tasks_path)
    except json.JSONDecodeError as exc:
        _error(errors, f"{tasks_path}: invalid JSON: {exc}")
        return errors, tasks_path

    tasks = _validate_tasks_shape(data, errors, tasks_path)
    if tasks is None:
        return errors, tasks_path

    meta = data.get("meta", {}) if isinstance(data, dict) else {}
    external_list = meta.get("external_task_ids", []) if isinstance(meta, dict) else []
    external_task_ids: Set[str] = set(external_list) if isinstance(external_list, list) else set()
    if not all(isinstance(x, str) for x in external_task_ids):
        _error(errors, f"{tasks_path}: meta.external_task_ids must be an array of strings")

    _validate_task_fields(tasks, errors, tasks_path)
    _validate_references(feature_dir, tasks, external_task_ids, errors, tasks_path)
    _validate_smoke_linkage(feature_dir, tasks, errors, tasks_path)

    return errors, tasks_path


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate Planning Pack tasks.json invariants.")
    parser.add_argument("--feature-dir", required=True, help="Feature Planning Pack directory under docs/project_management/next/...")
    args = parser.parse_args()

    feature_dir = args.feature_dir.rstrip("/").rstrip("\\")
    errors, tasks_path = validate_tasks_json(feature_dir=feature_dir)
    if errors:
        for err in errors:
            print(err.message, file=sys.stderr)
        print(f"FAIL: tasks.json validation failed: {tasks_path}", file=sys.stderr)
        return 1

    print(f"OK: tasks.json validation passed: {tasks_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
