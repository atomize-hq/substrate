#!/usr/bin/env python3
"""Run each sampled session through compactor -> analyzer in isolation.

Step 2 of the semantic-goal-drift real-world validation pipeline
(see README.md). Each session is compacted and analyzed inside its own
temp codex-home so cross-session exact-dedupe in the compactor cannot
contaminate results. Per-session `checkpoints.jsonl` outputs are written
under `<batch-dir>/checkpoints/<session_id>.jsonl`, each row annotated
with `_month`, `_repo`, `_session_file_id` for downstream tabulation.

Requires the release/debug binaries to be built first:
    cargo build -p agent-session-compactor -p agent-drift-analyzer
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from collections import Counter
from pathlib import Path


def find_repo_root(start: str) -> str:
    d = os.path.abspath(start)
    while d != "/":
        if os.path.exists(os.path.join(d, "Cargo.toml")) and os.path.isdir(os.path.join(d, "crates")):
            return d
        d = os.path.dirname(d)
    raise SystemExit("could not locate repo root (Cargo.toml + crates/) above " + start)


# Session-level delegation stratification (R6-3.5). Coarse marker scan over the raw rollout using
# the analyzer's own delegation vocabulary (crates/agent-drift-analyzer/src/inference/mod.rs). This
# is a *reporting* tag so the batch precision numbers can be reported separately for delegated vs
# single-agent traces; it is intentionally coarser than the analyzer's per-checkpoint
# DelegationContext (topology + child_work_visibility). Categories mirror the R6-3.5 charter:
# single_agent / delegated_parent_opaque / delegated_child_visible / unknown.
_DELEGATION_MARKERS = ("multi_agent_v1", "spawn_agent", "wait_agent", "close_agent")
_CHILD_VISIBILITY_SIGNALS = (
    "child rollout", "separate rollout", "subagent", "spawned agent", "child session id",
)
_FORBIDDEN_RECEIPT_KEYS = {
    "path",
    "relative_path",
    "cwd",
    "repo",
    "session_id",
    "source_file_id",
    "message",
    "text",
    "selected_ids",
    "repositories",
}
_CANDIDATE_DIGEST_FIELDS = (
    "timestamp",
    "relative_path",
    "session_id",
    "cwd",
    "repo",
    "month",
    "size",
    "source_digest",
    "rollout_format",
)


def classify_session_delegation(rollout_path: str) -> str:
    try:
        with open(rollout_path, "r", errors="replace") as f:
            blob = f.read().lower()
    except OSError:
        return "unknown"
    has_marker = any(m in blob for m in _DELEGATION_MARKERS)
    has_child_visibility = any(s in blob for s in _CHILD_VISIBILITY_SIGNALS)
    if not has_marker and not has_child_visibility:
        return "single_agent"
    if has_child_visibility:
        return "delegated_child_visible"
    return "delegated_parent_opaque"


def validate_safe_receipt(value, key_path=()) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in _FORBIDDEN_RECEIPT_KEYS:
                joined = ".".join((*key_path, key))
                raise ValueError(f"forbidden private key in receipt: {joined}")
            validate_safe_receipt(child, (*key_path, key))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            validate_safe_receipt(child, (*key_path, str(index)))
    elif isinstance(value, str):
        if value.startswith("/") or "~/.codex" in value or re_windows_absolute_path(value):
            joined = ".".join(key_path)
            raise ValueError(f"absolute private path in receipt: {joined}")


def re_windows_absolute_path(value: str) -> bool:
    return len(value) >= 3 and value[0].isalpha() and value[1:3] in (":\\", ":/")


def canonical_digest(value: object) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def selected_manifest_digest(records: list[dict], config_digest: str) -> str:
    selected = []
    for index, record in enumerate(records):
        try:
            candidate = {field: record[field] for field in _CANDIDATE_DIGEST_FIELDS}
        except KeyError as error:
            raise ValueError(
                f"selected manifest record {index} is missing {error.args[0]}"
            ) from error
        strata = record.get("strata")
        if not isinstance(strata, dict):
            raise ValueError(f"selected manifest record {index} has invalid strata")
        labels = []
        for family, values in sorted(strata.items()):
            if not isinstance(family, str) or not isinstance(values, list):
                raise ValueError(f"selected manifest record {index} has invalid strata")
            for label in values:
                if not isinstance(label, str):
                    raise ValueError(
                        f"selected manifest record {index} has invalid stratum label"
                    )
                labels.append(f"{family}:{label}")
        selected.append({"candidate": candidate, "labels": sorted(labels)})
    return canonical_digest(
        {
            "schema_version": 1,
            "config_digest": config_digest,
            "selected": selected,
        }
    )


def validate_selected_source_bytes(
    record: dict,
    index: int,
    *,
    path: str | os.PathLike[str] | None = None,
) -> None:
    source_value = path if path is not None else record.get("path")
    declared_size = record.get("size")
    declared_digest = record.get("source_digest")
    if not isinstance(source_value, (str, os.PathLike)) or not str(source_value):
        raise ValueError(f"selected manifest record {index} has an invalid source path")
    source_path = Path(source_value)
    if not isinstance(declared_size, int) or declared_size < 0:
        raise ValueError(f"selected manifest record {index} has an invalid source size")
    if (
        not isinstance(declared_digest, str)
        or len(declared_digest) != 64
        or any(character not in "0123456789abcdef" for character in declared_digest)
    ):
        raise ValueError(f"selected manifest record {index} has an invalid source digest")

    digest = hashlib.sha256()
    actual_size = 0
    try:
        with source_path.open("rb") as handle:
            while chunk := handle.read(1024 * 1024):
                actual_size += len(chunk)
                digest.update(chunk)
    except OSError as error:
        raise ValueError(
            f"selected manifest record {index} source cannot be read"
        ) from error
    if actual_size != declared_size or digest.hexdigest() != declared_digest:
        raise ValueError(
            f"selected manifest record {index} does not match declared source bytes"
        )


def validate_selected_manifest(receipt: dict, records: list[dict]) -> None:
    selection = receipt["selection"]
    if len(records) != selection["selected_count"]:
        raise ValueError("selected manifest count does not match selection receipt")
    identities = [record.get("session_id") for record in records]
    if any(not isinstance(identity, str) or not identity for identity in identities):
        raise ValueError("selected manifest contains an invalid session identity")
    if len(identities) != len(set(identities)):
        raise ValueError("selected manifest contains duplicate session identities")
    actual_digest = selected_manifest_digest(
        records, receipt["quota_config"]["digest"]
    )
    if actual_digest != selection["digest"]:
        raise ValueError("selected manifest does not match selected-set digest")
    for index, record in enumerate(records):
        validate_selected_source_bytes(record, index)


def prepare_batch_directories(
    batch_dir: str | os.PathLike[str],
) -> tuple[Path, Path]:
    batch_path = Path(batch_dir)
    if batch_path.exists():
        raise ValueError("batch directory must not already exist; use a fresh scratch path")
    checkpoints = batch_path / "checkpoints"
    work = batch_path / "work"
    checkpoints.mkdir(parents=True)
    work.mkdir()
    return checkpoints, work


def checkpoint_set_summary(checkpoints_dir: str | os.PathLike[str]) -> dict:
    records = []
    for path in sorted(Path(checkpoints_dir).glob("*.jsonl")):
        raw = path.read_bytes()
        checkpoint_count = sum(bool(line.strip()) for line in raw.splitlines())
        records.append(
            {
                "bytes": len(raw),
                "checkpoint_count": checkpoint_count,
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        )
    return {
        "schema_version": 1,
        "file_count": len(records),
        "checkpoint_count": sum(record["checkpoint_count"] for record in records),
        "digest": canonical_digest(
            {
                "schema_version": 1,
                "files": sorted(
                    records,
                    key=lambda record: (
                        record["sha256"],
                        record["bytes"],
                        record["checkpoint_count"],
                    ),
                ),
            }
        ),
    }


def load_selection_receipt(path: str) -> dict:
    with open(path) as handle:
        receipt = json.load(handle)
    validate_safe_receipt(receipt)
    if receipt.get("schema_version") != "p7-private-selection-v1":
        raise ValueError("selection receipt has the wrong schema_version")
    selected_count = (receipt.get("selection") or {}).get("selected_count")
    if not isinstance(selected_count, int) or selected_count < 1:
        raise ValueError("selection receipt must prove a non-empty selected set")
    if not isinstance(receipt.get("coverage"), list) or not receipt["coverage"]:
        raise ValueError("selection receipt must include quota coverage")
    return receipt


def build_batch_receipt(
    selection_receipt: dict,
    status: list[dict],
    checkpoint_set: dict,
) -> dict:
    validate_safe_receipt(selection_receipt)
    validate_safe_receipt(checkpoint_set)
    failure_kinds = Counter()
    for record in status:
        if record.get("ok"):
            continue
        error = str(record.get("error") or "")
        prefix = error.split(":", 1)[0]
        failure_kinds[prefix if prefix in {"compact", "analyze"} else "other"] += 1
    succeeded = sum(bool(record.get("ok")) for record in status)
    receipt = {
        "schema_version": "p7-private-batch-v1",
        "selection": selection_receipt,
        "batch": {
            "attempted_count": len(status),
            "succeeded_count": succeeded,
            "failed_count": len(status) - succeeded,
            "failure_kinds": dict(sorted(failure_kinds.items())),
        },
        "checkpoint_set": checkpoint_set,
    }
    if checkpoint_set.get("file_count") != succeeded:
        raise ValueError("checkpoint set file count does not match successful sessions")
    validate_safe_receipt(receipt)
    return receipt


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--repo", default=None,
                    help="repo root (default: auto-detect from this script's location)")
    ap.add_argument("--profile", default="debug", choices=["debug", "release"],
                    help="which target/<profile> binaries to use (default: debug)")
    ap.add_argument("--selected", default="selected_sessions.jsonl",
                    help="manifest from sample_sessions.py (default: ./selected_sessions.jsonl)")
    ap.add_argument("--selection-receipt", default="selection_receipt.json",
                    help="sanitized authority receipt from sample_sessions.py")
    ap.add_argument("--batch-dir", default="batch",
                    help="output dir for per-session checkpoints (default: ./batch)")
    ap.add_argument("--batch-receipt", default=None,
                    help="sanitized batch receipt (default: <batch-dir>/batch_receipt.json)")
    ap.add_argument("--compact-timeout", type=int, default=180)
    ap.add_argument("--analyze-timeout", type=int, default=300)
    args = ap.parse_args()

    repo = args.repo or find_repo_root(__file__)
    compact = os.path.join(repo, "target", args.profile, "agent-session-compactor")
    analyze = os.path.join(repo, "target", args.profile, "agent-drift-analyzer")
    for b in (compact, analyze):
        if not os.path.exists(b):
            raise SystemExit(f"missing binary {b}\n  build first: "
                             f"cargo build -p agent-session-compactor -p agent-drift-analyzer")

    selection_receipt = load_selection_receipt(args.selection_receipt)
    sessions = [json.loads(l) for l in open(args.selected) if l.strip()]
    try:
        validate_selected_manifest(selection_receipt, sessions)
        checkpoints_path, work_path = prepare_batch_directories(args.batch_dir)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    ckdir = str(checkpoints_path)
    work = str(work_path)
    status = []
    ok = 0
    for i, s in enumerate(sessions, 1):
        sid = s["session_id"]
        path = s["path"]
        tmp = tempfile.mkdtemp(dir=work)
        ch = os.path.join(tmp, "codex-home", "sessions")
        os.makedirs(ch, exist_ok=True)
        bundle = os.path.join(tmp, "bundle")
        analysis = os.path.join(tmp, "analysis")
        os.makedirs(bundle, exist_ok=True)
        os.makedirs(analysis, exist_ok=True)
        rec = {"session_id": sid, "month": s["month"], "repo": s["repo"],
               "size": s["size"], "ok": False, "n_checkpoints": 0, "error": None}
        selected_delegation = (s.get("strata") or {}).get("delegation") or []
        delegation = (
            selected_delegation[0]
            if len(selected_delegation) == 1
            else classify_session_delegation(path)
        )
        rec["delegation"] = delegation
        checkpoint_output = os.path.join(ckdir, sid + ".jsonl")
        checkpoint_temp = checkpoint_output + ".tmp"
        try:
            copied_path = shutil.copy(path, ch)
            validate_selected_source_bytes(s, i - 1, path=copied_path)
            r1 = subprocess.run([compact, "--codex-home", os.path.join(tmp, "codex-home"),
                                 "--output-dir", bundle], capture_output=True, text=True,
                                timeout=args.compact_timeout)
            if r1.returncode != 0:
                rec["error"] = "compact:" + (r1.stderr.strip().splitlines()[-1]
                                             if r1.stderr.strip() else "rc%d" % r1.returncode)
                raise RuntimeError(rec["error"])
            r2 = subprocess.run([analyze, "--input-dir", bundle, "--output-dir", analysis],
                                capture_output=True, text=True, timeout=args.analyze_timeout)
            if r2.returncode != 0:
                rec["error"] = "analyze:" + (r2.stderr.strip().splitlines()[-1]
                                             if r2.stderr.strip() else "rc%d" % r2.returncode)
                raise RuntimeError(rec["error"])
            cpsrc = os.path.join(analysis, "checkpoints.jsonl")
            n = 0
            with open(checkpoint_temp, "w") as out:
                for line in open(cpsrc):
                    if not line.strip():
                        continue
                    cp = json.loads(line)
                    cp["_month"] = s["month"]
                    cp["_repo"] = s["repo"]
                    cp["_session_file_id"] = sid
                    cp["_delegation"] = delegation
                    cp["_selection_strata"] = s.get("strata", {})
                    out.write(json.dumps(cp) + "\n")
                    n += 1
            os.replace(checkpoint_temp, checkpoint_output)
            rec["ok"] = True
            rec["n_checkpoints"] = n
            ok += 1
        except Exception as e:
            try:
                os.unlink(checkpoint_temp)
            except FileNotFoundError:
                pass
            if not rec["error"]:
                rec["error"] = str(e)[:200]
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        status.append(rec)
        if i % 15 == 0 or i == len(sessions):
            print(f"  {i}/{len(sessions)} done ok={ok}", flush=True)

    with open(os.path.join(args.batch_dir, "status.jsonl"), "w") as f:
        for r in status:
            f.write(json.dumps(r) + "\n")
    print(f"\nRAN {len(sessions)} sessions | ok={ok} | failed={len(sessions) - ok}")
    errs = Counter((r["error"] or "").split(":")[0] for r in status if not r["ok"])
    if errs:
        print("failure kinds:", dict(errs))
    batch_receipt = build_batch_receipt(
        selection_receipt,
        status,
        checkpoint_set_summary(ckdir),
    )
    batch_receipt_path = args.batch_receipt or os.path.join(
        args.batch_dir, "batch_receipt.json"
    )
    with open(batch_receipt_path, "w") as handle:
        json.dump(batch_receipt, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print(f"sanitized batch receipt: {batch_receipt_path}")
    if batch_receipt["batch"]["failed_count"]:
        raise SystemExit("batch failed; aggregate receipt is not eligible for signoff")


if __name__ == "__main__":
    main()
