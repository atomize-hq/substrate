#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import validate_execution_touchset_coherence as vetc


SAFE_EXACT = {
    "pre-planning/impact_map.md",
    "plan.md",
    "tasks.json",
    "manual_testing_playbook.md",
    "execution_preflight_report.md",
}

HARD_FAIL_EXACT = {
    "contract.md",
    "decision_register.md",
    "pre-planning/workstream_triage.md",
    "pre-planning/minimal_spec_draft.md",
    "pre-planning/spec_manifest.md",
    "pre-planning/ci_checkpoint_plan.md",
}


@dataclass(frozen=True)
class ConvergenceIssue:
    validator: str
    source_name: str
    path: str | None
    message: str
    remediation: str


def _fail(msg: str) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(2)


def _safe_path(rel_path: str) -> bool:
    if rel_path in SAFE_EXACT:
        return True
    if rel_path.startswith("kickoff_prompts/"):
        return True
    parts = Path(rel_path).parts
    if len(parts) >= 3 and parts[0] == "slices" and parts[2] == "kickoff_prompts":
        return True
    if len(parts) == 3 and parts[0] == "slices" and parts[2].endswith("-closeout_report.md"):
        return True
    return False


def _hard_fail_path(rel_path: str) -> bool:
    if rel_path in HARD_FAIL_EXACT:
        return True
    parts = Path(rel_path).parts
    if len(parts) == 3 and parts[0] == "slices" and parts[2].endswith("-spec.md"):
        return True
    return False


def _extract_path_mentions(text: str, feature_dir: Path, feature_dir_rel: str) -> list[str]:
    paths: list[str] = []
    seen: set[str] = set()

    abs_prefix = str(feature_dir.resolve())
    for token in text.split():
        cleaned = token.strip("`\"'(),:;[]{}")
        if cleaned.startswith(abs_prefix):
            try:
                rel = Path(cleaned).resolve().relative_to(feature_dir.resolve()).as_posix()
            except Exception:
                continue
            if rel not in seen:
                seen.add(rel)
                paths.append(rel)
            continue

        if cleaned.startswith(feature_dir_rel.rstrip("/") + "/"):
            rel = cleaned[len(feature_dir_rel.rstrip("/")) + 1 :].split("#", 1)[0].rstrip(".,:;)]}")
            if rel and rel not in seen:
                seen.add(rel)
                paths.append(rel)

    return paths


def _message_from_output(output: str) -> str:
    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("FAIL:") or stripped.startswith("ERROR:"):
            return stripped
    for line in output.splitlines():
        stripped = line.strip()
        if stripped:
            return stripped
    return "validator failed without a message"


def _run_validator(name: str, argv: list[str], feature_dir: Path, feature_dir_rel: str) -> list[ConvergenceIssue]:
    res = subprocess.run(argv, text=True, capture_output=True, check=False)
    if res.returncode == 0:
        return []

    output = "\n".join(part for part in (res.stderr.strip(), res.stdout.strip()) if part).strip()
    message = _message_from_output(output)
    mentioned_paths = _extract_path_mentions(output, feature_dir, feature_dir_rel)

    if not mentioned_paths:
        return [
            ConvergenceIssue(
                validator=name,
                source_name=name,
                path=None,
                message=message,
                remediation="hard_fail",
            )
        ]

    remediation = "safe" if all(_safe_path(p) for p in mentioned_paths) else "hard_fail"
    if any(_hard_fail_path(p) for p in mentioned_paths):
        remediation = "hard_fail"

    issues: list[ConvergenceIssue] = []
    for rel_path in mentioned_paths:
        issues.append(
            ConvergenceIssue(
                validator=name,
                source_name=name,
                path=rel_path,
                message=message,
                remediation="safe" if remediation == "safe" and _safe_path(rel_path) else "hard_fail",
            )
        )
    return issues


def _ci_checkpoint_applicable(feature_dir: Path) -> bool:
    return (feature_dir / "pre-planning" / "ci_checkpoint_plan.md").exists() or (feature_dir / "ci_checkpoint_plan.md").exists()


def inspect_post_full_planning(feature_dir: Path) -> list[ConvergenceIssue]:
    feature_dir = feature_dir.resolve()
    repo_root = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
    feature_dir_rel = feature_dir.relative_to(Path(repo_root).resolve()).as_posix()
    scripts_dir = Path(__file__).resolve().parent

    issues: list[ConvergenceIssue] = []
    validators = [
        (
            "validate_tasks_json.py",
            [sys.executable, str(scripts_dir / "validate_tasks_json.py"), "--feature-dir", str(feature_dir)],
        ),
        (
            "validate_slice_inventory_coherence.py",
            [
                sys.executable,
                str(scripts_dir / "validate_slice_inventory_coherence.py"),
                "--feature-dir",
                str(feature_dir),
                "--phase",
                "execution_ready",
            ],
        ),
        (
            "validate_slice_specs.py",
            [sys.executable, str(scripts_dir / "validate_slice_specs.py"), "--feature-dir", str(feature_dir)],
        ),
        (
            "validate_impact_map.py",
            [sys.executable, str(scripts_dir / "validate_impact_map.py"), "--feature-dir", str(feature_dir)],
        ),
    ]
    if _ci_checkpoint_applicable(feature_dir):
        validators.append(
            (
                "validate_ci_checkpoint_plan.py",
                [sys.executable, str(scripts_dir / "validate_ci_checkpoint_plan.py"), "--feature-dir", str(feature_dir)],
            )
        )

    for name, argv in validators:
        issues.extend(_run_validator(name, argv, feature_dir, feature_dir_rel))

    for issue in vetc.inspect_feature_dir(feature_dir):
        issues.append(
            ConvergenceIssue(
                validator="validate_execution_touchset_coherence.py",
                source_name=issue.source_name,
                path=issue.source_path.relative_to(feature_dir).as_posix() if issue.source_path is not None else None,
                message=issue.message,
                remediation=issue.remediation,
            )
        )

    if not issues:
        issues.extend(
            _run_validator(
                "planning-lint",
                ["make", "planning-lint", f"FEATURE_DIR={feature_dir_rel}"],
                feature_dir,
                feature_dir_rel,
            )
        )

    return issues


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Classify post-full-planning execution-readiness convergence.")
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    args = ap.parse_args(argv)

    feature_dir = Path(args.feature_dir).resolve()
    if not feature_dir.exists() or not feature_dir.is_dir():
        _fail(f"feature dir does not exist: {feature_dir}")

    issues = inspect_post_full_planning(feature_dir)
    stale_docs: set[str] = set()
    remediation_allowed = bool(issues)
    for issue in issues:
        if issue.remediation != "safe":
            remediation_allowed = False
        if issue.path is not None and _safe_path(issue.path):
            stale_docs.add(issue.path)
    if any(issue.validator == "validate_execution_touchset_coherence.py" for issue in issues):
        stale_docs.add("pre-planning/impact_map.md")

    if not issues:
        status = "pass"
    elif remediation_allowed:
        status = "needs_remediation"
    else:
        status = "hard_fail"

    print(
        json.dumps(
            {
                "status": status,
                "stale_docs": sorted(stale_docs),
                "remediation_allowed": remediation_allowed,
                "issues": [
                    {
                        "validator": issue.validator,
                        "source": issue.source_name,
                        "path": issue.path,
                        "message": issue.message,
                        "remediation": issue.remediation,
                    }
                    for issue in issues
                ],
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
