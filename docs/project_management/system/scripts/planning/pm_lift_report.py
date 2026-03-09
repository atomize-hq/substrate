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


def _die(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(1)


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


def _read_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        _usage_error(f"missing required path: {path}")
    except json.JSONDecodeError as e:
        _usage_error(f"invalid JSON: {path}: {e}")
    return {}


def _pack_is_strict(pack_dir: Path) -> bool:
    tasks = _read_json(pack_dir / "tasks.json")
    meta = tasks.get("meta")
    if isinstance(meta, dict):
        v = meta.get("slice_spec_version")
        return isinstance(v, int) and v >= 2
    return False


def _run_pm_lift(repo_root: Path, pm_lift: Path, args: list[str]) -> dict[str, Any]:
    cmd = [sys.executable, str(pm_lift), *args, "--emit-json"]
    res = subprocess.run(
        cmd,
        text=True,
        capture_output=True,
        check=False,
        cwd=str(repo_root),
    )
    if res.returncode != 0:
        detail = res.stderr.strip() or res.stdout.strip()
        _die(f"pm_lift failed (exit={res.returncode}):\n{detail}")

    try:
        out = json.loads(res.stdout)
    except Exception as e:
        _die(f"pm_lift returned non-JSON stdout: {e}")

    if not isinstance(out, dict):
        _die("pm_lift JSON stdout must be an object at the top level")
    for k in REQUIRED_CONTRACT3_KEYS:
        if k not in out:
            _die(f"pm_lift JSON missing required key: {k!r}")
    return out


def _render_report(
    *,
    context: str,
    out: dict[str, Any],
    max_triggers: int,
    max_missing_inputs: int,
) -> str:
    model_version = out["model_version"]
    lift_score = out["lift_score"]
    estimated_slices = out["estimated_slices"]
    confidence = out["confidence"]

    triggers = out.get("triggers") or []
    if not isinstance(triggers, list) or not all(isinstance(x, str) for x in triggers):
        _die("pm_lift JSON triggers must be an array<string>")
    visible_triggers = [t for t in triggers if not t.startswith("missing_inputs:")]

    missing_inputs = out.get("missing_inputs") or []
    if not isinstance(missing_inputs, list) or not all(isinstance(x, str) for x in missing_inputs):
        _die("pm_lift JSON missing_inputs must be an array<string>")

    lines: list[str] = []
    lines.append("== Work Lift (advisory) ==")
    lines.append(f"Context: {context}")
    lines.append(f"Model: v{model_version}")
    lines.append(f"Lift Score: {lift_score}")
    lines.append(f"Estimated slices: {estimated_slices}")
    lines.append(f"Confidence: {confidence}")

    if visible_triggers:
        lines.append(f"Triggers (top {max_triggers}):")
        shown = visible_triggers[:max_triggers]
        for t in shown:
            lines.append(f"- {t}")
        remaining = len(visible_triggers) - len(shown)
        if remaining > 0:
            lines.append(f"(+{remaining} more)")
    else:
        lines.append("Triggers: none")

    if missing_inputs:
        lines.append(f"Missing inputs (top {max_missing_inputs}):")
        shown = missing_inputs[:max_missing_inputs]
        for m in shown:
            lines.append(f"- {m}")
        remaining = len(missing_inputs) - len(shown)
        if remaining > 0:
            lines.append(f"(+{remaining} more)")
    else:
        lines.append("Missing inputs: none")

    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Print an advisory Work Lift report (wrapper around pm_lift --emit-json).")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--intake", help="Path to intake/ADR markdown file.")
    g.add_argument("--feature-dir", help="Planning pack directory under docs/project_management/packs/<bucket>/<feature>.")
    g.add_argument("--git-range", help="Git range (e.g. base..head) for calibration.")
    ap.add_argument("--max-triggers", type=int, default=5, help="Max number of non-missing triggers to display.")
    ap.add_argument("--max-missing-inputs", type=int, default=8, help="Max number of missing inputs to display.")
    args = ap.parse_args(argv)

    if args.max_triggers <= 0:
        _usage_error("--max-triggers must be >= 1")
    if args.max_missing_inputs <= 0:
        _usage_error("--max-missing-inputs must be >= 1")

    repo_root = _repo_root()
    script_dir = Path(__file__).resolve().parent
    pm_lift = script_dir / "pm_lift.py"
    if not pm_lift.is_file():
        _die(f"pm_lift.py not found at expected path: {pm_lift}")

    if args.intake:
        out = _run_pm_lift(repo_root, pm_lift, ["from-intake", "--intake", str(args.intake)])
        sys.stdout.write(
            _render_report(
                context="intake",
                out=out,
                max_triggers=int(args.max_triggers),
                max_missing_inputs=int(args.max_missing_inputs),
            )
        )
        return 0

    if args.feature_dir:
        pack_dir = Path(args.feature_dir)
        if not pack_dir.exists() or not pack_dir.is_dir():
            _usage_error(f"--feature-dir must be an existing directory: {pack_dir}")
        if not _pack_is_strict(pack_dir):
            print("SKIP: Work Lift advisory report (legacy pack; meta.slice_spec_version < 2)")
            return 0
        out = _run_pm_lift(repo_root, pm_lift, ["from-impact-map", "--feature-dir", str(pack_dir)])
        sys.stdout.write(
            _render_report(
                context="pack",
                out=out,
                max_triggers=int(args.max_triggers),
                max_missing_inputs=int(args.max_missing_inputs),
            )
        )
        return 0

    if args.git_range:
        out = _run_pm_lift(repo_root, pm_lift, ["from-git-diff", "--git-range", str(args.git_range)])
        sys.stdout.write(
            _render_report(
                context="git-diff",
                out=out,
                max_triggers=int(args.max_triggers),
                max_missing_inputs=int(args.max_missing_inputs),
            )
        )
        return 0

    _usage_error("no context selected")
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

