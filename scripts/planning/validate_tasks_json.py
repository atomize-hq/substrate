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
ALLOWED_PLATFORMS_REQUIRED = {"linux", "macos", "windows"}
ALLOWED_RUNNERS = {"local", "github-actions", "manual"}
DEFAULT_SCHEMA_VERSION = 1
ALLOWED_WSL_TASK_MODES = {"bundled", "separate"}
AUTOMATION_SCHEMA_VERSION = 3


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


def _validate_meta(data: Any, errors: List[ValidationError], path: str) -> Dict[str, Any]:
    if not isinstance(data, dict):
        return {}

    meta = data.get("meta", {})
    if meta is None:
        return {}
    if not isinstance(meta, dict):
        _error(errors, f"{path}: meta must be an object when present")
        return {}

    schema_version = meta.get("schema_version", DEFAULT_SCHEMA_VERSION)
    if not isinstance(schema_version, int) or schema_version < 1:
        _error(errors, f"{path}: meta.schema_version must be an integer >= 1")

    platforms_required = meta.get("platforms_required")
    if platforms_required is not None:
        if not isinstance(platforms_required, list) or not all(isinstance(p, str) for p in platforms_required):
            _error(errors, f"{path}: meta.platforms_required must be an array of strings")
        else:
            unknown = sorted({p for p in platforms_required if p not in ALLOWED_PLATFORMS_REQUIRED})
            if unknown:
                _error(errors, f"{path}: meta.platforms_required contains unknown platform(s): {', '.join(unknown)}")
                if "wsl" in unknown:
                    _error(
                        errors,
                        f"{path}: do not include 'wsl' in meta.platforms_required; use meta.wsl_required=true and meta.wsl_task_mode='bundled'|'separate'",
                    )
            duplicates = sorted({p for p in platforms_required if platforms_required.count(p) > 1})
            if duplicates:
                _error(errors, f"{path}: meta.platforms_required contains duplicate platform(s): {', '.join(duplicates)}")

    wsl_required = meta.get("wsl_required")
    if wsl_required is not None and not isinstance(wsl_required, bool):
        _error(errors, f"{path}: meta.wsl_required must be a boolean when present")

    wsl_task_mode = meta.get("wsl_task_mode")
    if wsl_task_mode is not None:
        if not isinstance(wsl_task_mode, str) or wsl_task_mode not in ALLOWED_WSL_TASK_MODES:
            _error(errors, f"{path}: meta.wsl_task_mode must be one of {sorted(ALLOWED_WSL_TASK_MODES)} when present")
        if wsl_required is not True:
            _error(errors, f"{path}: meta.wsl_task_mode requires meta.wsl_required=true")

    # Back-compat convenience: if WSL is required but no mode specified, default to bundled.
    if wsl_required is True and wsl_task_mode is None:
        meta["wsl_task_mode"] = "bundled"

    if wsl_required is True:
        if not isinstance(platforms_required, list) or "linux" not in platforms_required:
            _error(errors, f"{path}: meta.wsl_required=true requires meta.platforms_required to include 'linux'")

    execution_gates = meta.get("execution_gates")
    if execution_gates is not None and not isinstance(execution_gates, bool):
        _error(errors, f"{path}: meta.execution_gates must be a boolean when present")

    return meta


def _validate_execution_gates(
    feature_dir: str, tasks: List[Dict[str, Any]], meta: Dict[str, Any], errors: List[ValidationError], path: str
) -> None:
    if meta.get("execution_gates") is not True:
        return

    preflight_report = os.path.join(feature_dir, "execution_preflight_report.md")
    if not os.path.isfile(preflight_report):
        _error(errors, f"{path}: meta.execution_gates=true requires {preflight_report!r} to exist")

    tasks_by_id: Dict[str, Dict[str, Any]] = {t.get("id"): t for t in tasks if isinstance(t.get("id"), str)}
    preflight_task = tasks_by_id.get("F0-exec-preflight")
    if preflight_task is None:
        _error(errors, f"{path}: meta.execution_gates=true requires a task with id 'F0-exec-preflight'")
    else:
        if preflight_task.get("type") != "ops":
            _error(errors, f"{path}: 'F0-exec-preflight' must have type='ops'")
        txt = "\n".join(preflight_task.get("references", []) + preflight_task.get("start_checklist", []) + preflight_task.get("end_checklist", []))
        if "execution_preflight_report.md" not in txt:
            _error(errors, f"{path}: 'F0-exec-preflight' must reference execution_preflight_report.md")

    # Per-slice closeout reports: require <SLICE>-closeout_report.md and linkage from <SLICE>-integ.
    slice_ids: Set[str] = set()
    for task_id, task in tasks_by_id.items():
        if not isinstance(task_id, str):
            continue
        if task.get("type") != "integration":
            continue
        if task_id.endswith("-integ") and not task_id.endswith("-integ-core"):
            slice_ids.add(task_id[: -len("-integ")])

    for slice_id in sorted(slice_ids):
        closeout_report = os.path.join(feature_dir, f"{slice_id}-closeout_report.md")
        if not os.path.isfile(closeout_report):
            _error(errors, f"{path}: meta.execution_gates=true requires {closeout_report!r} to exist")

        final_id = f"{slice_id}-integ"
        final_task = tasks_by_id.get(final_id)
        if final_task is None:
            continue
        txt = "\n".join(final_task.get("references", []) + final_task.get("end_checklist", []))
        if f"{slice_id}-closeout_report.md" not in txt:
            _error(errors, f"{path}: {final_id!r} must reference {slice_id}-closeout_report.md in references/end_checklist")


def _validate_task_automation(
    feature_dir: str, tasks: List[Dict[str, Any]], meta: Dict[str, Any], errors: List[ValidationError], path: str
) -> None:
    """
    Enforce the triad automation shape only when explicitly opted in:
      - meta.schema_version >= 3, and
      - meta.automation.enabled == true

    This avoids breaking existing Planning Packs.
    """
    schema_version = meta.get("schema_version", DEFAULT_SCHEMA_VERSION)
    automation = meta.get("automation")
    if not isinstance(schema_version, int):
        return
    if schema_version < AUTOMATION_SCHEMA_VERSION:
        # If someone sets meta.automation.enabled=true without bumping schema_version, treat as invalid opt-in.
        if isinstance(automation, dict) and automation.get("enabled") is True:
            _error(errors, f"{path}: meta.automation.enabled=true requires meta.schema_version >= {AUTOMATION_SCHEMA_VERSION}")
        return

    if not isinstance(automation, dict) or automation.get("enabled") is not True:
        _error(errors, f"{path}: meta.schema_version >= {AUTOMATION_SCHEMA_VERSION} requires meta.automation.enabled=true")
        return

    feature = meta.get("feature")
    if not isinstance(feature, str) or not feature:
        _error(errors, f"{path}: meta.schema_version >= {AUTOMATION_SCHEMA_VERSION} requires meta.feature to be a non-empty string")

    orchestration_branch = automation.get("orchestration_branch")
    if not isinstance(orchestration_branch, str) or not orchestration_branch:
        _error(
            errors,
            f"{path}: meta.schema_version >= {AUTOMATION_SCHEMA_VERSION} requires meta.automation.orchestration_branch to be a non-empty string",
        )

    tasks_by_id: Dict[str, Dict[str, Any]] = {t.get("id"): t for t in tasks if isinstance(t.get("id"), str)}

    # Feature-level cleanup task (required for worktree retention model).
    cleanup = tasks_by_id.get("FZ-feature-cleanup")
    if cleanup is None:
        _error(errors, f"{path}: meta.schema_version >= {AUTOMATION_SCHEMA_VERSION} requires an ops task with id 'FZ-feature-cleanup'")
    else:
        if cleanup.get("type") != "ops":
            _error(errors, f"{path}: 'FZ-feature-cleanup' must have type='ops'")
        if cleanup.get("worktree") is not None:
            _error(errors, f"{path}: 'FZ-feature-cleanup' must set worktree=null")
        kickoff = cleanup.get("kickoff_prompt")
        if not isinstance(kickoff, str) or not kickoff:
            _error(errors, f"{path}: 'FZ-feature-cleanup' must have a kickoff_prompt path")
        else:
            expected_prefix = os.path.join(feature_dir, "kickoff_prompts")
            if os.path.commonpath([os.path.abspath(kickoff), os.path.abspath(expected_prefix)]) != os.path.abspath(expected_prefix):
                _error(errors, f"{path}: 'FZ-feature-cleanup' kickoff_prompt must live under feature_dir/kickoff_prompts")

    # Per-task structured automation fields.
    branches: List[str] = []
    for index, task in enumerate(tasks):
        task_id = task.get("id")
        if not isinstance(task_id, str):
            continue

        task_type = task.get("type")
        if task_type not in {"code", "test", "integration"}:
            continue

        prefix = f"{path}:tasks[{index}]({task_id})"

        git_branch = task.get("git_branch")
        if not isinstance(git_branch, str) or not git_branch:
            _error(errors, f"{prefix}.git_branch: required non-empty string for automation packs")
        else:
            branches.append(git_branch)

        required_targets = task.get("required_make_targets")
        if required_targets is None:
            _error(errors, f"{prefix}.required_make_targets: required array of strings (may be empty) for automation packs")
        elif not _is_str_list(required_targets):
            _error(errors, f"{prefix}.required_make_targets: must be an array of strings")

        if task_type == "integration":
            merge_to_orch = task.get("merge_to_orchestration")
            if not isinstance(merge_to_orch, bool):
                _error(errors, f"{prefix}.merge_to_orchestration: required boolean for integration tasks in automation packs")

    duplicates = {b for b in branches if branches.count(b) > 1}
    if duplicates:
        _error(errors, f"{path}: duplicate git_branch values (must be unique): {', '.join(sorted(duplicates))}")

    # Parallel code/test pairing is required for automation packs so the pair launcher can be used
    # deterministically and without ad-hoc task selection.
    for code_id, code_task in tasks_by_id.items():
        if not isinstance(code_id, str) or not code_id.endswith("-code"):
            continue
        if code_task.get("type") != "code":
            _error(errors, f"{path}: {code_id!r} ends with '-code' but has type={code_task.get('type')!r}")
            continue

        test_id = f"{code_id[:-5]}-test"
        test_task = tasks_by_id.get(test_id)
        if test_task is None:
            _error(errors, f"{path}: automation packs require a matching test task {test_id!r} for code task {code_id!r}")
            continue
        if test_task.get("type") != "test":
            _error(errors, f"{path}: {test_id!r} must have type='test' (paired with {code_id!r})")

        code_concurrent = code_task.get("concurrent_with")
        test_concurrent = test_task.get("concurrent_with")
        if isinstance(code_concurrent, list) and test_id not in code_concurrent:
            _error(errors, f"{path}: {code_id!r}.concurrent_with must include {test_id!r} (parallel code/test is required)")
        if isinstance(test_concurrent, list) and code_id not in test_concurrent:
            _error(errors, f"{path}: {test_id!r}.concurrent_with must include {code_id!r} (parallel code/test is required)")

        code_integration = code_task.get("integration_task")
        test_integration = test_task.get("integration_task")
        if code_integration != test_integration:
            _error(
                errors,
                f"{path}: {code_id!r} and {test_id!r} must share the same integration_task (got {code_integration!r} vs {test_integration!r})",
            )


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

def _validate_platform_integ_model(
    feature_dir: str, tasks: List[Dict[str, Any]], meta: Dict[str, Any], errors: List[ValidationError], path: str
) -> None:
    """
    Enforce the cross-platform integration structure only when the planning pack opts in via:
      - meta.schema_version >= 2, and
      - meta.platforms_required is present.

    Model (per slice X):
      - X-integ-core (integration): merges code+tests and gets primary-platform green
      - X-integ-<platform> (integration, platform set): platform-fix task (may be no-op if already green)
      - X-integ (integration): final aggregator merges any platform fixes and records results
    """
    schema_version = meta.get("schema_version", DEFAULT_SCHEMA_VERSION)
    if not isinstance(schema_version, int) or schema_version < 2:
        return

    platforms_required = meta.get("platforms_required")
    if not isinstance(platforms_required, list) or not platforms_required:
        return

    wsl_required = meta.get("wsl_required") is True
    wsl_task_mode = meta.get("wsl_task_mode", "bundled") if wsl_required else None

    effective_platform_tasks = list(platforms_required)
    if wsl_required and wsl_task_mode == "separate":
        effective_platform_tasks.append("wsl")

    tasks_by_id: Dict[str, Dict[str, Any]] = {t.get("id"): t for t in tasks if isinstance(t.get("id"), str)}

    # Determine slices present by looking for platform integ task ids.
    slices: Dict[str, Set[str]] = {}
    for task_id, task in tasks_by_id.items():
        if not isinstance(task_id, str):
            continue
        if task.get("type") != "integration":
            continue
        for platform in effective_platform_tasks:
            suffix = f"-integ-{platform}"
            if task_id.endswith(suffix):
                slice_id = task_id[: -len(suffix)]
                slices.setdefault(slice_id, set()).add(platform)
                break

    if not slices:
        _error(
            errors,
            f"{path}: meta.schema_version>=2 and meta.platforms_required set, but no '*-integ-<platform>' integration tasks found",
        )
        return

    for slice_id, platforms_present in sorted(slices.items()):
        missing = sorted(set(effective_platform_tasks) - platforms_present)
        if missing:
            _error(
                errors,
                f"{path}: slice {slice_id!r} missing required platform integration task(s): {', '.join(missing)}",
            )

        core_id = f"{slice_id}-integ-core"
        final_id = f"{slice_id}-integ"

        core = tasks_by_id.get(core_id)
        if core is None:
            _error(errors, f"{path}: missing required core integration task: {core_id!r}")
        elif core.get("type") != "integration":
            _error(errors, f"{path}: {core_id!r} must have type=integration")

        final = tasks_by_id.get(final_id)
        if final is None:
            _error(errors, f"{path}: missing required final integration task: {final_id!r}")
        elif final.get("type") != "integration":
            _error(errors, f"{path}: {final_id!r} must have type=integration")

        # Dependency wiring: platform tasks depend on core; final depends on core + all platform tasks.
        if core is None or final is None:
            continue

        if wsl_required and wsl_task_mode == "bundled":
            wsl_id = f"{slice_id}-integ-wsl"
            if wsl_id in tasks_by_id:
                _error(
                    errors,
                    f"{path}: {wsl_id!r} exists but meta.wsl_task_mode='bundled' (remove the task or set meta.wsl_task_mode='separate')",
                )

        for platform in effective_platform_tasks:
            platform_id = f"{slice_id}-integ-{platform}"
            platform_task = tasks_by_id.get(platform_id)
            if platform_task is None:
                continue
            if platform_task.get("type") != "integration":
                _error(errors, f"{path}: {platform_id!r} must have type=integration")
                continue
            if platform_task.get("platform") != platform:
                _error(errors, f"{path}: {platform_id!r} must set platform={platform!r}")

            depends_on = platform_task.get("depends_on")
            if not isinstance(depends_on, list) or core_id not in depends_on:
                _error(errors, f"{path}: {platform_id!r} depends_on must include {core_id!r}")

            # Bundled WSL: require the Linux platform task to include WSL smoke.
            if wsl_required and wsl_task_mode == "bundled" and platform == "linux":
                txt = "\n".join(platform_task.get("references", []) + platform_task.get("end_checklist", []))
                if "--run-wsl" not in txt and "run_wsl=true" not in txt and "run_wsl" not in txt:
                    _error(
                        errors,
                        f"{path}: {platform_id!r} must include WSL smoke dispatch (expected '--run-wsl') because meta.wsl_required=true and meta.wsl_task_mode='bundled'",
                    )

        final_deps = final.get("depends_on")
        if not isinstance(final_deps, list):
            _error(errors, f"{path}: {final_id!r} depends_on must be an array")
            continue
        if core_id not in final_deps:
            _error(errors, f"{path}: {final_id!r} depends_on must include {core_id!r}")
        for platform in effective_platform_tasks:
            platform_id = f"{slice_id}-integ-{platform}"
            if platform_id in tasks_by_id and platform_id not in final_deps:
                _error(errors, f"{path}: {final_id!r} depends_on must include {platform_id!r}")


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

    meta = _validate_meta(data, errors, tasks_path)

    tasks = _validate_tasks_shape(data, errors, tasks_path)
    if tasks is None:
        return errors, tasks_path

    external_list = meta.get("external_task_ids", []) if isinstance(meta, dict) else []
    external_task_ids: Set[str] = set(external_list) if isinstance(external_list, list) else set()
    if not all(isinstance(x, str) for x in external_task_ids):
        _error(errors, f"{tasks_path}: meta.external_task_ids must be an array of strings")

    _validate_task_fields(tasks, errors, tasks_path)
    _validate_references(feature_dir, tasks, external_task_ids, errors, tasks_path)
    _validate_smoke_linkage(feature_dir, tasks, errors, tasks_path)
    _validate_platform_integ_model(feature_dir, tasks, meta, errors, tasks_path)
    _validate_execution_gates(feature_dir, tasks, meta, errors, tasks_path)
    _validate_task_automation(feature_dir, tasks, meta, errors, tasks_path)

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
