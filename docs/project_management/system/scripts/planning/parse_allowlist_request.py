#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def _non_empty_str(value: Any) -> str | None:
    if not isinstance(value, str):
        return None
    stripped = value.strip()
    return stripped or None


def _normalize_requested_paths(value: Any) -> list[str] | None:
    if not isinstance(value, list):
        return None

    normalized: list[str] = []
    for item in value:
        s = _non_empty_str(item)
        if s is None:
            return None
        normalized.append(s)
    return normalized


def parse_allowlist_request(*, request_path: Path, expected_pws_id: str | None = None) -> dict[str, Any]:
    out: dict[str, Any] = {
        "alias_used": None,
        "errors": [],
        "extra_keys": [],
        "path": str(request_path),
        "requested_tracked_paths": [],
        "status": "missing",
    }

    if not request_path.exists():
        return out

    try:
        raw = json.loads(request_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        out["status"] = "malformed"
        out["errors"].append(f"invalid JSON: {exc}")
        return out

    if not isinstance(raw, dict):
        out["status"] = "malformed"
        out["errors"].append("top-level value must be a JSON object")
        return out

    out["extra_keys"] = sorted(
        key for key in raw.keys() if key not in {"pws_id", "requested_tracked_paths", "requested_paths", "reason"}
    )

    pws_id = _non_empty_str(raw.get("pws_id"))
    if pws_id is None:
        out["errors"].append("missing or invalid 'pws_id' (expected non-empty string)")
    else:
        out["pws_id"] = pws_id

    reason = _non_empty_str(raw.get("reason"))
    if reason is None:
        out["errors"].append("missing or invalid 'reason' (expected non-empty string)")
    else:
        out["reason"] = reason

    requested = None
    if "requested_tracked_paths" in raw:
        requested = _normalize_requested_paths(raw.get("requested_tracked_paths"))
        if requested is None or not requested:
            out["errors"].append(
                "missing or invalid 'requested_tracked_paths' (expected array of non-empty strings)"
            )
    elif "requested_paths" in raw:
        requested = _normalize_requested_paths(raw.get("requested_paths"))
        out["alias_used"] = "requested_paths"
        if requested is None or not requested:
            out["errors"].append("legacy alias 'requested_paths' is present but invalid")
    else:
        out["errors"].append(
            "missing required path field (expected 'requested_tracked_paths'; legacy alias 'requested_paths' is accepted)"
        )

    if requested:
        out["requested_tracked_paths"] = requested

    if expected_pws_id:
        expected = expected_pws_id.strip()
        if expected and pws_id is not None and pws_id != expected:
            out["errors"].append(f"pws_id mismatch: expected {expected!r}, got {pws_id!r}")

    out["status"] = "ok" if not out["errors"] else "malformed"
    return out


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Parse and normalize a PWS allowlist_request.json file.")
    ap.add_argument("--request", required=True, help="Path to allowlist_request.json")
    ap.add_argument("--expected-pws-id", default="", help="Optional PWS id that the request must match")
    args = ap.parse_args(argv)

    request_path = Path(args.request)
    result = parse_allowlist_request(
        request_path=request_path,
        expected_pws_id=(args.expected_pws_id or "").strip() or None,
    )
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
