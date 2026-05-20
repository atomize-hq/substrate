#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import tempfile
import uuid
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCRIPT_PATH = "docs/project_management/system/scripts/execution/prepare_proving_run_closeout.py"
REQUIRED_SOURCE_STATE = "published_baseline"
TARGET_STATE = "closed_baseline"
REQUIRED_HUMAN_FIELDS = ("residual_friction", "manual_edits")
ALLOWED_HUMAN_FIELDS = set(REQUIRED_HUMAN_FIELDS) | {"operator_notes", "follow_ups"}


class PreparationError(RuntimeError):
    pass


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Prepare a repo-owned proving-run closeout draft from machine-known lifecycle/publication facts "
            "plus optional human inputs."
        )
    )
    parser.add_argument("--facts", required=True, help="Path to the machine-known lifecycle/publication facts JSON.")
    parser.add_argument(
        "--human-inputs",
        help=(
            "Optional path to human-owned closeout inputs JSON. Allowed keys: residual_friction, manual_edits, "
            "operator_notes, follow_ups."
        ),
    )
    parser.add_argument(
        "--output",
        default="proving-run-closeout.json",
        help="Output path for the prepared closeout JSON. Defaults to proving-run-closeout.json.",
    )
    parser.add_argument("--force", action="store_true", help="Overwrite the output path if it already exists.")
    return parser.parse_args()


def read_json(path: Path) -> dict[str, Any]:
    try:
        with path.open("r", encoding="utf-8") as handle:
            payload = json.load(handle)
    except FileNotFoundError as exc:
        raise PreparationError(f"missing required input file: {path}") from exc
    except json.JSONDecodeError as exc:
        raise PreparationError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(payload, dict):
        raise PreparationError(f"expected top-level JSON object in {path}")
    return payload


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_facts(facts: dict[str, Any], source_path: Path) -> dict[str, Any]:
    if facts.get("schema_version") != 1:
        raise PreparationError(
            f"{source_path}: facts.schema_version must be 1 for prepare_proving_run_closeout.py"
        )
    if facts.get("mode") != "create":
        raise PreparationError(f"{source_path}: facts.mode must be 'create'")

    run_id = facts.get("run_id")
    if not isinstance(run_id, str) or not run_id.strip():
        raise PreparationError(f"{source_path}: facts.run_id must be a non-empty string")

    lifecycle = facts.get("lifecycle")
    if not isinstance(lifecycle, dict):
        raise PreparationError(f"{source_path}: facts.lifecycle must be an object")
    current_state = lifecycle.get("current_state")
    if current_state != REQUIRED_SOURCE_STATE:
        raise PreparationError(
            f"{source_path}: facts.lifecycle.current_state must be '{REQUIRED_SOURCE_STATE}', got {current_state!r}"
        )

    publication = facts.get("publication")
    if not isinstance(publication, dict):
        raise PreparationError(f"{source_path}: facts.publication must be an object")
    status = publication.get("status")
    if status != "green":
        raise PreparationError(f"{source_path}: facts.publication.status must be 'green', got {status!r}")

    published_at = publication.get("published_at")
    if not isinstance(published_at, str) or not published_at.strip():
        raise PreparationError(f"{source_path}: facts.publication.published_at must be a non-empty string")

    artifact_path = publication.get("artifact_path")
    if not isinstance(artifact_path, str) or not artifact_path.strip():
        raise PreparationError(f"{source_path}: facts.publication.artifact_path must be a non-empty string")

    evidence_refs = publication.get("evidence_refs")
    if evidence_refs is not None:
        if not isinstance(evidence_refs, list) or not all(isinstance(item, str) for item in evidence_refs):
            raise PreparationError(f"{source_path}: facts.publication.evidence_refs must be an array of strings")

    return facts


def validate_human_inputs(human_inputs: dict[str, Any], source_path: Path) -> dict[str, Any]:
    unknown = sorted(set(human_inputs) - ALLOWED_HUMAN_FIELDS)
    if unknown:
        raise PreparationError(
            f"{source_path}: unexpected human-owned keys: {', '.join(unknown)}"
        )

    if "residual_friction" in human_inputs:
        value = human_inputs["residual_friction"]
        if value is not None and not isinstance(value, str):
            raise PreparationError(f"{source_path}: residual_friction must be a string or null")

    if "manual_edits" in human_inputs:
        value = human_inputs["manual_edits"]
        if value is not None:
            if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
                raise PreparationError(f"{source_path}: manual_edits must be an array of strings or null")

    if "operator_notes" in human_inputs:
        value = human_inputs["operator_notes"]
        if value is not None and not isinstance(value, str):
            raise PreparationError(f"{source_path}: operator_notes must be a string or null")

    if "follow_ups" in human_inputs:
        value = human_inputs["follow_ups"]
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            raise PreparationError(f"{source_path}: follow_ups must be an array of strings")

    return human_inputs


def build_human_owned_section(human_inputs: dict[str, Any] | None) -> tuple[dict[str, Any], list[str]]:
    human_owned: dict[str, Any] = {
        "residual_friction": None,
        "manual_edits": None,
        "operator_notes": None,
        "follow_ups": [],
    }
    if human_inputs:
        for key, value in human_inputs.items():
            human_owned[key] = deepcopy(value)

    missing: list[str] = []
    for key in REQUIRED_HUMAN_FIELDS:
        if human_owned.get(key) is None:
            missing.append(f"human_owned.{key}")

    return human_owned, missing


def build_closeout(
    facts: dict[str, Any],
    facts_path: Path,
    human_inputs: dict[str, Any] | None,
    human_inputs_path: Path | None,
) -> dict[str, Any]:
    prepared_at = utc_now()
    transaction_id = str(uuid.uuid4())
    human_owned, missing_fields = build_human_owned_section(human_inputs)

    preparation_inputs: list[dict[str, str]] = [
        {
            "kind": "lifecycle_facts",
            "path": str(facts_path),
            "sha256": sha256_file(facts_path),
        }
    ]
    if human_inputs_path is not None:
        preparation_inputs.append(
            {
                "kind": "human_inputs",
                "path": str(human_inputs_path),
                "sha256": sha256_file(human_inputs_path),
            }
        )

    return {
        "schema_version": 1,
        "artifact_type": "proving_run_closeout",
        "prepared_at": prepared_at,
        "mode": "create",
        "preparation": {
            "script": SCRIPT_PATH,
            "transaction_id": transaction_id,
            "inputs": preparation_inputs,
        },
        "machine_owned": {
            "run_id": facts["run_id"],
            "lifecycle": {
                "source_state": facts["lifecycle"]["current_state"],
                "target_state": TARGET_STATE,
            },
            "publication": deepcopy(facts["publication"]),
            "facts_snapshot": deepcopy(facts),
        },
        "human_owned": human_owned,
        "handoff": {
            "status": "awaiting_human_inputs" if missing_fields else "ready_to_close",
            "required_human_fields": missing_fields,
        },
    }


def atomic_write_json(path: Path, payload: dict[str, Any], force: bool) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and not force:
        raise PreparationError(f"refusing to overwrite existing output without --force: {path}")

    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    fd, temp_path_raw = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    temp_path = Path(temp_path_raw)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(rendered)
            handle.flush()
            os.fsync(handle.fileno())
        temp_path.replace(path)
    finally:
        try:
            temp_path.unlink()
        except FileNotFoundError:
            pass


def main() -> int:
    args = parse_args()
    facts_path = Path(args.facts).resolve()
    output_path = Path(args.output).resolve()
    human_inputs_path = Path(args.human_inputs).resolve() if args.human_inputs else None

    try:
        facts = validate_facts(read_json(facts_path), facts_path)
        human_inputs = None
        if human_inputs_path is not None:
            human_inputs = validate_human_inputs(read_json(human_inputs_path), human_inputs_path)
        closeout = build_closeout(facts, facts_path, human_inputs, human_inputs_path)
        atomic_write_json(output_path, closeout, force=args.force)
    except PreparationError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    print(output_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
