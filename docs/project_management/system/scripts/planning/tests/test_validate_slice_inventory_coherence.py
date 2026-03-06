import json
import subprocess
import sys
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text(*, slice_prefix: str, slice_ids: list[str]) -> str:
    pws = [
        {
            "id": f"{slice_prefix}-PWS-contract",
            "role": "contract",
            "depends_on": [],
            "assumes": [],
            "owns": ["contract.md"],
        },
        {
            "id": f"{slice_prefix}-PWS-docs_validation",
            "role": "docs_validation",
            "depends_on": [f"{slice_prefix}-PWS-contract"],
            "assumes": [],
            "owns": ["plan.md"],
        },
    ]
    for slice_id in slice_ids:
        pws.append(
            {
                "id": f"{slice_prefix}-PWS-slice_spec_{slice_id.lower()}",
                "role": "slice_spec",
                "depends_on": [f"{slice_prefix}-PWS-contract"],
                "assumes": [],
                "owns": [f"slices/{slice_id}/{slice_id}-spec.md"],
            }
        )
    pws.append(
        {
            "id": f"{slice_prefix}-PWS-tasks_checkpoints",
            "role": "tasks_checkpoints",
            "depends_on": [f"{slice_prefix}-PWS-docs_validation", *(f"{slice_prefix}-PWS-slice_spec_{sid.lower()}" for sid in slice_ids)],
            "assumes": [],
            "owns": ["tasks.json", "session_log.md", "kickoff_prompts/", *(f"slices/{sid}/kickoff_prompts/" for sid in slice_ids)],
        }
    )

    headings = []
    for p in pws:
        headings.append(f"### {p['id']} — {p['role']}\n\n- Goal: fixture\n")

    idx = {"pws_index_version": 1, "slice_prefix": slice_prefix, "pws": pws}
    body = json.dumps(idx, indent=2, sort_keys=False)
    return (
        "# Workstream triage fixture\n\n"
        "## Planning workstreams (PWS)\n\n"
        + "\n".join(headings)
        + "\n"
        + f"{BEGIN}\n"
        + "```json\n"
        + body
        + "\n```\n"
        + f"{END}\n"
    )


def _minimal_spec_text(feature_name: str, slice_prefix: str, slice_ids: list[str], *, backtick_key: bool = False) -> str:
    blocks = []
    for slice_id in slice_ids:
        key = "`slice_id`" if backtick_key else "slice_id"
        blocks.append(
            f"- {key}: `{slice_id}`\n"
            f"  - name: Fixture {slice_id}\n"
            f"  - intent: Keep slice {slice_id} in the accepted skeleton.\n"
        )
    return (
        f"# {feature_name}\n\n"
        "## Draft slice skeleton (pre-planning only)\n\n"
        f"Slice prefix (draft): `{slice_prefix}`\n\n"
        + "\n".join(blocks)
    )


def _plan_text(feature_name: str, slice_ids: list[str]) -> str:
    headings = []
    for slice_id in slice_ids:
        headings.append(f"### {slice_id} — Fixture slice\n\n- Deliverable: `{slice_id}`\n")
    return (
        f"# {feature_name} — plan\n\n"
        "## Slices (sequencing)\n\n"
        + "\n".join(headings)
    )


def _checkpoint_plan_text(slice_ids: list[str], *, root_level: bool = False) -> str:
    checkpoints = [
        {
            "id": "CP1",
            "task_id": "CP1-ci-checkpoint",
            "slices": slice_ids,
            "gates": {"compile_parity": True, "feature_smoke": False, "ci_testing": "quick"},
            "rationale": "Fixture checkpoint covers the accepted slice set.",
        }
    ]
    return (
        "# Fixture checkpoint plan\n\n"
        "## Machine-readable plan (linted)\n\n"
        "```json\n"
        + json.dumps(
            {
                "version": 1,
                "defaults": {"min_triads_per_checkpoint": 1, "max_triads_per_checkpoint": 8},
                "checkpoints": checkpoints,
            },
            indent=2,
            sort_keys=False,
        )
        + "\n```\n"
    )


def _tasks_json(slice_ids: list[str]) -> dict:
    tasks = []
    for slice_id in slice_ids:
        tasks.extend(
            [
                {"id": f"{slice_id}-code", "type": "code"},
                {"id": f"{slice_id}-test", "type": "test"},
                {"id": f"{slice_id}-integ", "type": "integration"},
            ]
        )
    tasks.append({"id": "CP1-ci-checkpoint", "type": "ops"})
    return {"meta": {"schema_version": 4, "cross_platform": True}, "tasks": tasks}


class TestValidateSliceInventoryCoherence(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_slice_inventory_coherence"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_slice_inventory_coherence.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_slice_inventory_coherence.py not found at expected canonical path.")

    def _run(self, feature_dir: Path, phase: str) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.validator), "--feature-dir", str(feature_dir), "--phase", phase]
        return subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=str(self.repo_root))

    def _make_draft_pack(
        self,
        name: str,
        *,
        slice_ids: list[str],
        min_spec_slice_ids: list[str] | None = None,
        checkpoint_slice_ids: list[str] | None = None,
        task_slice_ids: list[str] | None = None,
        extra_spec_ids: list[str] | None = None,
        spec_manifest_slice_ids: list[str] | None = None,
        backticked_min_spec_keys: bool = False,
    ) -> Path:
        feature_dir = self.tmp_root / name
        slice_prefix = slice_ids[0][:-1]
        min_spec_slice_ids = min_spec_slice_ids or list(slice_ids)
        checkpoint_slice_ids = checkpoint_slice_ids or list(slice_ids)
        task_slice_ids = task_slice_ids or list(slice_ids)
        extra_spec_ids = extra_spec_ids or []
        spec_manifest_slice_ids = spec_manifest_slice_ids or list(slice_ids)

        _write_text(
            feature_dir / "pre-planning" / "minimal_spec_draft.md",
            _minimal_spec_text(name, slice_prefix, min_spec_slice_ids, backtick_key=backticked_min_spec_keys),
        )
        _write_text(feature_dir / "pre-planning" / "workstream_triage.md", _triage_text(slice_prefix=slice_prefix, slice_ids=slice_ids))
        _write_text(feature_dir / "plan.md", _plan_text(name, slice_ids))
        _write_text(feature_dir / "pre-planning" / "ci_checkpoint_plan.md", _checkpoint_plan_text(checkpoint_slice_ids))
        _write_text(
            feature_dir / "pre-planning" / "spec_manifest.md",
            "Canonical slice IDs selected for this feature:\n" + "\n".join(f"- `{sid}`" for sid in spec_manifest_slice_ids),
        )

        for slice_id in [*slice_ids, *extra_spec_ids]:
            _write_text(feature_dir / "slices" / slice_id / f"{slice_id}-spec.md", f"# {slice_id}\n")

        _write_text(feature_dir / "tasks.json", json.dumps(_tasks_json(task_slice_ids), indent=2, sort_keys=True))
        return feature_dir

    def _make_active_pack(self, name: str, *, slice_ids: list[str]) -> Path:
        feature_dir = self.tmp_root / name
        _write_text(feature_dir / "plan.md", _plan_text(name, slice_ids))
        _write_text(feature_dir / "ci_checkpoint_plan.md", _checkpoint_plan_text(slice_ids, root_level=True))
        for slice_id in slice_ids:
            _write_text(feature_dir / "slices" / slice_id / f"{slice_id}-spec.md", f"# {slice_id}\n")
        _write_text(feature_dir / "tasks.json", json.dumps(_tasks_json(slice_ids), indent=2, sort_keys=True))
        return feature_dir

    def test_passes_active_pack_without_pre_planning(self) -> None:
        feature_dir = self._make_active_pack("active_pack_pass", slice_ids=["C0"])
        res = self._run(feature_dir, PHASE_EXECUTION_READY)
        self.assertEqual(res.returncode, 0, msg=res.stderr)

    def test_passes_with_backticked_min_spec_as_only_slice_source(self) -> None:
        feature_dir = self.tmp_root / "backticked_min_spec_only"
        _write_text(
            feature_dir / "pre-planning" / "minimal_spec_draft.md",
            _minimal_spec_text("backticked_min_spec_only", "BEDPM", ["BEDPM0", "BEDPM1"], backtick_key=True),
        )
        res = self._run(feature_dir, PHASE_PRE_TASKS)
        self.assertEqual(res.returncode, 0, msg=res.stderr)

    def test_fails_pre_tasks_when_checkpoint_plan_drifts(self) -> None:
        feature_dir = self._make_draft_pack(
            "pre_tasks_checkpoint_drift",
            slice_ids=["BEDPM0", "BEDPM1", "BEDPM2"],
            checkpoint_slice_ids=["BEDPM0"],
        )
        res = self._run(feature_dir, PHASE_PRE_TASKS)
        self.assertEqual(res.returncode, 1)
        self.assertIn("ci_checkpoint_plan", res.stderr)
        self.assertIn("expected ['BEDPM0', 'BEDPM1', 'BEDPM2']", res.stderr)

    def test_fails_execution_ready_when_tasks_drop_accepted_slice(self) -> None:
        feature_dir = self._make_draft_pack(
            "execution_ready_missing_tasks",
            slice_ids=["BEDPM0", "BEDPM1", "BEDPM2"],
            task_slice_ids=["BEDPM0", "BEDPM1"],
        )
        res = self._run(feature_dir, PHASE_EXECUTION_READY)
        self.assertEqual(res.returncode, 1)
        self.assertIn("tasks_json", res.stderr)
        self.assertIn("missing ['BEDPM2']", res.stderr)

    def test_fails_when_slice_specs_are_orphaned(self) -> None:
        feature_dir = self._make_draft_pack(
            "orphaned_slice_spec",
            slice_ids=["BEDPM0", "BEDPM1"],
            extra_spec_ids=["BEDPM2"],
        )
        res = self._run(feature_dir, PHASE_PRE_TASKS)
        self.assertEqual(res.returncode, 1)
        self.assertIn("slice_specs", res.stderr)
        self.assertIn("extra ['BEDPM2']", res.stderr)

    def test_fails_when_spec_manifest_mentions_stale_slice_set(self) -> None:
        feature_dir = self._make_draft_pack(
            "spec_manifest_stale",
            slice_ids=["BEDPM0", "BEDPM1"],
            spec_manifest_slice_ids=["BEDPM0"],
        )
        res = self._run(feature_dir, PHASE_PRE_TASKS)
        self.assertEqual(res.returncode, 1)
        self.assertIn("spec_manifest", res.stderr)
        self.assertIn("missing ['BEDPM1']", res.stderr)

    def test_passes_when_triage_adopts_split_without_mutating_min_spec(self) -> None:
        feature_dir = self._make_draft_pack(
            "triage_adopted_split",
            slice_ids=["PDLDPM0", "PDLDPM1", "PDLDPM3", "PDLDPM2"],
            min_spec_slice_ids=["PDLDPM0", "PDLDPM1", "PDLDPM2"],
            spec_manifest_slice_ids=["PDLDPM0", "PDLDPM1", "PDLDPM3", "PDLDPM2"],
            backticked_min_spec_keys=True,
        )
        res = self._run(feature_dir, PHASE_PRE_TASKS)
        self.assertEqual(res.returncode, 0, msg=res.stderr)


PHASE_PRE_TASKS = "pre_tasks_checkpoints"
PHASE_EXECUTION_READY = "execution_ready"
