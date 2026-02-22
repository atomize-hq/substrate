#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


PM_LIFT_BEGIN = "<!-- PM_LIFT_VECTOR:BEGIN -->"
PM_LIFT_END = "<!-- PM_LIFT_VECTOR:END -->"

# Directory-prefix expansion (lift-only; does not rewrite impact_map.md).
EXPAND_DISCOUNT = 0.20
EXPAND_CAP = 10


@dataclass(frozen=True)
class LiftResult:
    model_version: int
    lift_score: int
    estimated_slices: int
    confidence: str
    triggers: list[str]
    missing_inputs: list[str]
    vector: dict[str, Any]
    derived: dict[str, Any]


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _die(msg: str, code: int = 1) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(code)


def _repo_root() -> Path:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as e:
        _die(f"git rev-parse failed (not in repo?): {e}")
    if not out:
        _die("git rev-parse returned empty repo root")
    return Path(out).resolve()


def _load_model_config(repo_root: Path) -> dict[str, Any] | None:
    """
    Optional (future) model config:
      docs/project_management/system/schemas/work_lift_model.v1.json

    If missing, the script uses baked-in defaults for the initial sketch.
    """
    p = repo_root / "docs/project_management/system/schemas/work_lift_model.v1.json"
    if not p.exists():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception as e:
        _die(f"failed to parse model config JSON: {p}: {e}")
    return None


def _extract_lift_json_block(text: str, path: Path) -> dict[str, Any]:
    if PM_LIFT_BEGIN not in text or PM_LIFT_END not in text:
        _die(f"missing lift markers in {path} (expected {PM_LIFT_BEGIN} ... {PM_LIFT_END})")
    start = text.index(PM_LIFT_BEGIN) + len(PM_LIFT_BEGIN)
    end = text.index(PM_LIFT_END, start)
    region = text[start:end]

    # Extract first ```json fenced block within the region.
    m = re.search(r"```json\s*(\{.*?\})\s*```", region, flags=re.DOTALL)
    if not m:
        _die(f"lift markers present but no ```json {{...}}``` block found in {path}")
    raw = m.group(1)
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as e:
        _die(f"invalid lift JSON block in {path}: {e}")
    if not isinstance(data, dict):
        _die(f"lift JSON block must be an object: {path}")
    return data


def _num(value: Any, field: str, missing: list[str]) -> float:
    if value is None:
        missing.append(field)
        return 0.0
    if isinstance(value, bool):
        _die(f"{field} must be a number, got bool")
    if isinstance(value, (int, float)):
        return float(value)
    _die(f"{field} must be a number or null, got {type(value).__name__}")
    return 0.0


def _bool(value: Any, field: str) -> bool:
    if value is None:
        return False
    if isinstance(value, bool):
        return value
    _die(f"{field} must be a boolean, got {type(value).__name__}")
    return False


def _compute_lift(vector: dict[str, Any], *, model_config: dict[str, Any] | None = None) -> LiftResult:
    # Baked-in Lift Score v1 (canonical until moved into model config).
    model_version = int(vector.get("model_version") or 1)

    touch = vector.get("touch") or {}
    contract = vector.get("contract") or {}
    qa = vector.get("qa") or {}
    docs = vector.get("docs") or {}
    ops = vector.get("ops") or {}
    risk = vector.get("risk") or {}

    if not all(isinstance(x, dict) for x in (touch, contract, qa, docs, ops, risk)):
        _die("lift vector sections touch/contract/qa/docs/ops/risk must be objects")

    missing: list[str] = []

    create_files = _num(touch.get("create_files"), "touch.create_files", missing)
    edit_files = _num(touch.get("edit_files"), "touch.edit_files", missing)
    delete_files = _num(touch.get("delete_files"), "touch.delete_files", missing)
    deprecate_files = _num(touch.get("deprecate_files"), "touch.deprecate_files", missing)
    crates_touched = _num(touch.get("crates_touched"), "touch.crates_touched", missing)
    boundary_crossings = _num(touch.get("boundary_crossings"), "touch.boundary_crossings", missing)

    cli_flags = _num(contract.get("cli_flags"), "contract.cli_flags", missing)
    config_keys = _num(contract.get("config_keys"), "contract.config_keys", missing)
    exit_codes = _num(contract.get("exit_codes"), "contract.exit_codes", missing)
    file_formats = _num(contract.get("file_formats"), "contract.file_formats", missing)
    behavior_deltas = _num(contract.get("behavior_deltas"), "contract.behavior_deltas", missing)

    new_test_files = _num(qa.get("new_test_files"), "qa.new_test_files", missing)
    new_test_cases = _num(qa.get("new_test_cases"), "qa.new_test_cases", missing)
    new_docs_files = _num(docs.get("new_docs_files"), "docs.new_docs_files", missing)
    new_smoke_steps = _num(ops.get("new_smoke_steps"), "ops.new_smoke_steps", missing)
    ci_changes = _num(ops.get("ci_changes"), "ops.ci_changes", missing)

    cross_platform = _bool(risk.get("cross_platform"), "risk.cross_platform")
    security_sensitive = _bool(risk.get("security_sensitive"), "risk.security_sensitive")
    concurrency_or_ordering = _bool(risk.get("concurrency_or_ordering"), "risk.concurrency_or_ordering")
    migration_or_backfill = _bool(risk.get("migration_or_backfill"), "risk.migration_or_backfill")
    unknowns_high = _num(risk.get("unknowns_high"), "risk.unknowns_high", missing)

    base = 0.0
    base += 3.0 * create_files
    base += 2.0 * edit_files
    base += 1.0 * delete_files
    base += 1.0 * deprecate_files
    base += 4.0 * crates_touched
    base += 3.0 * boundary_crossings

    base += 3.0 * cli_flags
    base += 3.0 * config_keys
    base += 4.0 * exit_codes
    base += 5.0 * file_formats
    base += 10.0 * max(0.0, behavior_deltas - 1.0)

    base += 2.0 * new_test_files
    base += 1.0 * new_test_cases
    base += 2.0 * new_docs_files
    base += 3.0 * new_smoke_steps
    base += 3.0 * ci_changes

    m = 1.0
    if cross_platform:
        m *= 1.15
    if security_sensitive:
        m *= 1.20
    if concurrency_or_ordering:
        m *= 1.15
    if migration_or_backfill:
        m *= 1.25

    score = math.ceil(base * m + (2.0 * unknowns_high))
    estimated_slices = max(1, math.ceil(score / 12.0))

    triggers: list[str] = []

    if behavior_deltas > 1.0:
        triggers.append("split_required:behavior_deltas>1")
    if crates_touched > 2.0:
        triggers.append("likely_split:crates_touched>2")
    if (create_files + edit_files + delete_files) > 12.0:
        triggers.append("likely_split:touch_files_sum>12")
    if (cli_flags + config_keys + exit_codes + file_formats) > 4.0:
        triggers.append("likely_split:contract_surface_sum>4")
    if score > 24:
        triggers.append("likely_split:lift_score>24")
    if estimated_slices > 3:
        triggers.append("split_required:estimated_slices>3")

    confidence = "high"
    if missing:
        confidence = "low"
        for f in missing:
            triggers.append(f"missing_inputs:{f}")

    derived = {
        "base_points": base,
        "risk_multiplier": m,
    }

    return LiftResult(
        model_version=model_version,
        lift_score=int(score),
        estimated_slices=int(estimated_slices),
        confidence=confidence,
        triggers=sorted(set(triggers)),
        missing_inputs=sorted(set(missing)),
        vector=vector,
        derived=derived,
    )


def _run_validate_impact_map_emit_json(feature_dir: Path) -> dict[str, Any]:
    script_dir = Path(__file__).resolve().parent
    validator = script_dir / "validate_impact_map.py"
    cmd = [sys.executable, str(validator), "--feature-dir", str(feature_dir), "--emit-json"]
    try:
        res = subprocess.run(cmd, text=True, capture_output=True, check=False)
    except Exception as e:
        _die(f"failed to run validate_impact_map.py: {e}")
    if res.returncode != 0:
        _die(f"validate_impact_map.py failed:\n{res.stderr.strip() or res.stdout.strip()}")
    try:
        data = json.loads(res.stdout)
    except json.JSONDecodeError as e:
        # validate_impact_map.py prints warnings to stderr; stdout should be JSON-only.
        _die(f"validate_impact_map.py did not return JSON on stdout: {e}")
    if not isinstance(data, dict):
        _die("validate_impact_map.py JSON must be an object")
    return data


def _expand_prefix(repo_root: Path, prefix: str) -> list[str]:
    prefix = prefix.strip()
    if not prefix.endswith("/"):
        return []
    try:
        out = subprocess.check_output(
            ["git", "-C", str(repo_root), "ls-files", prefix],
            text=True,
            stderr=subprocess.STDOUT,
        )
    except Exception:
        return []
    files = [line.strip() for line in out.splitlines() if line.strip()]
    return [f for f in files if f.startswith(prefix)]


def _infer_crates_touched(paths: list[str]) -> int:
    crates: set[str] = set()
    for p in paths:
        parts = p.split("/")
        if len(parts) >= 2 and parts[0] == "crates":
            crates.add(parts[1])
    return len(crates)


def cmd_from_intake(intake_path: Path, emit_json: bool) -> int:
    text = intake_path.read_text(encoding="utf-8")
    vector = _extract_lift_json_block(text, intake_path)
    repo_root = _repo_root()
    model_cfg = _load_model_config(repo_root)
    res = _compute_lift(vector, model_config=model_cfg)
    return _print_result(res, emit_json=emit_json)


def cmd_from_impact_map(feature_dir: Path, emit_json: bool) -> int:
    repo_root = _repo_root()
    allow = _run_validate_impact_map_emit_json(feature_dir)

    per_section = {}
    expanded_paths_all: list[str] = []
    raw_paths_all: list[str] = []

    for sec in ("create", "edit", "deprecate", "delete"):
        items = allow.get(sec) or []
        if not isinstance(items, list) or not all(isinstance(x, str) for x in items):
            _die(f"validate_impact_map.py JSON field {sec!r} must be an array of strings")

        explicit_files = [p for p in items if not p.endswith("/")]
        prefixes = [p for p in items if p.endswith("/")]

        # Raw count policy: prefix counts as 1 file for the vector.
        raw_count = len(explicit_files) + len(prefixes)

        # Effective count policy (lift-only): per-prefix discounted/capped expansion.
        eff = float(len(explicit_files))
        expansions: dict[str, int] = {}
        for pref in prefixes:
            expanded = _expand_prefix(repo_root, pref)
            expansions[pref] = len(expanded)
            expanded_paths_all.extend(expanded)
            eff += min(len(expanded), EXPAND_CAP) * EXPAND_DISCOUNT

        per_section[sec] = {
            "explicit_files": len(explicit_files),
            "prefix_entries": len(prefixes),
            "prefix_expanded_counts": expansions,
            "raw_count": raw_count,
            "effective_count": eff,
        }

        raw_paths_all.extend(items)

    # Construct a minimal vector for scoring. Contract/QA/etc. are unknown from impact_map alone.
    touch = {
        "create_files": int(per_section["create"]["raw_count"]),
        "edit_files": int(per_section["edit"]["raw_count"]),
        "delete_files": int(per_section["delete"]["raw_count"]),
        "deprecate_files": int(per_section["deprecate"]["raw_count"]),
        "crates_touched": _infer_crates_touched(raw_paths_all + expanded_paths_all),
        "boundary_crossings": None,
    }
    vector = {
        "model_version": 1,
        "touch": touch,
        "contract": {
            "cli_flags": None,
            "config_keys": None,
            "exit_codes": None,
            "file_formats": None,
            "behavior_deltas": 1,
        },
        "qa": {"new_test_files": None, "new_test_cases": None},
        "docs": {"new_docs_files": None},
        "ops": {"new_smoke_steps": None, "ci_changes": None},
        "risk": {
            "cross_platform": False,
            "security_sensitive": False,
            "concurrency_or_ordering": False,
            "migration_or_backfill": False,
            "unknowns_high": None,
        },
        "notes": "",
    }

    model_cfg = _load_model_config(repo_root)
    res = _compute_lift(vector, model_config=model_cfg)
    derived = dict(res.derived)
    derived["impact_map_touch_counts"] = per_section
    derived["touch_effective_for_scoring"] = {
        "create_files": per_section["create"]["effective_count"],
        "edit_files": per_section["edit"]["effective_count"],
        "deprecate_files": per_section["deprecate"]["effective_count"],
        "delete_files": per_section["delete"]["effective_count"],
    }
    res = LiftResult(
        model_version=res.model_version,
        lift_score=res.lift_score,
        estimated_slices=res.estimated_slices,
        confidence="low" if allow.get("dir_prefixes") else res.confidence,
        triggers=sorted(set(res.triggers + (["touch_set_contains_prefix_entries"] if allow.get("dir_prefixes") else []))),
        missing_inputs=res.missing_inputs,
        vector=res.vector,
        derived=derived,
    )
    return _print_result(res, emit_json=emit_json)


def cmd_from_git_diff(git_range: str, emit_json: bool) -> int:
    repo_root = _repo_root()
    try:
        out = subprocess.check_output(
            ["git", "-C", str(repo_root), "diff", "--name-status", "-M", git_range],
            text=True,
            stderr=subprocess.STDOUT,
        )
    except Exception as e:
        _die(f"git diff failed for range {git_range!r}: {e}")

    create = 0
    edit = 0
    delete = 0
    for line in out.splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split("\t")
        status = parts[0]
        if status.startswith("A"):
            create += 1
        elif status.startswith("D"):
            delete += 1
        elif status.startswith("M") or status.startswith("R") or status.startswith("C"):
            edit += 1
        else:
            edit += 1

    vector = {
        "model_version": 1,
        "touch": {
            "create_files": create,
            "edit_files": edit,
            "delete_files": delete,
            "deprecate_files": 0,
            "crates_touched": None,
            "boundary_crossings": None,
        },
        "contract": {
            "cli_flags": None,
            "config_keys": None,
            "exit_codes": None,
            "file_formats": None,
            "behavior_deltas": 1,
        },
        "qa": {"new_test_files": None, "new_test_cases": None},
        "docs": {"new_docs_files": None},
        "ops": {"new_smoke_steps": None, "ci_changes": None},
        "risk": {
            "cross_platform": False,
            "security_sensitive": False,
            "concurrency_or_ordering": False,
            "migration_or_backfill": False,
            "unknowns_high": None,
        },
        "notes": "",
    }

    model_cfg = _load_model_config(repo_root)
    res = _compute_lift(vector, model_config=model_cfg)
    return _print_result(res, emit_json=emit_json)


def _print_result(res: LiftResult, *, emit_json: bool) -> int:
    if emit_json:
        out = {
            "model_version": res.model_version,
            "lift_score": res.lift_score,
            "estimated_slices": res.estimated_slices,
            "confidence": res.confidence,
            "triggers": res.triggers,
            "missing_inputs": res.missing_inputs,
            "vector": res.vector,
            "derived": res.derived,
        }
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0

    print(f"Lift Score (v{res.model_version}): {res.lift_score}")
    print(f"Estimated slices: {res.estimated_slices}")
    print(f"Confidence: {res.confidence}")
    if res.triggers:
        print("Triggers:")
        for t in res.triggers:
            print(f"- {t}")
    if res.missing_inputs:
        print("Missing inputs:")
        for m in res.missing_inputs:
            print(f"- {m}")
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Compute time-free Work Lift score + split triggers.")
    sub = ap.add_subparsers(dest="cmd", required=True)

    ap_intake = sub.add_parser("from-intake", help="Compute lift from a markdown intake/ADR containing a PM_LIFT_VECTOR JSON block.")
    ap_intake.add_argument("--intake", required=True, help="Path to intake/ADR markdown file.")
    ap_intake.add_argument("--emit-json", action="store_true", help="Emit machine JSON summary to stdout.")

    ap_impact = sub.add_parser("from-impact-map", help="Compute lift from a Planning Pack impact_map.md touch set.")
    ap_impact.add_argument("--feature-dir", required=True, help="Planning Pack dir (docs/project_management/packs/<bucket>/<feature>).")
    ap_impact.add_argument("--emit-json", action="store_true", help="Emit machine JSON summary to stdout.")

    ap_diff = sub.add_parser("from-git-diff", help="Compute lift from a git diff range (for calibration).")
    ap_diff.add_argument("--git-range", required=True, help="Git range (e.g. base..head).")
    ap_diff.add_argument("--emit-json", action="store_true", help="Emit machine JSON summary to stdout.")

    args = ap.parse_args(argv)

    if args.cmd == "from-intake":
        return cmd_from_intake(Path(args.intake), emit_json=bool(args.emit_json))
    if args.cmd == "from-impact-map":
        return cmd_from_impact_map(Path(args.feature_dir), emit_json=bool(args.emit_json))
    if args.cmd == "from-git-diff":
        return cmd_from_git_diff(str(args.git_range), emit_json=bool(args.emit_json))

    _die(f"unknown command: {args.cmd}", code=2)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
