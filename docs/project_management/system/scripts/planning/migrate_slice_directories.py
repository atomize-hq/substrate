#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class Move:
    src: Path
    dst: Path


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _git_repo_root(at_path: Path) -> Path:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(at_path), "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as exc:
        raise SystemExit(f"ERROR: failed to locate repo root via git: {exc}")
    if not out:
        raise SystemExit("ERROR: git rev-parse returned empty repo root")
    return Path(out)


def _feature_dir_rel(feature_dir: Path, repo_root: Path) -> str:
    if feature_dir.is_absolute():
        rel = os.path.relpath(feature_dir, repo_root)
    else:
        rel = os.path.normpath(str(feature_dir)).lstrip("./")
    return Path(rel).as_posix().rstrip("/")


def _read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def _derive_slice_ids(tasks_data: dict[str, Any]) -> list[str]:
    tasks = tasks_data.get("tasks")
    if not isinstance(tasks, list):
        raise SystemExit("ERROR: tasks.json must contain top-level tasks[] array")

    slice_ids: set[str] = set()
    for t in tasks:
        if not isinstance(t, dict):
            continue
        if t.get("type") != "code":
            continue
        task_id = t.get("id")
        if isinstance(task_id, str) and task_id.endswith("-code"):
            slice_ids.add(task_id[: -len("-code")])

    return sorted(slice_ids)


def _collect_moves(feature_dir: Path, slice_ids: list[str]) -> tuple[list[Move], list[str]]:
    moves: list[Move] = []
    missing: list[str] = []

    kickoff_root = feature_dir / "kickoff_prompts"
    for slice_id in slice_ids:
        slice_dir = feature_dir / "slices" / slice_id
        kickoff_dir = slice_dir / "kickoff_prompts"

        spec_src = feature_dir / f"{slice_id}-spec.md"
        spec_dst = slice_dir / f"{slice_id}-spec.md"
        if spec_src.exists():
            moves.append(Move(src=spec_src, dst=spec_dst))
        else:
            missing.append(str(spec_src))

        closeout_src = feature_dir / f"{slice_id}-closeout_report.md"
        closeout_dst = slice_dir / f"{slice_id}-closeout_report.md"
        if closeout_src.exists():
            moves.append(Move(src=closeout_src, dst=closeout_dst))
        else:
            missing.append(str(closeout_src))

        if kickoff_root.exists():
            for prompt in sorted(kickoff_root.glob(f"{slice_id}-*.md")):
                moves.append(Move(src=prompt, dst=kickoff_dir / prompt.name))

    return moves, missing


def _build_rewrite_map(feature_dir_rel: str, moves: list[Move]) -> dict[str, str]:
    mapping: dict[str, str] = {}
    for m in moves:
        # Only rewrite strings that appear as repo-relative paths. We intentionally do not rewrite
        # feature-dir-relative references like "WS0-spec.md" or "kickoff_prompts/WS0-code.md".
        src = Path(feature_dir_rel) / m.src.name
        dst = Path(feature_dir_rel) / "slices" / m.dst.parent.name / m.dst.name

        # For kickoff prompts, preserve the full relative path under kickoff_prompts/.
        if m.src.parent.name == "kickoff_prompts":
            src = Path(feature_dir_rel) / "kickoff_prompts" / m.src.name
            dst = Path(feature_dir_rel) / "slices" / m.dst.parent.parent.name / "kickoff_prompts" / m.dst.name

        mapping[src.as_posix()] = dst.as_posix()
    return mapping


def _rewrite_strings(value: Any, mapping: dict[str, str], changed: list[dict[str, Any]], pointer: str) -> Any:
    if isinstance(value, str):
        updated = value
        replacements: list[dict[str, str]] = []

        # Rewrite repo-relative path substrings inside string values.
        #
        # This is necessary for checklist items like:
        #   "Complete ...: docs/.../C0-closeout_report.md"
        #
        # Sort by descending key length to avoid partial rewrites when keys overlap.
        for src, dst in sorted(mapping.items(), key=lambda kv: len(kv[0]), reverse=True):
            if src not in updated:
                continue
            updated = updated.replace(src, dst)
            replacements.append({"from": src, "to": dst})

        if updated != value:
            entry: dict[str, Any] = {"pointer": pointer, "from": value, "to": updated}
            if len(replacements) > 1:
                entry["replacements"] = replacements
            changed.append(entry)

        return updated
    if isinstance(value, list):
        out: list[Any] = []
        for i, item in enumerate(value):
            out.append(_rewrite_strings(item, mapping, changed, f"{pointer}/{i}"))
        return out
    if isinstance(value, dict):
        out: dict[str, Any] = {}
        for k, v in value.items():
            out[k] = _rewrite_strings(v, mapping, changed, f"{pointer}/{k}")
        return out
    return value


def _apply_moves(moves: list[Move], dry_run: bool) -> list[dict[str, str]]:
    applied: list[dict[str, str]] = []
    for m in moves:
        if m.src == m.dst:
            continue
        if not m.src.exists():
            continue
        if dry_run:
            continue
        m.dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(m.src), str(m.dst))
        applied.append({"from": str(m.src), "to": str(m.dst)})
    return applied


def _rewrite_spec_manifest(spec_manifest: Path, mapping: dict[str, str], dry_run: bool) -> list[dict[str, str]]:
    if not spec_manifest.exists():
        return []

    text = spec_manifest.read_text(encoding="utf-8")
    changes: list[dict[str, str]] = []
    updated = text

    # Only rewrite spec paths (and only when the manifest already uses full repo-relative paths).
    for src, dst in sorted(mapping.items()):
        if not src.endswith("-spec.md"):
            continue
        if src in updated:
            updated = updated.replace(src, dst)
            changes.append({"from": src, "to": dst})

    if not dry_run and updated != text:
        spec_manifest.write_text(updated, encoding="utf-8")
    return changes


def _rewrite_markdown_tree(feature_dir: Path, mapping: dict[str, str], dry_run: bool) -> list[dict[str, str]]:
    """
    Rewrite repo-relative path strings in markdown files under the feature directory.

    This intentionally operates on raw text (not a markdown AST). We only replace exact repo-relative
    paths (e.g. docs/project_management/packs/.../<slice>-spec.md) using the mapping keys.

    Excludes feature logs to avoid mutating large/generated artifacts.
    """
    changes: list[dict[str, str]] = []

    for path in sorted(feature_dir.rglob("*.md")):
        if not path.is_file():
            continue
        if "logs" in path.parts:
            continue

        text = path.read_text(encoding="utf-8")
        updated = text
        file_changes: list[dict[str, str]] = []

        for src, dst in sorted(mapping.items()):
            if src not in updated:
                continue
            updated = updated.replace(src, dst)
            file_changes.append({"from": src, "to": dst})

        if updated != text:
            for c in file_changes:
                changes.append({"path": str(path), **c})
            if not dry_run:
                path.write_text(updated, encoding="utf-8")

    return changes


def _rewrite_sequencing_json(repo_root: Path, mapping: dict[str, str], dry_run: bool) -> list[dict[str, str]]:
    """
    Best-effort rewrite of spec paths in docs/project_management/packs/sequencing.json.

    This prevents latent drift when a slice spec is moved into slices/<SLICE_ID>/ but the sequencing
    entry still points at the old flat file location.
    """
    sequencing = repo_root / "docs/project_management/packs/sequencing.json"
    if not sequencing.exists():
        return []

    text = sequencing.read_text(encoding="utf-8")
    updated = text
    changes: list[dict[str, str]] = []

    spec_mapping = {k: v for k, v in mapping.items() if k.endswith("-spec.md")}
    for src, dst in sorted(spec_mapping.items()):
        if src in updated:
            updated = updated.replace(src, dst)
            changes.append({"from": src, "to": dst})

    if not dry_run and updated != text:
        sequencing.write_text(updated, encoding="utf-8")
    return changes


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Migrate slice specs/closeouts/kickoff prompts into slices/<SLICE_ID>/ layout.")
    ap.add_argument("--feature-dir", required=True, help="Planning pack directory under docs/project_management/packs/<bucket>/<feature>")
    ap.add_argument("--dry-run", action="store_true", help="Print planned actions only (default if neither flag is provided)")
    ap.add_argument("--apply", action="store_true", help="Apply filesystem moves and file rewrites")
    ap.add_argument("--report-json", help="Optional path to write a JSON migration report")
    args = ap.parse_args(argv)

    dry_run = bool(args.dry_run)
    apply = bool(args.apply)
    if not dry_run and not apply:
        dry_run = True

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists() or not feature_dir.is_dir():
        _eprint(f"ERROR: --feature-dir must be an existing directory: {feature_dir}")
        return 2

    tasks_json = feature_dir / "tasks.json"
    if not tasks_json.exists():
        _eprint(f"ERROR: missing required file: {tasks_json}")
        return 2

    repo_root = _git_repo_root(feature_dir)
    feature_rel = _feature_dir_rel(feature_dir, repo_root=repo_root)

    tasks_data = _read_json(tasks_json)
    if not isinstance(tasks_data, dict):
        _eprint(f"ERROR: tasks.json must be a JSON object: {tasks_json}")
        return 2

    slice_ids = _derive_slice_ids(tasks_data)
    if not slice_ids:
        _eprint("ERROR: no slice ids discovered (expected code tasks with ids '*-code')")
        return 2

    moves, missing_sources = _collect_moves(feature_dir, slice_ids)
    mapping = _build_rewrite_map(feature_dir_rel=feature_rel, moves=moves)

    planned_moves = [{"from": str(m.src), "to": str(m.dst)} for m in moves if m.src.exists() and m.src != m.dst]

    report: dict[str, Any] = {
        "feature_dir": str(feature_dir),
        "feature_dir_rel": feature_rel,
        "slice_ids": slice_ids,
        "planned_moves": planned_moves,
        "missing_sources": missing_sources,
        "tasks_json_rewrites": [],
        "spec_manifest_rewrites": [],
        "markdown_rewrites": [],
        "sequencing_rewrites": [],
        "applied_moves": [],
    }

    if dry_run:
        print(json.dumps(report, indent=2))
        return 0

    if not apply:
        _eprint("ERROR: refusing to mutate without --apply")
        return 2

    report["applied_moves"] = _apply_moves(moves, dry_run=False)

    # Rewrite tasks.json after moves so any feature-dir-relative references keep working.
    changed: list[dict[str, Any]] = []
    rewritten = _rewrite_strings(tasks_data, mapping, changed, "")
    if rewritten != tasks_data:
        _write_json(tasks_json, rewritten)
    report["tasks_json_rewrites"] = changed

    spec_manifest = feature_dir / "spec_manifest.md"
    report["spec_manifest_rewrites"] = _rewrite_spec_manifest(spec_manifest, mapping, dry_run=False)

    report["markdown_rewrites"] = _rewrite_markdown_tree(feature_dir, mapping, dry_run=False)
    report["sequencing_rewrites"] = _rewrite_sequencing_json(repo_root, mapping, dry_run=False)

    if args.report_json:
        Path(args.report_json).write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
