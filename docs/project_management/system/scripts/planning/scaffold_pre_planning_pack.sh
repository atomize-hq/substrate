#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scaffold_pre_planning_pack.sh --adr <path> [--feature <slug>] [--bucket <bucket>]

Required:
  --adr <path>        ADR markdown file (must exist inside this repo)

Optional:
  --feature <slug>    Feature directory name (default: derive from ADR filename)
  --bucket <bucket>   Pack bucket under docs/project_management/packs/ (default: PM_DEFAULT_PACK_BUCKET, else draft)

Output contract:
  - stdout: a single repo-relative POSIX path to the feature dir (no trailing slash), e.g.:
      docs/project_management/packs/draft/<feature>
  - stderr: errors / diagnostics

Behavior:
  - Creates/ensures <FEATURE_DIR>/ exists.
  - Ensures <FEATURE_DIR>/tasks.json exists and includes meta.adr_paths containing the ADR path.
  - Creates tasks.json if missing with minimal schema:
      {
        "meta": {
          "schema_version": 4,
          "cross_platform": true,
          "ci_parity_platforms_required": ["linux", "macos", "windows"],
          "behavior_platforms_required": ["linux", "macos", "windows"],
          "automation": { "enabled": true, "orchestration_branch": "feat/<feature>" },
          "slice_spec_version": 2,
          "feature": "<slug>",
          "adr_paths": ["<adr>"]
        },
        "tasks": []
      }
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

need_cmd() {
    local cmd="$1"
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        die "${cmd} not found on PATH"
    fi
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ADR_RAW=""
FEATURE=""
BUCKET=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --adr)
            ADR_RAW="${2:-}"
            shift 2
            ;;
        --feature)
            FEATURE="${2:-}"
            shift 2
            ;;
        --bucket)
            BUCKET="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown arg: $1 (use --help)"
            ;;
    esac
done

[[ -n "${ADR_RAW}" ]] || die "missing --adr"

need_cmd git
need_cmd python3

REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

ADR_REL="$(
    python3 - "${REPO_ROOT}" "${ADR_RAW}" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path

repo = Path(sys.argv[1]).resolve()
raw = sys.argv[2]
p = Path(raw)
if p.is_absolute():
    abs_path = p.resolve()
else:
    abs_path = (repo / p).resolve()

try:
    rel = abs_path.relative_to(repo)
except Exception:
    print(f"ERROR: ADR path resolves outside repo root: {raw!r} -> {abs_path}", file=sys.stderr)
    raise SystemExit(2)

print(rel.as_posix())
PY
)" || exit $?

[[ -f "${REPO_ROOT}/${ADR_REL}" ]] || die "ADR does not exist: ${ADR_RAW} (resolved to ${ADR_REL})"

if [[ -z "${FEATURE}" ]]; then
    adr_base="$(basename "${ADR_REL}")"
    adr_stem="${adr_base%.md}"
    if [[ "${adr_stem}" =~ ^ADR-[0-9]{4}-(.+)$ ]]; then
        FEATURE="${BASH_REMATCH[1]}"
    else
        die "unable to derive feature slug from ADR filename (${adr_base}); pass --feature"
    fi
fi

if [[ -z "${FEATURE}" ]]; then
    die "--feature must not be empty"
fi
if [[ "${FEATURE}" == *"/"* ]]; then
    die "invalid --feature: must not contain '/' (${FEATURE})"
fi
if ! [[ "${FEATURE}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
    die "invalid --feature: ${FEATURE} (expected [A-Za-z0-9][A-Za-z0-9._-]*)"
fi

if [[ -z "${BUCKET}" ]]; then
    BUCKET="${PM_DEFAULT_PACK_BUCKET:-}"
fi
if [[ -z "${BUCKET}" ]]; then
    BUCKET="draft"
fi
if [[ "${BUCKET}" == *"/"* ]]; then
    die "invalid --bucket: must not contain '/' (${BUCKET})"
fi
if ! [[ "${BUCKET}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
    die "invalid --bucket: ${BUCKET} (expected [A-Za-z0-9][A-Za-z0-9._-]*)"
fi

pm_roots_json="$(python3 "${SCRIPT_DIR}/pm_paths.py" print-roots 2>/dev/null)" || {
    die "failed to resolve PM roots (pm_paths.py print-roots)"
}
PM_PACKS_ROOT="$(
    python3 -c 'import json,sys; print(json.load(sys.stdin)["pm_packs_root"])' <<<"${pm_roots_json}" 2>/dev/null || true
)"
if [[ -z "${PM_PACKS_ROOT}" ]]; then
    die "pm_paths.py print-roots returned empty pm_packs_root"
fi
PM_PACKS_ROOT="${PM_PACKS_ROOT%/}"

BUCKET_DIR_ABS="${REPO_ROOT}/${PM_PACKS_ROOT}/${BUCKET}"
if [[ ! -d "${BUCKET_DIR_ABS}" ]]; then
    existing="$(
        find "${REPO_ROOT}/${PM_PACKS_ROOT}" -maxdepth 1 -mindepth 1 -type d -exec basename {} \; 2>/dev/null | sort || true
    )"
    if [[ -n "${existing}" ]]; then
        die "unknown pack bucket: ${BUCKET} (expected one of: $(tr '\n' ' ' <<<"${existing}"))"
    fi
    die "unknown pack bucket: ${BUCKET} (no buckets found under ${PM_PACKS_ROOT})"
fi

FEATURE_DIR_REL="${PM_PACKS_ROOT}/${BUCKET}/${FEATURE}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
mkdir -p "${FEATURE_DIR_ABS}"

TASKS_JSON_ABS="${FEATURE_DIR_ABS}/tasks.json"

python3 - "${TASKS_JSON_ABS}" "${FEATURE}" "${ADR_REL}" <<'PY'
from __future__ import annotations

import json
import os
import sys
from typing import Any, Dict

path = sys.argv[1]
feature = sys.argv[2]
adr_rel = sys.argv[3]


def _die(msg: str) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(2)


def _write_json(dst: str, data: Dict[str, Any]) -> None:
    tmp = f"{dst}.tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        json.dump(data, handle, indent=2, sort_keys=False)
        handle.write("\n")
    os.replace(tmp, dst)


if not os.path.exists(path):
    data: Dict[str, Any] = {
        "meta": {
            "schema_version": 4,
            "cross_platform": True,
            "ci_parity_platforms_required": ["linux", "macos", "windows"],
            "behavior_platforms_required": ["linux", "macos", "windows"],
            "automation": {"enabled": True, "orchestration_branch": f"feat/{feature}"},
            "slice_spec_version": 2,
            "feature": feature,
            "adr_paths": [adr_rel],
        },
        "tasks": [],
    }
    _write_json(path, data)
    raise SystemExit(0)

try:
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
except Exception as exc:
    _die(f"failed to parse tasks.json as JSON: {path} ({exc})")

if not isinstance(data, dict):
    _die(f"tasks.json must be a JSON object: {path}")

changed = False
meta = data.get("meta")
if meta is None:
    meta = {}
    data["meta"] = meta
    changed = True
if not isinstance(meta, dict):
    _die(f"tasks.json meta must be an object when present: {path}")

slice_spec_version = meta.get("slice_spec_version")
if slice_spec_version is None:
    meta["slice_spec_version"] = 2
    changed = True
elif not isinstance(slice_spec_version, int):
    _die(f"tasks.json meta.slice_spec_version must be an integer when present: {path}")
elif slice_spec_version < 2:
    meta["slice_spec_version"] = 2
    changed = True

schema_version = meta.get("schema_version")
if schema_version is None:
    meta["schema_version"] = 4
    changed = True
elif not isinstance(schema_version, int):
    _die(f"tasks.json meta.schema_version must be an integer when present: {path}")
elif schema_version < 4:
    meta["schema_version"] = 4
    changed = True

cross_platform = meta.get("cross_platform")
if cross_platform is None:
    meta["cross_platform"] = True
    changed = True
elif not isinstance(cross_platform, bool):
    _die(f"tasks.json meta.cross_platform must be a boolean when present: {path}")

ci_parity = meta.get("ci_parity_platforms_required")
if ci_parity is None:
    meta["ci_parity_platforms_required"] = ["linux", "macos", "windows"]
    changed = True

behavior = meta.get("behavior_platforms_required")
if behavior is None:
    meta["behavior_platforms_required"] = ["linux", "macos", "windows"]
    changed = True

automation = meta.get("automation")
if automation is None:
    meta["automation"] = {"enabled": True, "orchestration_branch": f"feat/{feature}"}
    changed = True
elif not isinstance(automation, dict):
    _die(f"tasks.json meta.automation must be an object when present: {path}")
else:
    enabled = automation.get("enabled")
    if enabled is None:
        automation["enabled"] = True
        changed = True
    elif not isinstance(enabled, bool):
        _die(f"tasks.json meta.automation.enabled must be a boolean when present: {path}")
    elif enabled is not True:
        automation["enabled"] = True
        changed = True

    orchestration_branch = automation.get("orchestration_branch")
    if orchestration_branch is None:
        automation["orchestration_branch"] = f"feat/{feature}"
        changed = True
    elif not isinstance(orchestration_branch, str):
        _die(f"tasks.json meta.automation.orchestration_branch must be a string when present: {path}")
    elif orchestration_branch.strip() == "":
        automation["orchestration_branch"] = f"feat/{feature}"
        changed = True

adr_paths = meta.get("adr_paths")
if adr_paths is None:
    meta["adr_paths"] = [adr_rel]
    changed = True
else:
    if not isinstance(adr_paths, list) or not all(isinstance(x, str) for x in adr_paths):
        _die(f"tasks.json meta.adr_paths must be an array of strings when present: {path}")
    if adr_rel not in adr_paths:
        adr_paths.append(adr_rel)
        changed = True

if changed:
    _write_json(path, data)
PY

printf '%s\n' "${FEATURE_DIR_REL}"
