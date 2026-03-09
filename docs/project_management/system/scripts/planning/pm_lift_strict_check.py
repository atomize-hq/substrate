#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


REQUIRED_CONTRACT3_KEYS = [
    "model_version",
    "lift_score",
    "estimated_slices",
    "confidence",
    "triggers",
    "missing_inputs",
    "vector",
    "derived",
]


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _usage_error(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def _tool_error(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def _invariant_fail(msg: str) -> None:
    _eprint(f"FAIL: {msg}")
    raise SystemExit(1)


def _repo_root() -> Path:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as e:
        _tool_error(f"git rev-parse failed (not in repo?): {e}")
    if not out:
        _tool_error("git rev-parse returned empty repo root")
    return Path(out).resolve()


def _read_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        _usage_error(f"missing required path: {path}")
    except json.JSONDecodeError as e:
        _usage_error(f"invalid JSON: {path}: {e}")
    return {}


def _pack_is_eligible(pack_dir: Path) -> bool:
    tasks = _read_json(pack_dir / "tasks.json")
    meta = tasks.get("meta")
    if isinstance(meta, dict):
        v = meta.get("slice_spec_version")
        return isinstance(v, int) and v >= 2
    return False


def _run_json_subprocess(*, repo_root: Path, argv: list[str]) -> dict[str, Any]:
    res = subprocess.run(
        argv,
        text=True,
        capture_output=True,
        check=False,
        cwd=str(repo_root),
    )
    if res.returncode != 0:
        detail = res.stderr.strip() or res.stdout.strip()
        _tool_error(f"subprocess failed (exit={res.returncode}):\n{detail}")
    try:
        out = json.loads(res.stdout)
    except Exception as e:
        _tool_error(f"subprocess returned non-JSON stdout: {e}")
    if not isinstance(out, dict):
        _tool_error("subprocess JSON stdout must be an object at the top level")
    return out


def _run_pm_lift_emit_json(*, repo_root: Path, pm_lift: Path, args: list[str]) -> dict[str, Any]:
    out = _run_json_subprocess(
        repo_root=repo_root,
        argv=[sys.executable, str(pm_lift), *args, "--emit-json"],
    )
    for k in REQUIRED_CONTRACT3_KEYS:
        if k not in out:
            _tool_error(f"pm_lift JSON missing required key: {k!r}")
    return out


def _evaluate_intake(out: dict[str, Any]) -> None:
    confidence = out.get("confidence")
    missing_inputs = out.get("missing_inputs")
    estimated_slices = out.get("estimated_slices")
    vector = out.get("vector")

    failures: list[str] = []

    if confidence != "high":
        failures.append(f"confidence must be 'high' (got {confidence!r})")

    if not isinstance(missing_inputs, list) or not all(isinstance(x, str) for x in missing_inputs):
        _tool_error("pm_lift JSON missing_inputs must be an array<string>")
    if len(missing_inputs) != 0:
        failures.append(f"missing_inputs must be empty (got {missing_inputs!r})")

    if not isinstance(vector, dict):
        _tool_error("pm_lift JSON vector must be an object")
    behavior_deltas = None
    contract = vector.get("contract")
    if isinstance(contract, dict):
        behavior_deltas = contract.get("behavior_deltas")
    if behavior_deltas != 1:
        failures.append(f"vector.contract.behavior_deltas must be 1 (got {behavior_deltas!r})")

    if not isinstance(estimated_slices, int):
        _tool_error("pm_lift JSON estimated_slices must be an integer")
    if estimated_slices > 3:
        failures.append(f"estimated_slices must be <= 3 (got {estimated_slices})")

    if failures:
        _invariant_fail("intake strict invariants failed:\n- " + "\n- ".join(failures))


def _evaluate_pack(*, repo_root: Path, validate_impact_map: Path, pack_dir: Path) -> None:
    data = _run_json_subprocess(
        repo_root=repo_root,
        argv=[sys.executable, str(validate_impact_map), "--feature-dir", str(pack_dir), "--emit-json"],
    )
    for k in ("create", "edit", "deprecate", "delete", "dir_prefixes"):
        if k not in data:
            _tool_error(f"validate_impact_map JSON missing required key: {k!r}")
    dir_prefixes = data.get("dir_prefixes")
    if not isinstance(dir_prefixes, list) or not all(isinstance(x, str) for x in dir_prefixes):
        _tool_error("validate_impact_map JSON dir_prefixes must be an array<string>")
    if dir_prefixes:
        _invariant_fail(f"strict pack forbids prefix entries (dir_prefixes must be []): {dir_prefixes!r}")


def main(argv: list[str]) -> int:
    import os

    ap = argparse.ArgumentParser(description="Opt-in strict Work Lift check (wrapper around pm_lift --emit-json).")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--intake", help="Path to intake/ADR markdown file.")
    g.add_argument("--feature-dir", help="Planning pack directory under docs/project_management/packs/<bucket>/<feature>.")
    args = ap.parse_args(argv)

    if str(os.environ.get("PM_LIFT_STRICT", "")).strip() != "1":
        print("SKIP: strict mode disabled (set PM_LIFT_STRICT=1)")
        return 0

    repo_root = _repo_root()
    script_dir = Path(__file__).resolve().parent
    pm_lift = script_dir / "pm_lift.py"
    validate_impact_map = script_dir / "validate_impact_map.py"
    if not pm_lift.is_file():
        _tool_error(f"pm_lift.py not found at expected path: {pm_lift}")
    if not validate_impact_map.is_file():
        _tool_error(f"validate_impact_map.py not found at expected path: {validate_impact_map}")

    if args.intake:
        out = _run_pm_lift_emit_json(repo_root=repo_root, pm_lift=pm_lift, args=["from-intake", "--intake", str(args.intake)])
        _evaluate_intake(out)
        print("OK: strict lift check passed (intake)")
        return 0

    if args.feature_dir:
        pack_dir = Path(args.feature_dir)
        if not pack_dir.exists() or not pack_dir.is_dir():
            _usage_error(f"--feature-dir must be an existing directory: {pack_dir}")
        if not _pack_is_eligible(pack_dir):
            print("NOT ELIGIBLE: legacy pack (meta.slice_spec_version < 2)")
            return 0

        _evaluate_pack(repo_root=repo_root, validate_impact_map=validate_impact_map, pack_dir=pack_dir)
        _run_pm_lift_emit_json(
            repo_root=repo_root,
            pm_lift=pm_lift,
            args=["from-impact-map", "--feature-dir", str(pack_dir)],
        )
        print("OK: strict lift check passed (pack)")
        return 0

    _usage_error("no context selected")
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
