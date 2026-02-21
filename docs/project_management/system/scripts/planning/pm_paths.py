#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _usage_error(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def _repo_root() -> Path:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as e:
        _usage_error(f"not in a git repo/worktree (git rev-parse failed): {e}")

    if not out:
        _usage_error("git rev-parse returned empty repo root")
    return Path(out).resolve()


def _abspath_in_repo(repo_root: Path, raw: str) -> Path:
    p = Path(raw)
    if p.is_absolute():
        abs_path = p.resolve()
    else:
        abs_path = (repo_root / p).resolve()

    try:
        abs_path.relative_to(repo_root)
    except Exception:
        _usage_error(f"path resolves outside repo root: {raw!r} -> {abs_path}")
    return abs_path


def _relposix(repo_root: Path, abs_path: Path) -> str:
    try:
        rel = abs_path.relative_to(repo_root)
    except Exception:
        _usage_error(f"path is outside repo root: {abs_path}")
    return rel.as_posix()


def _env_value(name: str) -> str | None:
    v = os.environ.get(name)
    if v is None:
        return None
    v = v.strip()
    return v if v else None


def _compute_roots(repo_root: Path) -> dict[str, str]:
    pm_root_raw = _env_value("PM_ROOT") or "docs/project_management"
    pm_root_abs = _abspath_in_repo(repo_root, pm_root_raw)
    if not pm_root_abs.exists() or not pm_root_abs.is_dir():
        _usage_error(
            f"PM_ROOT does not exist or is not a directory: {pm_root_raw!r} (resolved to {pm_root_abs})"
        )

    pm_root = _relposix(repo_root, pm_root_abs)

    pm_system_root_raw = _env_value("PM_SYSTEM_ROOT") or f"{pm_root}/system"
    pm_adrs_root_raw = _env_value("PM_ADRS_ROOT") or f"{pm_root}/adrs"
    pm_packs_root_raw = _env_value("PM_PACKS_ROOT") or f"{pm_root}/packs"
    pm_workstreams_root_raw = _env_value("PM_WORKSTREAMS_ROOT") or f"{pm_root}/workstreams"
    pm_work_items_root_raw = _env_value("PM_WORK_ITEMS_ROOT") or f"{pm_root}/work_items"
    pm_default_pack_bucket = _env_value("PM_DEFAULT_PACK_BUCKET") or "active"

    pm_system_root_abs = _abspath_in_repo(repo_root, pm_system_root_raw)
    pm_adrs_root_abs = _abspath_in_repo(repo_root, pm_adrs_root_raw)
    pm_packs_root_abs = _abspath_in_repo(repo_root, pm_packs_root_raw)
    pm_workstreams_root_abs = _abspath_in_repo(repo_root, pm_workstreams_root_raw)
    pm_work_items_root_abs = _abspath_in_repo(repo_root, pm_work_items_root_raw)

    return {
        "pm_root": pm_root,
        "pm_system_root": _relposix(repo_root, pm_system_root_abs),
        "pm_adrs_root": _relposix(repo_root, pm_adrs_root_abs),
        "pm_packs_root": _relposix(repo_root, pm_packs_root_abs),
        "pm_workstreams_root": _relposix(repo_root, pm_workstreams_root_abs),
        "pm_work_items_root": _relposix(repo_root, pm_work_items_root_abs),
        "pm_default_pack_bucket": pm_default_pack_bucket,
    }


def cmd_print_roots() -> int:
    repo_root = _repo_root()
    roots = _compute_roots(repo_root)
    print(json.dumps(roots, sort_keys=True))
    return 0


def cmd_resolve_feature_dir(feature_dir: str) -> int:
    repo_root = _repo_root()
    abs_feature = Path(feature_dir)
    if abs_feature.is_absolute():
        abs_feature = abs_feature.resolve()
    else:
        abs_feature = (Path.cwd() / abs_feature).resolve()

    try:
        abs_feature.relative_to(repo_root)
    except Exception:
        _usage_error(f"--feature-dir resolves outside repo root: {feature_dir!r} -> {abs_feature}")

    print(_relposix(repo_root, abs_feature))
    return 0


def cmd_resolve_sequencing_json() -> int:
    repo_root = _repo_root()
    roots = _compute_roots(repo_root)

    pm_packs_root = roots["pm_packs_root"].rstrip("/")

    canonical = Path(repo_root) / pm_packs_root / "sequencing.json"

    if not canonical.exists():
        _usage_error(f"sequencing.json not found: {_relposix(repo_root, canonical)}")

    print(_relposix(repo_root, canonical))
    return 0


def cmd_resolve_workstreams_root() -> int:
    repo_root = _repo_root()
    roots = _compute_roots(repo_root)
    print(roots["pm_workstreams_root"])
    return 0


def cmd_resolve_work_items_root() -> int:
    repo_root = _repo_root()
    roots = _compute_roots(repo_root)
    print(roots["pm_work_items_root"])
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Resolve project_management roots and normalize feature dirs (repo-relative POSIX paths)."
    )
    sub = ap.add_subparsers(dest="cmd", required=True)

    sub.add_parser("print-roots", help="Print resolved PM roots as JSON (stdout JSON-only).")

    ap_resolve = sub.add_parser("resolve-feature-dir", help="Normalize a feature dir to repo-relative POSIX path.")
    ap_resolve.add_argument("--feature-dir", required=True)

    sub.add_parser(
        "resolve-sequencing-json",
        help="Print the canonical sequencing.json path under packs.",
    )
    sub.add_parser("resolve-workstreams-root", help="Print the workstreams registry root (repo-relative POSIX path).")
    sub.add_parser("resolve-work-items-root", help="Print the work_items registry root (repo-relative POSIX path).")

    args = ap.parse_args(argv)

    if args.cmd == "print-roots":
        return cmd_print_roots()
    if args.cmd == "resolve-feature-dir":
        return cmd_resolve_feature_dir(args.feature_dir)
    if args.cmd == "resolve-sequencing-json":
        return cmd_resolve_sequencing_json()
    if args.cmd == "resolve-workstreams-root":
        return cmd_resolve_workstreams_root()
    if args.cmd == "resolve-work-items-root":
        return cmd_resolve_work_items_root()

    _usage_error(f"unknown command: {args.cmd}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
