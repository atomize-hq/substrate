#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import validate_slice_inventory_coherence as vsic


SAFE_REMEDIATION_SOURCES = {"spec_manifest", "impact_map", "ci_checkpoint_plan"}


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _fail(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Classify pre-full-planning slice convergence state (stdout JSON-only).",
    )
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    ap.add_argument(
        "--workstream-triage",
        default="pre-planning/workstream_triage.md",
        help="Path to workstream_triage.md (absolute or feature-dir-relative).",
    )
    args = ap.parse_args(argv)

    feature_dir = Path(args.feature_dir).resolve()
    if not feature_dir.exists():
        _fail(f"feature dir does not exist: {feature_dir}")
    if not feature_dir.is_dir():
        _fail(f"feature dir is not a directory: {feature_dir}")

    triage_source, _, issues = vsic.inspect_pre_full_planning(feature_dir, args.workstream_triage)
    accepted_slice_order = triage_source.ordered if triage_source is not None else []

    issue_payload = [
        {
            "source": issue.source_name,
            "path": str(issue.path) if issue.path is not None else None,
            "message": issue.message,
        }
        for issue in issues
    ]

    remediation_allowed = bool(issues) and all(issue.source_name in SAFE_REMEDIATION_SOURCES for issue in issues)
    stale_docs = sorted({issue.source_name for issue in issues if issue.source_name in SAFE_REMEDIATION_SOURCES})

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
                "accepted_slice_order": accepted_slice_order,
                "stale_docs": stale_docs,
                "issues": issue_payload,
                "remediation_allowed": remediation_allowed,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
