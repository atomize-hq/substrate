import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

FINGERPRINT = re.compile(r"^sha256:[0-9a-f]{64}$")
PRIORITIES = {"P1", "P2", "P3", "P4"}
STATUSES = {"in_progress", "complete", "bounded_stop"}
STOP_REASONS = {
    "authority_required",
    "budget_exhausted",
    "risk_expansion",
    "scope_expansion",
    "unrelated_blocker",
}
CYCLE_FIELDS = {
    "cycle_id",
    "kind",
    "subject_fingerprint",
    "trigger_cycle_id",
    "trigger_finding_ids",
    "causal_evidence_refs",
    "review_refs",
    "verdict",
    "findings",
}


class ValidationError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def require_string_list(value: Any, field: str, *, nonempty: bool = False) -> None:
    require(isinstance(value, list), f"{field} must be a list")
    require(
        all(isinstance(item, str) and item for item in value),
        f"{field} must contain non-empty strings",
    )
    require(len(value) == len(set(value)), f"{field} must not contain duplicates")
    if nonempty:
        require(bool(value), f"{field} must not be empty")


def require_enum(value: Any, choices: set[str], message: str) -> None:
    require(isinstance(value, str) and value in choices, message)


def validate_record(record: Any) -> None:
    require(isinstance(record, dict), "record must be a JSON object")
    require(
        set(record)
        == {"schema_version", "packet_id", "status", "stop_reason", "cycles"},
        "record fields must match the closed v1 shape",
    )
    require(
        type(record["schema_version"]) is int and record["schema_version"] == 1,
        "schema_version must equal 1",
    )
    require(
        isinstance(record["packet_id"], str) and record["packet_id"],
        "packet_id must be a non-empty string",
    )
    require_enum(record["status"], STATUSES, "status is invalid")
    require(
        isinstance(record["cycles"], list) and record["cycles"],
        "cycles must not be empty",
    )

    cycles = record["cycles"]
    supplemental_count = sum(
        isinstance(value, dict) and value.get("kind") == "supplemental_causal"
        for value in cycles
    )
    require(
        supplemental_count <= 2,
        "review lineage permits at most two supplemental cycles",
    )

    cycle_ids: set[str] = set()
    finding_ids: set[str] = set()
    previous: dict[str, Any] | None = None

    for index, current in enumerate(cycles):
        require(isinstance(current, dict), f"cycle {index} must be an object")
        require(
            set(current) == CYCLE_FIELDS,
            f"cycle {index} fields do not match the closed shape",
        )

        cycle_id = current["cycle_id"]
        require(
            isinstance(cycle_id, str) and cycle_id,
            f"cycle {index} has an invalid cycle_id",
        )
        require(cycle_id not in cycle_ids, f"cycle_id {cycle_id!r} is duplicated")
        cycle_ids.add(cycle_id)

        expected_kind = (
            "discovery"
            if index == 0
            else "closure"
            if index == 1
            else "supplemental_causal"
        )
        require(
            current["kind"] == expected_kind,
            f"cycle {cycle_id!r} must have kind {expected_kind!r}",
        )
        require(
            isinstance(current["subject_fingerprint"], str)
            and FINGERPRINT.fullmatch(current["subject_fingerprint"]) is not None,
            f"cycle {cycle_id!r} has an invalid subject fingerprint",
        )
        require_string_list(
            current["review_refs"], f"cycle {cycle_id!r} review_refs", nonempty=True
        )
        require_string_list(
            current["trigger_finding_ids"], f"cycle {cycle_id!r} trigger_finding_ids"
        )
        require_string_list(
            current["causal_evidence_refs"], f"cycle {cycle_id!r} causal_evidence_refs"
        )
        require_enum(
            current["verdict"],
            {"clean", "findings"},
            f"cycle {cycle_id!r} verdict is invalid",
        )
        require(
            isinstance(current["findings"], list),
            f"cycle {cycle_id!r} findings must be a list",
        )

        blocking_ids: list[str] = []
        for finding in current["findings"]:
            require(
                isinstance(finding, dict)
                and set(finding) == {"finding_id", "priority"},
                f"cycle {cycle_id!r} finding fields do not match the closed shape",
            )
            finding_id = finding["finding_id"]
            require(
                isinstance(finding_id, str) and finding_id,
                "finding_id must be non-empty",
            )
            require(
                finding_id not in finding_ids,
                f"finding_id {finding_id!r} is duplicated",
            )
            finding_ids.add(finding_id)
            require_enum(
                finding["priority"],
                PRIORITIES,
                f"finding {finding_id!r} priority is invalid",
            )
            if finding["priority"] in {"P1", "P2"}:
                blocking_ids.append(finding_id)

        if current["verdict"] == "clean":
            require(
                not blocking_ids,
                f"clean cycle {cycle_id!r} cannot carry P1/P2 findings",
            )
        else:
            require(
                bool(blocking_ids),
                f"findings cycle {cycle_id!r} must carry a P1/P2 finding",
            )

        if previous is None:
            require(
                current["trigger_cycle_id"] is None,
                "discovery cannot name a trigger cycle",
            )
            require(
                not current["trigger_finding_ids"],
                "discovery cannot name trigger findings",
            )
            require(
                not current["causal_evidence_refs"],
                "discovery cannot claim causal evidence",
            )
        else:
            require(
                previous["verdict"] == "findings",
                "a review cycle cannot follow a clean cycle",
            )
            require(
                current["subject_fingerprint"] != previous["subject_fingerprint"],
                f"cycle {cycle_id!r} must bind a changed subject fingerprint",
            )
            require(
                current["trigger_cycle_id"] == previous["cycle_id"],
                f"cycle {cycle_id!r} must trigger from the immediately preceding cycle",
            )
            expected_findings = [
                finding["finding_id"]
                for finding in previous["findings"]
                if finding["priority"] in {"P1", "P2"}
            ]
            require(
                current["trigger_finding_ids"] == expected_findings,
                f"cycle {cycle_id!r} must name exactly the preceding P1/P2 findings",
            )
            if current["kind"] == "supplemental_causal":
                require(
                    bool(current["causal_evidence_refs"]),
                    f"cycle {cycle_id!r} must cite causal evidence",
                )
            else:
                require(
                    not current["causal_evidence_refs"],
                    "closure records known-remediation lineage, not a supplemental causal claim",
                )

        previous = current

    status = record["status"]
    stop_reason = record["stop_reason"]
    final_verdict = cycles[-1]["verdict"]
    if status == "complete":
        require(stop_reason is None, "complete record cannot carry stop_reason")
        require(final_verdict == "clean", "complete record must end clean")
    elif status == "bounded_stop":
        require_enum(
            stop_reason, STOP_REASONS, "bounded_stop requires a recognized stop_reason"
        )
        require(
            final_verdict == "findings",
            "bounded_stop must preserve unresolved P1/P2 findings",
        )
        if stop_reason == "budget_exhausted":
            require(
                len(cycles) == 4,
                "budget_exhausted requires both supplemental cycles",
            )
    else:
        require(stop_reason is None, "in_progress record cannot carry stop_reason")


def validate_next_cycle(
    record: Any,
    next_kind: str,
    *,
    causal_evidence_refs: list[str] | None = None,
) -> None:
    validate_record(record)
    cycles = record["cycles"]
    require(
        cycles[-1]["verdict"] == "findings",
        "a review cycle cannot follow a clean cycle",
    )
    require(
        record["status"] == "in_progress",
        "only an in_progress record can authorize another cycle",
    )
    require(len(cycles) < 4, "review budget is exhausted")
    expected_kind = "closure" if len(cycles) == 1 else "supplemental_causal"
    require(next_kind == expected_kind, f"next cycle must have kind {expected_kind!r}")
    evidence_refs = causal_evidence_refs or []
    require_string_list(evidence_refs, "next cycle causal_evidence_refs")
    if next_kind == "supplemental_causal":
        require(bool(evidence_refs), "supplemental preflight must cite causal evidence")
    else:
        require(
            not evidence_refs,
            "closure preflight cannot claim supplemental causal evidence",
        )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate a runtime-refactor review-cycle record"
    )
    parser.add_argument("record", type=Path)
    parser.add_argument(
        "--next-cycle",
        choices=("closure", "supplemental_causal"),
        help="also verify that this next review-cycle kind is currently authorized",
    )
    parser.add_argument(
        "--causal-evidence-ref",
        action="append",
        default=[],
        help="repeatable evidence ref required before a supplemental_causal launch",
    )
    args = parser.parse_args()
    try:
        value = json.loads(args.record.read_text(encoding="utf-8"))
        validate_record(value)
        if args.next_cycle is not None:
            validate_next_cycle(
                value,
                args.next_cycle,
                causal_evidence_refs=args.causal_evidence_ref,
            )
    except (OSError, json.JSONDecodeError, ValidationError) as error:
        print(f"INVALID: {error}", file=sys.stderr)
        return 1
    suffix = f"; next cycle {args.next_cycle!r} authorized" if args.next_cycle else ""
    print(f"VALID: {args.record}{suffix}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
