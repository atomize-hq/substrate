import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from validate_review_cycle import ValidationError, validate_next_cycle, validate_record


def fingerprint(char: str) -> str:
    return f"sha256:{char * 64}"


def cycle(
    cycle_id: str,
    kind: str,
    subject_char: str,
    verdict: str,
    findings: list[tuple[str, str]],
    *,
    trigger_cycle_id: str | None = None,
    trigger_finding_ids: list[str] | None = None,
    causal_evidence_refs: list[str] | None = None,
) -> dict:
    return {
        "cycle_id": cycle_id,
        "kind": kind,
        "subject_fingerprint": fingerprint(subject_char),
        "trigger_cycle_id": trigger_cycle_id,
        "trigger_finding_ids": trigger_finding_ids or [],
        "causal_evidence_refs": causal_evidence_refs or [],
        "review_refs": [f"reviews/{cycle_id}.txt"],
        "verdict": verdict,
        "findings": [
            {"finding_id": finding_id, "priority": priority}
            for finding_id, priority in findings
        ],
    }


def record(cycles: list[dict], *, status: str = "complete", stop_reason=None) -> dict:
    return {
        "schema_version": 1,
        "packet_id": "A1.1d-5R2-3-example",
        "status": status,
        "stop_reason": stop_reason,
        "cycles": cycles,
    }


class ValidateReviewCycleTests(unittest.TestCase):
    def test_accepts_clean_discovery(self) -> None:
        validate_record(record([cycle("discovery", "discovery", "a", "clean", [])]))

    def test_accepts_closure_after_blocking_discovery(self) -> None:
        validate_record(
            record(
                [
                    cycle("discovery", "discovery", "a", "findings", [("P2-1", "P2")]),
                    cycle(
                        "closure",
                        "closure",
                        "b",
                        "clean",
                        [],
                        trigger_cycle_id="discovery",
                        trigger_finding_ids=["P2-1"],
                    ),
                ]
            )
        )

    def test_accepts_two_causal_supplemental_cycles(self) -> None:
        validate_record(
            record(
                [
                    cycle("d", "discovery", "a", "findings", [("P2-1", "P2")]),
                    cycle(
                        "c",
                        "closure",
                        "b",
                        "findings",
                        [("P2-2", "P2")],
                        trigger_cycle_id="d",
                        trigger_finding_ids=["P2-1"],
                    ),
                    cycle(
                        "s1",
                        "supplemental_causal",
                        "c",
                        "findings",
                        [("P1-3", "P1")],
                        trigger_cycle_id="c",
                        trigger_finding_ids=["P2-2"],
                        causal_evidence_refs=["proof/causal-1.txt"],
                    ),
                    cycle(
                        "s2",
                        "supplemental_causal",
                        "d",
                        "clean",
                        [],
                        trigger_cycle_id="s1",
                        trigger_finding_ids=["P1-3"],
                        causal_evidence_refs=["proof/causal-2.txt"],
                    ),
                ]
            )
        )

    def test_rejects_review_after_clean(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "clean", []),
                cycle("c", "closure", "b", "clean", []),
            ]
        )
        with self.assertRaisesRegex(ValidationError, "cannot follow a clean cycle"):
            validate_record(value)

    def test_rejects_third_supplemental_cycle(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2")]),
                cycle(
                    "c",
                    "closure",
                    "b",
                    "findings",
                    [("F2", "P2")],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["F1"],
                ),
                cycle(
                    "s1",
                    "supplemental_causal",
                    "c",
                    "findings",
                    [("F3", "P2")],
                    trigger_cycle_id="c",
                    trigger_finding_ids=["F2"],
                    causal_evidence_refs=["e1"],
                ),
                cycle(
                    "s2",
                    "supplemental_causal",
                    "d",
                    "findings",
                    [("F4", "P2")],
                    trigger_cycle_id="s1",
                    trigger_finding_ids=["F3"],
                    causal_evidence_refs=["e2"],
                ),
                cycle(
                    "s3",
                    "supplemental_causal",
                    "e",
                    "clean",
                    [],
                    trigger_cycle_id="s2",
                    trigger_finding_ids=["F4"],
                    causal_evidence_refs=["e3"],
                ),
            ]
        )
        with self.assertRaisesRegex(ValidationError, "at most two supplemental"):
            validate_record(value)

    def test_rejects_inexact_trigger_findings(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2"), ("A1", "P3")]),
                cycle(
                    "c",
                    "closure",
                    "b",
                    "clean",
                    [],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["A1"],
                ),
            ]
        )
        with self.assertRaisesRegex(
            ValidationError, "exactly the preceding P1/P2 findings"
        ):
            validate_record(value)

    def test_rejects_unchanged_post_remediation_subject(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2")]),
                cycle(
                    "c",
                    "closure",
                    "a",
                    "clean",
                    [],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["F1"],
                ),
            ]
        )
        with self.assertRaisesRegex(ValidationError, "changed subject fingerprint"):
            validate_record(value)

    def test_rejects_complete_record_without_final_clean(self) -> None:
        value = record([cycle("d", "discovery", "a", "findings", [("F1", "P2")])])
        with self.assertRaisesRegex(ValidationError, "complete record must end clean"):
            validate_record(value)

    def test_accepts_bounded_stop_with_blocking_findings(self) -> None:
        validate_record(
            record(
                [cycle("d", "discovery", "a", "findings", [("F1", "P2")])],
                status="bounded_stop",
                stop_reason="authority_required",
            )
        )

    def test_rejects_supplemental_without_causal_evidence(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2")]),
                cycle(
                    "c",
                    "closure",
                    "b",
                    "findings",
                    [("F2", "P2")],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["F1"],
                ),
                cycle(
                    "s1",
                    "supplemental_causal",
                    "c",
                    "clean",
                    [],
                    trigger_cycle_id="c",
                    trigger_finding_ids=["F2"],
                ),
            ]
        )
        with self.assertRaisesRegex(ValidationError, "causal evidence"):
            validate_record(value)

    def test_authorizes_closure_after_discovery_findings(self) -> None:
        value = record(
            [cycle("d", "discovery", "a", "findings", [("F1", "P2")])],
            status="in_progress",
        )
        validate_next_cycle(value, "closure")

    def test_rejects_supplemental_preflight_without_causal_evidence(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2")]),
                cycle(
                    "c",
                    "closure",
                    "b",
                    "findings",
                    [("F2", "P2")],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["F1"],
                ),
            ],
            status="in_progress",
        )
        with self.assertRaisesRegex(ValidationError, "causal evidence"):
            validate_next_cycle(value, "supplemental_causal")

    def test_authorizes_supplemental_preflight_with_causal_evidence(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2")]),
                cycle(
                    "c",
                    "closure",
                    "b",
                    "findings",
                    [("F2", "P2")],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["F1"],
                ),
            ],
            status="in_progress",
        )
        validate_next_cycle(
            value,
            "supplemental_causal",
            causal_evidence_refs=["proof/closure-unmasked-p2.txt"],
        )

    def test_rejects_next_cycle_after_clean(self) -> None:
        value = record([cycle("d", "discovery", "a", "clean", [])])
        with self.assertRaisesRegex(ValidationError, "cannot follow a clean cycle"):
            validate_next_cycle(value, "closure")

    def test_rejects_next_cycle_after_budget_exhaustion(self) -> None:
        value = record(
            [
                cycle("d", "discovery", "a", "findings", [("F1", "P2")]),
                cycle(
                    "c",
                    "closure",
                    "b",
                    "findings",
                    [("F2", "P2")],
                    trigger_cycle_id="d",
                    trigger_finding_ids=["F1"],
                ),
                cycle(
                    "s1",
                    "supplemental_causal",
                    "c",
                    "findings",
                    [("F3", "P2")],
                    trigger_cycle_id="c",
                    trigger_finding_ids=["F2"],
                    causal_evidence_refs=["e1"],
                ),
                cycle(
                    "s2",
                    "supplemental_causal",
                    "d",
                    "findings",
                    [("F4", "P2")],
                    trigger_cycle_id="s1",
                    trigger_finding_ids=["F3"],
                    causal_evidence_refs=["e2"],
                ),
            ],
            status="in_progress",
        )
        with self.assertRaisesRegex(ValidationError, "review budget is exhausted"):
            validate_next_cycle(value, "supplemental_causal")

    def test_malformed_status_fails_with_validation_error(self) -> None:
        value = record([cycle("d", "discovery", "a", "clean", [])])
        value["status"] = []
        with self.assertRaisesRegex(ValidationError, "status is invalid"):
            validate_record(value)

    def test_malformed_priority_fails_with_validation_error(self) -> None:
        value = record([cycle("d", "discovery", "a", "findings", [("F1", "P2")])])
        value["cycles"][0]["findings"][0]["priority"] = []
        with self.assertRaisesRegex(ValidationError, "priority is invalid"):
            validate_record(value)

    def test_rejects_premature_budget_exhausted_stop(self) -> None:
        value = record(
            [cycle("d", "discovery", "a", "findings", [("F1", "P2")])],
            status="bounded_stop",
            stop_reason="budget_exhausted",
        )
        with self.assertRaisesRegex(
            ValidationError, "requires both supplemental cycles"
        ):
            validate_record(value)


if __name__ == "__main__":
    unittest.main()
