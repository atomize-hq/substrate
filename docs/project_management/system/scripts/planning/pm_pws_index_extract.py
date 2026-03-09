#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, NoReturn

import validate_pws_index as vpi


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _die(msg: str) -> NoReturn:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


class _ArgParser(argparse.ArgumentParser):
    def error(self, message: str) -> NoReturn:  # type: ignore[override]
        _die(message)


def _resolve_feature_dir(raw: str) -> Path:
    feature_dir = Path(raw).resolve()
    if not feature_dir.exists():
        _die(f"feature dir does not exist: {feature_dir}")
    if not feature_dir.is_dir():
        _die(f"feature dir is not a directory: {feature_dir}")
    return feature_dir


def _resolve_triage_path(feature_dir: Path, raw: str) -> Path:
    raw = (raw or "").strip()
    if not raw:
        _die("--workstream-triage cannot be empty")

    candidate = Path(raw)
    if not candidate.is_absolute():
        candidate = feature_dir / candidate

    if candidate.exists():
        return candidate

    # Legacy fallback only if the default path was used.
    if raw == vpi.DEFAULT_TRIAGE_REL:
        legacy = feature_dir / vpi.LEGACY_TRIAGE_REL
        if legacy.exists():
            return legacy

    _die(f"workstream triage artifact not found: {candidate}")


def _load_index(triage_path: Path) -> dict[str, Any]:
    try:
        text = triage_path.read_text(encoding="utf-8")
    except Exception as e:
        _die(f"unable to read triage artifact: {triage_path} ({e})")

    try:
        return vpi._extract_pm_pws_index_json(text)
    except Exception as e:
        _die(f"{triage_path}: {e}")


def main(argv: list[str]) -> int:
    ap = _ArgParser(description="Extract one PWS entry from PM_PWS_INDEX (stdout JSON-only).")
    ap.add_argument("--feature-dir", required=True, help="Feature directory (absolute or relative).")
    ap.add_argument("--pws-id", required=True, help="Exact PWS id to extract.")
    ap.add_argument(
        "--workstream-triage",
        default=vpi.DEFAULT_TRIAGE_REL,
        help=(
            "Path to workstream_triage.md (absolute or feature-dir-relative). "
            f"Default: {vpi.DEFAULT_TRIAGE_REL} (legacy fallback: {vpi.LEGACY_TRIAGE_REL})"
        ),
    )
    args = ap.parse_args(argv)

    feature_dir = _resolve_feature_dir(args.feature_dir)
    triage_path = _resolve_triage_path(feature_dir, args.workstream_triage)

    errors = vpi._validate_doc(feature_dir, triage_path, advisory=False)
    if errors:
        for e in errors:
            _eprint(f"ERROR: {e.message}")
        return 2

    idx = _load_index(triage_path)

    pws_id = str(args.pws_id).strip()
    if not pws_id:
        _die("--pws-id cannot be empty")

    pws_list = idx.get("pws")
    if not isinstance(pws_list, list):
        _die(f"{triage_path}: pws must be an array")

    entry: dict[str, Any] | None = None
    for raw in pws_list:
        if not isinstance(raw, dict):
            continue
        if raw.get("id") == pws_id:
            entry = raw
            break

    if entry is None:
        _die(f"unknown PWS_ID: {pws_id}")

    slice_prefix = idx.get("slice_prefix")
    if not isinstance(slice_prefix, str):
        _die(f"{triage_path}: slice_prefix must be a string")
    slice_prefix = slice_prefix.strip()

    role = entry.get("role")
    if not isinstance(role, str):
        _die(f"{triage_path}: {pws_id}.role must be a string")
    role = role.strip()

    depends_on = entry.get("depends_on")
    if not isinstance(depends_on, list) or not all(isinstance(x, str) for x in depends_on):
        _die(f"{triage_path}: {pws_id}.depends_on must be an array of strings")

    owns_raw = entry.get("owns")
    if not isinstance(owns_raw, list) or not all(isinstance(x, str) for x in owns_raw):
        _die(f"{triage_path}: {pws_id}.owns must be an array of strings")

    owns_norm = [vpi._normalize_owns_path(x) for x in owns_raw]
    owns_exact_norm = [x for x in owns_norm if not x.endswith("/")]
    owns_prefix_norm = [x for x in owns_norm if x.endswith("/")]

    out = {
        "depends_on": [x.strip() for x in depends_on],
        "owns_exact_norm": owns_exact_norm,
        "owns_norm": owns_norm,
        "owns_prefix_norm": owns_prefix_norm,
        "owns_raw": list(owns_raw),
        "pws_id": pws_id,
        "role": role,
        "slice_prefix": slice_prefix,
    }
    print(json.dumps(out, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
