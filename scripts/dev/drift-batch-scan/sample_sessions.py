#!/usr/bin/env python3
"""Freeze a current-native Codex rollout inventory for private batch selection.

The committed tests use synthetic rollouts. Real inventory and selection artifacts contain private
paths and identifiers and must stay in an untracked scratch directory (see README.md).
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import glob
import hashlib
import json
import os
import random
import re
from collections import Counter
from pathlib import Path
from typing import Iterable


@dataclasses.dataclass(frozen=True, order=True)
class Candidate:
    timestamp: str
    relative_path: str
    session_id: str
    path: str
    cwd: str | None
    repo: str
    month: str
    size: int
    source_digest: str
    rollout_format: str = "CurrentNativeV2"

    def digest_record(self) -> dict:
        return {
            "timestamp": self.timestamp,
            "relative_path": self.relative_path,
            "session_id": self.session_id,
            "cwd": self.cwd,
            "repo": self.repo,
            "month": self.month,
            "size": self.size,
            "source_digest": self.source_digest,
            "rollout_format": self.rollout_format,
        }

    def local_record(self) -> dict:
        return {"path": self.path, **self.digest_record()}


@dataclasses.dataclass(frozen=True, order=True)
class QuotaBucket:
    family: str
    label: str
    quota: int
    mandatory_population: int
    risk_class: str


@dataclasses.dataclass(frozen=True)
class QuotaConfig:
    schema_version: int
    seed: int
    required_families: tuple[str, ...]
    buckets: tuple[QuotaBucket, ...]

    @property
    def digest(self) -> str:
        return canonical_digest(self.public_record())

    def public_record(self) -> dict:
        return {
            "schema_version": self.schema_version,
            "seed": self.seed,
            "required_families": list(self.required_families),
            "buckets": [dataclasses.asdict(bucket) for bucket in self.buckets],
        }


@dataclasses.dataclass(frozen=True)
class FrozenInventory:
    as_of: str
    candidates: tuple[Candidate, ...]
    digest: str


def canonical_digest(value: object) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def freeze_quota_config(seed: int = 42) -> QuotaConfig:
    ordinary = {
        "language_repo": ("rust", "js_ts", "python", "docs_only", "mixed"),
        "workflow": ("implementation", "docs_planning", "verification", "review_fix", "mixed"),
        "tooling": ("cargo_rust", "node_npm", "python_pytest", "generic_filesystem_doc"),
        "delegation": ("single_agent",),
    }
    high_risk = {
        "delegation": ("delegated_parent_opaque", "delegated_child_visible"),
    }
    buckets = []
    for family, labels in ordinary.items():
        for label in labels:
            buckets.append(QuotaBucket(family, label, 3, 3, "ordinary"))
    for family, labels in high_risk.items():
        for label in labels:
            buckets.append(QuotaBucket(family, label, 5, 5, "high_risk"))
    return QuotaConfig(
        schema_version=1,
        seed=seed,
        required_families=("language_repo", "workflow", "tooling", "delegation"),
        buckets=tuple(sorted(buckets)),
    )


def parse_timestamp(value: str) -> dt.datetime:
    parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("timestamp must include a timezone")
    return parsed.astimezone(dt.timezone.utc)


def canonical_timestamp(value: str) -> str:
    parsed = parse_timestamp(value)
    return parsed.isoformat(timespec="seconds").replace("+00:00", "Z")


def norm_repo(cwd: str | None) -> str:
    if not cwd:
        return "(unknown)"
    collapsed = re.sub(r"/worktrees/[0-9a-fA-F]+/", "/", cwd)
    collapsed = re.sub(r"/\.conductor/[^/]+/", "/", collapsed)
    return os.path.basename(collapsed.rstrip("/")) or collapsed


def current_native_metadata(raw: bytes) -> tuple[str, str | None, str] | None:
    formats = set()
    identities = set()
    metadata = []
    for raw_line in raw.decode("utf-8", errors="replace").splitlines():
        try:
            row = json.loads(raw_line)
        except json.JSONDecodeError:
            continue
        if row.get("type") != "session_meta":
            continue
        payload = row.get("payload")
        if not isinstance(payload, dict):
            return None
        version = payload.get("multi_agent_version")
        if version == "v2":
            formats.add("CurrentNativeV2")
        elif version is None or version in ("disabled", "v1"):
            formats.add("Legacy")
        else:
            return None
        session_id = payload.get("id")
        if not isinstance(session_id, str) or not session_id.strip():
            return None
        identities.add(session_id.strip())
        timestamp = row.get("timestamp")
        if not isinstance(timestamp, str):
            return None
        metadata.append((timestamp, payload.get("cwd")))
    if formats != {"CurrentNativeV2"} or len(identities) != 1 or not metadata:
        return None
    timestamp, cwd = metadata[0]
    if cwd is not None and not isinstance(cwd, str):
        return None
    return next(iter(identities)), cwd, canonical_timestamp(timestamp)


def candidate_from_path(
    path: str | os.PathLike[str],
    *,
    sessions_root: str | os.PathLike[str],
    as_of: str,
    min_bytes: int,
    max_bytes: int,
) -> Candidate | None:
    candidate_path = Path(path).resolve()
    root = Path(sessions_root).resolve()
    try:
        relative = candidate_path.relative_to(root).as_posix()
        raw = candidate_path.read_bytes()
    except (OSError, ValueError):
        return None
    size = len(raw)
    if size < min_bytes or size > max_bytes:
        return None
    metadata = current_native_metadata(raw)
    if metadata is None:
        return None
    session_id, cwd, timestamp = metadata
    if parse_timestamp(timestamp) > parse_timestamp(as_of):
        return None
    return Candidate(
        timestamp=timestamp,
        relative_path=relative,
        session_id=session_id,
        path=str(candidate_path),
        cwd=cwd,
        repo=norm_repo(cwd),
        month=timestamp[:7],
        size=size,
        source_digest=hashlib.sha256(raw).hexdigest(),
    )


def freeze_inventory(
    paths: Iterable[str | os.PathLike[str]],
    *,
    sessions_root: str | os.PathLike[str],
    as_of: str,
    min_bytes: int = 1024,
    max_bytes: int = 15 * 1024 * 1024,
) -> FrozenInventory:
    canonical_as_of = canonical_timestamp(as_of)
    candidates = []
    for path in paths:
        candidate = candidate_from_path(
            path,
            sessions_root=sessions_root,
            as_of=canonical_as_of,
            min_bytes=min_bytes,
            max_bytes=max_bytes,
        )
        if candidate is not None:
            candidates.append(candidate)
    candidates.sort()
    session_ids = [candidate.session_id for candidate in candidates]
    if len(session_ids) != len(set(session_ids)):
        raise ValueError("current-native inventory contains duplicate session identities")
    digest = canonical_digest(
        {
            "schema_version": 1,
            "as_of": canonical_as_of,
            "candidates": [candidate.digest_record() for candidate in candidates],
        }
    )
    return FrozenInventory(canonical_as_of, tuple(candidates), digest)


def discover_rollout_paths(sessions_root: str) -> list[str]:
    return glob.glob(os.path.join(sessions_root, "**", "rollout-*.jsonl"), recursive=True)


def select_monthly_compatibility_sample(
    inventory: FrozenInventory, *, per_month: int, seed: int
) -> tuple[Candidate, ...]:
    """Preserve the R6 batch shape until P7's set-cover selector replaces it in P7-6.2."""
    rng = random.Random(seed)
    by_month: dict[str, list[Candidate]] = {}
    for candidate in inventory.candidates:
        by_month.setdefault(candidate.month, []).append(candidate)
    selected = []
    for month in sorted(by_month):
        candidates = list(by_month[month])
        candidates.sort(key=lambda candidate: (canonical_digest(candidate.digest_record()), candidate))
        rng.shuffle(candidates)
        preferred = []
        seen_repos = set()
        for candidate in candidates:
            if candidate.repo not in seen_repos:
                preferred.append(candidate)
                seen_repos.add(candidate.repo)
        picked = preferred[:per_month]
        picked_ids = {candidate.session_id for candidate in picked}
        picked.extend(
            candidate
            for candidate in candidates
            if candidate.session_id not in picked_ids
        )
        selected.extend(picked[:per_month])
    return tuple(sorted(selected))


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--sessions-root",
        default=os.path.expanduser("~/.codex/sessions"),
        help="private Codex rollout store (default: ~/.codex/sessions)",
    )
    parser.add_argument(
        "--as-of",
        required=True,
        help="inclusive RFC3339 inventory cutoff; required for reproducibility",
    )
    parser.add_argument("--out", default="selected_sessions.jsonl")
    parser.add_argument("--inventory-out", default="candidate_inventory.jsonl")
    parser.add_argument("--per-month", type=int, default=10)
    parser.add_argument("--min-bytes", type=int, default=1024)
    parser.add_argument("--max-bytes", type=int, default=15 * 1024 * 1024)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    config = freeze_quota_config(args.seed)
    inventory = freeze_inventory(
        discover_rollout_paths(args.sessions_root),
        sessions_root=args.sessions_root,
        as_of=args.as_of,
        min_bytes=args.min_bytes,
        max_bytes=args.max_bytes,
    )
    selected = select_monthly_compatibility_sample(
        inventory, per_month=args.per_month, seed=config.seed
    )

    with open(args.inventory_out, "w") as handle:
        for candidate in inventory.candidates:
            handle.write(json.dumps(candidate.local_record(), sort_keys=True) + "\n")
    with open(args.out, "w") as handle:
        for candidate in selected:
            handle.write(json.dumps(candidate.local_record(), sort_keys=True) + "\n")

    print(f"inventory candidates: {len(inventory.candidates)}")
    print(f"inventory as-of:      {inventory.as_of}")
    print(f"inventory digest:     {inventory.digest}")
    print(f"quota config digest:  {config.digest}")
    print(f"selected (compat):    {len(selected)}")
    month_counts = Counter(candidate.month for candidate in selected)
    print("selected by month:    " + json.dumps(dict(sorted(month_counts.items()))))
    print(f"private inventory ->  {args.inventory_out}")
    print(f"private selection ->  {args.out}")


if __name__ == "__main__":
    main()
