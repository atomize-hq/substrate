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
import re
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


@dataclasses.dataclass(frozen=True)
class LabeledCandidate:
    candidate: Candidate
    labels: frozenset[str]

    def local_record(self) -> dict:
        strata: dict[str, list[str]] = {}
        for key in sorted(self.labels):
            family, label = key.split(":", 1)
            strata.setdefault(family, []).append(label)
        return {**self.candidate.local_record(), "strata": strata}

    def digest_record(self) -> dict:
        return {
            "candidate": self.candidate.digest_record(),
            "labels": sorted(self.labels),
        }


@dataclasses.dataclass(frozen=True, order=True)
class BucketCoverage:
    family: str
    label: str
    quota: int
    mandatory_population: int
    eligible: int
    selected: int
    underfill: int
    status: str


@dataclasses.dataclass(frozen=True)
class SelectionResult:
    selected: tuple[LabeledCandidate, ...]
    selected_digest: str
    coverage: tuple[BucketCoverage, ...]


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
        if not isinstance(row, dict):
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


def nested_strings(value: object) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [text for item in value for text in nested_strings(item)]
    if isinstance(value, dict):
        return [text for item in value.values() for text in nested_strings(item)]
    return []


def observable_source(candidate: Candidate) -> str:
    try:
        rows = [
            json.loads(line)
            for line in Path(candidate.path).read_text(errors="replace").splitlines()
            if line.strip()
        ]
    except OSError:
        rows = []
    except json.JSONDecodeError:
        rows = []
    values = [candidate.cwd or "", candidate.repo]
    for row in rows:
        row_type = row.get("type")
        payload = row.get("payload")
        if not isinstance(payload, dict):
            continue
        if row_type == "session_meta":
            if payload.get("source") is not None:
                values.append(json.dumps(payload["source"], sort_keys=True))
        elif row_type == "turn_context":
            values.extend(nested_strings(payload.get("user_instructions")))
        elif row_type == "response_item":
            payload_type = payload.get("type")
            if payload_type == "message" and payload.get("role") == "user":
                values.extend(nested_strings(payload.get("content")))
            elif payload_type in {
                "function_call",
                "custom_tool_call",
                "local_shell_call",
            }:
                values.extend(nested_strings(payload.get("name")))
                values.extend(nested_strings(payload.get("arguments")))
                values.extend(nested_strings(payload.get("input")))
        elif row_type == "event_msg" and payload.get("type") in {
            "user_message",
            "task_started",
        }:
            values.extend(nested_strings(payload.get("message")))
            values.extend(nested_strings(payload.get("text")))
    return "\n".join(values).lower()


def assign_observable_labels(candidate: Candidate) -> LabeledCandidate:
    source = observable_source(candidate)
    labels = set()

    language_signals = {
        "rust": (".rs", "cargo", "rustc", "clippy", "rustfmt"),
        "js_ts": (".js", ".jsx", ".ts", ".tsx", "node", "npm", "pnpm", "yarn"),
        "python": (".py", "python", "pytest", "poetry", " uv "),
    }
    languages = {
        label
        for label, signals in language_signals.items()
        if any(signal in source for signal in signals)
    }
    labels.update(f"language_repo:{label}" for label in languages)
    if len(languages) > 1:
        labels.add("language_repo:mixed")
    if not languages and any(
        signal in source
        for signal in (".md", ".mdx", "readme", "/docs/", "/specs/", "documentation")
    ):
        labels.add("language_repo:docs_only")
    if not any(label.startswith("language_repo:") for label in labels):
        labels.add("language_repo:unknown")

    workflow_signals = {
        "implementation": ("implement", "build", "create", "edit", "write code"),
        "docs_planning": (" plan", "spec", "document", "research", "design"),
        "verification": (" test", "verify", "validation", "check", "clippy"),
        "review_fix": ("review", " fix", "debug", "failure", "error"),
    }
    workflows = {
        label
        for label, signals in workflow_signals.items()
        if any(signal in source for signal in signals)
    }
    labels.update(f"workflow:{label}" for label in workflows)
    if len(workflows) > 1:
        labels.add("workflow:mixed")
    if not workflows:
        labels.add("workflow:unknown")

    tooling_signals = {
        "cargo_rust": ("cargo", "rustc", "clippy", "rustfmt"),
        "node_npm": ("node", "npm", "npx", "pnpm", "yarn", "vitest", "jest"),
        "python_pytest": ("python", "pytest", "poetry", " uv "),
        "generic_filesystem_doc": (
            "readme",
            "documentation",
            "/docs/",
            "/specs/",
            "\"name\":\"shell_command\"",
        ),
    }
    tooling = {
        label
        for label, signals in tooling_signals.items()
        if any(signal in source for signal in signals)
    }
    labels.update(f"tooling:{label}" for label in tooling)
    if not tooling:
        labels.add("tooling:unknown")

    child_visible = (
        ("subagent" in source and "thread_spawn" in source)
        or any(
            signal in source
            for signal in (
                "child rollout",
                "separate rollout",
                "spawned agent",
                "child session id",
            )
        )
    )
    delegated_parent = any(
        signal in source
        for signal in ("spawn_agent", "wait_agent", "close_agent", "multi_agent_v1")
    )
    if child_visible:
        labels.add("delegation:delegated_child_visible")
    elif delegated_parent:
        labels.add("delegation:delegated_parent_opaque")
    else:
        labels.add("delegation:single_agent")

    return LabeledCandidate(candidate, frozenset(labels))


def bucket_key(bucket: QuotaBucket) -> str:
    return f"{bucket.family}:{bucket.label}"


def validate_selection(
    config: QuotaConfig,
    candidates: Iterable[LabeledCandidate],
    selected: Iterable[LabeledCandidate],
) -> tuple[BucketCoverage, ...]:
    candidates = tuple(candidates)
    selected = tuple(selected)
    if not selected:
        raise ValueError("selected set must be non-empty")
    candidate_ids = {candidate.candidate.session_id for candidate in candidates}
    selected_ids = {candidate.candidate.session_id for candidate in selected}
    if len(selected_ids) != len(selected) or not selected_ids <= candidate_ids:
        raise ValueError("selected set must contain distinct inventory candidates")

    coverage = []
    for bucket in config.buckets:
        key = bucket_key(bucket)
        eligible = sum(key in candidate.labels for candidate in candidates)
        selected_count = sum(key in candidate.labels for candidate in selected)
        underfill = max(bucket.quota - selected_count, 0)
        if eligible >= bucket.mandatory_population and selected_count < bucket.quota:
            raise ValueError(
                f"sufficiently populated bucket {key} selected {selected_count}/{bucket.quota}"
            )
        if eligible < bucket.quota and selected_count != eligible:
            raise ValueError(
                f"scarce bucket {key} must select every eligible candidate "
                f"({selected_count}/{eligible})"
            )
        status = "filled"
        if underfill:
            status = "permitted_inventory_scarcity"
        coverage.append(
            BucketCoverage(
                bucket.family,
                bucket.label,
                bucket.quota,
                bucket.mandatory_population,
                eligible,
                selected_count,
                underfill,
                status,
            )
        )

    for family in config.required_families:
        recognized = any(
            any(
                label.startswith(f"{family}:") and not label.endswith(":unknown")
                for label in candidate.labels
            )
            for candidate in candidates
        )
        represented = any(
            any(
                label.startswith(f"{family}:") and not label.endswith(":unknown")
                for label in candidate.labels
            )
            for candidate in selected
        )
        if recognized and not represented:
            raise ValueError(f"required family {family} has no named selected representative")
    return tuple(coverage)


def select_overlapping_quotas(
    candidates: Iterable[LabeledCandidate], config: QuotaConfig
) -> SelectionResult:
    candidates = tuple(
        sorted(candidates, key=lambda candidate: candidate.candidate)
    )
    if len({candidate.candidate.session_id for candidate in candidates}) != len(candidates):
        raise ValueError("candidate set contains duplicate session identities")
    deficits = {bucket_key(bucket): bucket.quota for bucket in config.buckets}
    remaining = list(candidates)
    selected = []
    while remaining:
        scored = []
        for candidate in remaining:
            gain = sum(
                deficits.get(label, 0) > 0
                for label in candidate.labels
                if not label.endswith(":unknown")
            )
            tie = canonical_digest(
                {
                    "seed": config.seed,
                    "candidate": candidate.digest_record(),
                }
            )
            scored.append((-gain, tie, candidate.candidate, candidate))
        scored.sort(key=lambda item: item[:3])
        negative_gain, _, _, best = scored[0]
        if negative_gain == 0:
            break
        selected.append(best)
        remaining.remove(best)
        for label in best.labels:
            if deficits.get(label, 0) > 0:
                deficits[label] -= 1

    selected = tuple(sorted(selected, key=lambda candidate: candidate.candidate))
    coverage = validate_selection(config, candidates, selected)
    selected_digest = canonical_digest(
        {
            "schema_version": 1,
            "config_digest": config.digest,
            "selected": [candidate.digest_record() for candidate in selected],
        }
    )
    return SelectionResult(selected, selected_digest, coverage)


def build_selection_receipt(
    inventory: FrozenInventory,
    config: QuotaConfig,
    selection: SelectionResult,
) -> dict:
    return {
        "schema_version": "p7-private-selection-v1",
        "inventory": {
            "route": "CurrentNativeV2",
            "as_of": inventory.as_of,
            "digest": inventory.digest,
            "candidate_count": len(inventory.candidates),
        },
        "quota_config": {
            **config.public_record(),
            "digest": config.digest,
        },
        "selection": {
            "digest": selection.selected_digest,
            "selected_count": len(selection.selected),
        },
        "coverage": [
            dataclasses.asdict(bucket) for bucket in selection.coverage
        ],
        "privacy": {
            "raw_private_fields_included": False,
        },
    }


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
    parser.add_argument("--receipt-out", default="selection_receipt.json")
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
    labeled = tuple(assign_observable_labels(candidate) for candidate in inventory.candidates)
    selection = select_overlapping_quotas(labeled, config)

    with open(args.inventory_out, "w") as handle:
        for candidate in labeled:
            handle.write(json.dumps(candidate.local_record(), sort_keys=True) + "\n")
    with open(args.out, "w") as handle:
        for candidate in selection.selected:
            handle.write(json.dumps(candidate.local_record(), sort_keys=True) + "\n")
    receipt = build_selection_receipt(inventory, config, selection)
    with open(args.receipt_out, "w") as handle:
        json.dump(receipt, handle, indent=2, sort_keys=True)
        handle.write("\n")

    print(f"inventory candidates: {len(inventory.candidates)}")
    print(f"inventory as-of:      {inventory.as_of}")
    print(f"inventory digest:     {inventory.digest}")
    print(f"quota config digest:  {config.digest}")
    print(f"selected sessions:    {len(selection.selected)}")
    print(f"selected-set digest:  {selection.selected_digest}")
    print(
        "quota coverage:       "
        + json.dumps(
            [dataclasses.asdict(bucket) for bucket in selection.coverage],
            sort_keys=True,
            separators=(",", ":"),
        )
    )
    print(f"private inventory ->  {args.inventory_out}")
    print(f"private selection ->  {args.out}")
    print(f"sanitized receipt ->  {args.receipt_out}")


if __name__ == "__main__":
    main()
