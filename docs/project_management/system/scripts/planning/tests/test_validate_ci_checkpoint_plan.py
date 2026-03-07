import json
import shutil
import subprocess
import sys
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text(slice_prefix: str, accepted_slice_order: list[str]) -> str:
    idx = {
        "pws_index_version": 2,
        "slice_prefix": slice_prefix,
        "accepted_slice_order": accepted_slice_order,
        "draft_slice_order": [accepted_slice_order[0]],
        "pws": [
            {
                "id": f"{slice_prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": f"{slice_prefix}-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": [f"{slice_prefix}-PWS-contract"],
                "assumes": [],
                "owns": [
                    "tasks.json",
                    "session_log.md",
                    "kickoff_prompts/",
                    *(f"slices/{sid}/kickoff_prompts/" for sid in accepted_slice_order),
                ],
            },
        ],
    }
    return (
        "# Workstream triage fixture\n\n"
        f"### {slice_prefix}-PWS-contract — contract\n\n- Goal: fixture\n\n"
        f"### {slice_prefix}-PWS-tasks_checkpoints — tasks_checkpoints\n\n- Goal: fixture\n\n"
        f"{BEGIN}\n```json\n{json.dumps(idx, indent=2, sort_keys=False)}\n```\n{END}\n"
    )


def _checkpoint_plan_text(slice_ids: list[str]) -> str:
    return (
        "# Fixture checkpoint plan\n\n"
        "## Machine-readable plan (linted)\n\n"
        "```json\n"
        + json.dumps(
            {
                "version": 1,
                "defaults": {"min_triads_per_checkpoint": 1, "max_triads_per_checkpoint": 8},
                "checkpoints": [
                    {
                        "id": "CP1",
                        "task_id": "CP1-ci-checkpoint",
                        "slices": slice_ids,
                        "gates": {"compile_parity": True, "feature_smoke": False, "ci_testing": "quick"},
                        "rationale": "fixture",
                    }
                ],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n```\n"
    )


def _tasks_json(slice_ids: list[str], checkpoint_boundary: str) -> dict:
    tasks = []
    for slice_id in slice_ids:
        tasks.extend(
            [
                {"id": f"{slice_id}-code", "type": "code", "depends_on": []},
                {"id": f"{slice_id}-test", "type": "test", "depends_on": []},
                {"id": f"{slice_id}-integ", "type": "integration", "depends_on": [f"{slice_id}-code", f"{slice_id}-test"]},
            ]
        )
    tasks.append({"id": f"{checkpoint_boundary}-integ-core", "type": "integration", "depends_on": [f"{checkpoint_boundary}-integ"]})
    tasks.append(
        {
            "id": "CP1-ci-checkpoint",
            "type": "ops",
            "kickoff_prompt": "kickoff_prompts/CP1-ci-checkpoint.md",
            "depends_on": [f"{checkpoint_boundary}-integ-core"],
        }
    )
    return {
        "meta": {
            "schema_version": 4,
            "cross_platform": True,
            "checkpoint_boundaries": [checkpoint_boundary],
        },
        "tasks": tasks,
    }


class TestValidateCiCheckpointPlan(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_ci_checkpoint_plan"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_ci_checkpoint_plan.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_ci_checkpoint_plan.py not found at expected canonical path.")

    def _make_feature_dir(
        self,
        name: str,
        *,
        task_slice_ids: list[str],
        checkpoint_slice_ids: list[str],
        include_triage: bool,
        accepted_slice_order: list[str] | None = None,
    ) -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)

        checkpoint_boundary = checkpoint_slice_ids[-1]
        _write_text(feature_dir / "pre-planning" / "ci_checkpoint_plan.md", _checkpoint_plan_text(checkpoint_slice_ids))
        _write_text(feature_dir / "kickoff_prompts" / "CP1-ci-checkpoint.md", "# kickoff\n")
        _write_text(feature_dir / "tasks.json", json.dumps(_tasks_json(task_slice_ids, checkpoint_boundary), indent=2, sort_keys=True))

        if include_triage:
            assert accepted_slice_order is not None
            slice_prefix = accepted_slice_order[0][:-1]
            _write_text(feature_dir / "pre-planning" / "workstream_triage.md", _triage_text(slice_prefix, accepted_slice_order))

        return feature_dir

    def _run(self, feature_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(self.validator), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def test_uses_v2_accepted_slice_order_when_present(self) -> None:
        feature_dir = self._make_feature_dir(
            "accepted_order_pass",
            task_slice_ids=["WDAP0", "WDAP1", "WDAP2", "WDAP3"],
            checkpoint_slice_ids=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
            include_triage=True,
            accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
        )
        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("OK: ci_checkpoint_plan validation passed", res.stdout)

    def test_fails_when_checkpoint_order_disagrees_with_v2_authority(self) -> None:
        feature_dir = self._make_feature_dir(
            "accepted_order_mismatch",
            task_slice_ids=["WDAP0", "WDAP1", "WDAP2", "WDAP3"],
            checkpoint_slice_ids=["WDAP0", "WDAP1", "WDAP2", "WDAP3"],
            include_triage=True,
            accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
        )
        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 1)
        self.assertIn("accepted slice order", res.stderr)
        self.assertIn("workstream_triage.md", res.stderr)

    def test_falls_back_to_numeric_task_order_without_triage_authority(self) -> None:
        feature_dir = self._make_feature_dir(
            "no_triage_numeric_fallback",
            task_slice_ids=["WDAP0", "WDAP1", "WDAP2", "WDAP3"],
            checkpoint_slice_ids=["WDAP0", "WDAP1", "WDAP2", "WDAP3"],
            include_triage=False,
        )
        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("OK: ci_checkpoint_plan validation passed", res.stdout)


if __name__ == "__main__":
    unittest.main()
